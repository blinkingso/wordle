use std::{
    io,
    ops::{Deref, DerefMut},
    panic, thread,
    time::Duration,
};

use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture, Event as CrosstermEvent, KeyEventKind},
    terminal::{self, is_raw_mode_enabled, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::{FutureExt, StreamExt};
use ratatui::{prelude::CrosstermBackend, Terminal};
use tokio::{
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

use crate::error::{Result, WordError};

use super::event::Event;

pub type CrosstermTerminal = Terminal<CrosstermBackend<io::Stderr>>;
pub type Frame<'a> = ratatui::Frame<'a, CrosstermBackend<io::Stderr>>;

pub struct Tui {
    pub terminal: CrosstermTerminal,
    pub task: JoinHandle<()>,
    pub event_tx: UnboundedSender<Event>,
    pub event_rx: UnboundedReceiver<Event>,
    pub frame_rate: f64,
    pub tick_rate: f64,
    pub cancel_token: CancellationToken,
}

impl Tui {
    pub fn new() -> Result<Self> {
        let frame_rate = 60.0;
        let tick_rate = 4.0;
        let terminal = CrosstermTerminal::new(CrosstermBackend::new(std::io::stderr()))?;
        let (event_tx, event_rx) = unbounded_channel();
        let task = tokio::spawn(async {});
        let cancel_token = CancellationToken::new();
        Ok(Tui {
            terminal,
            task,
            event_tx,
            event_rx,
            frame_rate,
            tick_rate,
            cancel_token,
        })
    }

    pub fn tick_rate(mut self, tick_rate: f64) -> Self {
        self.tick_rate = tick_rate;
        self
    }

    pub fn frame_rate(mut self, frame_rate: f64) -> Self {
        self.frame_rate = frame_rate;
        self
    }

    pub fn cancel(&self) {
        self.cancel_token.cancel();
    }

    pub fn resume(&mut self) -> Result<()> {
        self.enter()?;
        Ok(())
    }

    pub fn start(&mut self) {
        let tick_delay = Duration::from_secs_f64(1.0 / self.tick_rate);
        let render_delay = Duration::from_secs_f64(1.0 / self.frame_rate);
        self.cancel();
        self.cancel_token = CancellationToken::new();
        let _cancel_token = self.cancel_token.clone();
        let _event_tx = self.event_tx.clone();

        self.task = tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            let mut tick_interval = tokio::time::interval(tick_delay);
            let mut render_interval = tokio::time::interval(render_delay);
            _event_tx.send(Event::Init).unwrap();
            loop {
                let tick_delay = tick_interval.tick();
                let render_delay = render_interval.tick();
                let crossterm_event = reader.next().fuse();
                tokio::select! {
                    _ = _cancel_token.cancelled() => {
                        break;
                    }
                    maybe_event = crossterm_event => {
                        match maybe_event {
                            Some(Ok(evt)) => {
                                match evt {
                                    CrosstermEvent::Key(key) => {
                                    if key.kind == KeyEventKind::Press {
                                        _event_tx.send(Event::Key(key)).unwrap();
                                    }
                                    },
                                    CrosstermEvent::Mouse(mouse) => {
                                        _event_tx.send(Event::Mouse(mouse)).unwrap();
                                    },
                                    _ => {}
                                }
                            },
                            Some(Err(_)) => {
                                _event_tx.send(Event::Error).unwrap();
                            },
                            None => {},
                        }
                    }
                    _ = tick_delay => {
                        _event_tx.send(Event::Tick).unwrap();
                    }
                    _ = render_delay => {
                        _event_tx.send(Event::Render).unwrap();
                    }
                }
            }
        });
    }

    pub fn enter(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;
        crossterm::execute!(
            io::stderr(),
            EnterAlternateScreen,
            EnableMouseCapture,
            cursor::Hide
        )?;

        self.start();
        Ok(())
    }

    fn reset() -> Result<()> {
        terminal::disable_raw_mode()?;
        crossterm::execute!(io::stderr(), LeaveAlternateScreen, DisableMouseCapture)?;

        Ok(())
    }

    fn stop(&self) -> Result<()> {
        self.cancel();
        let mut counter = 0;
        while !self.task.is_finished() {
            thread::sleep(Duration::from_millis(1));
            counter += 1;
            if counter > 50 {
                self.task.abort();
            }

            if counter > 100 {
                eprintln!("Failed to abort task in 100 milliseconds for unknown reason");
                break;
            }
        }
        Ok(())
    }

    pub fn exit(&mut self) -> Result<()> {
        self.stop()?;
        if is_raw_mode_enabled()? {
            self.flush()?;
            crossterm::execute!(
                io::stderr(),
                DisableMouseCapture,
                LeaveAlternateScreen,
                cursor::Show
            )?;
            crossterm::terminal::disable_raw_mode()?;
        }
        self.terminal.show_cursor()?;
        Ok(())
    }

    pub async fn next(&mut self) -> Result<Event> {
        self.event_rx
            .recv()
            .await
            .ok_or(WordError::EyreError(color_eyre::eyre::eyre!(
                "Unable to get event"
            )))
    }
}

impl Deref for Tui {
    type Target = CrosstermTerminal;

    fn deref(&self) -> &Self::Target {
        &self.terminal
    }
}

impl DerefMut for Tui {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.terminal
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        self.exit().unwrap();
    }
}
