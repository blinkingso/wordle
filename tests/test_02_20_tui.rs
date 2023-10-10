use std::io;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::{Constraint, CrosstermBackend, Layout},
    Terminal,
};
use wordle::tui::widgets::Button;

fn init() -> Result<(), io::Error> {
    enable_raw_mode()?;
    crossterm::execute!(io::stderr(), EnterAlternateScreen, EnableMouseCapture)?;
    Ok(())
}

fn startup() -> Result<Terminal<CrosstermBackend<io::Stderr>>, io::Error> {
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn exit(terminal: &mut Terminal<CrosstermBackend<io::Stderr>>) -> Result<(), io::Error> {
    disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    Ok(())
}

#[test]
fn test_layout() -> Result<(), io::Error> {
    init()?;
    let mut terminal = startup()?;
    loop {
        terminal.draw(|f| {
            let size = f.size();
            let mut row_constraints = vec![];
            for _ in 0..size.height as usize {
                row_constraints.push(ratatui::prelude::Constraint::Length(1));
            }
            let row_layout = Layout::default()
                .direction(ratatui::prelude::Direction::Vertical)
                .constraints(row_constraints)
                .split(f.size());
            let col_constraints = (0..size.width)
                .into_iter()
                .map(|_| Constraint::Length(2))
                .collect::<Vec<_>>();

            for j in 0..size.height as usize {
                if j % 2 == 0 {
                    let column_layout = Layout::default()
                        .direction(ratatui::prelude::Direction::Horizontal)
                        .constraints(col_constraints.clone())
                        .split(row_layout[j]);
                    for i in 0..size.width as usize {
                        if i % 2 != 0 {
                            f.render_widget(Button::new("A"), column_layout[i]);
                        }
                    }
                }
            }
        })?;

        if let event::Event::Key(key_event) = event::read()? {
            if let KeyCode::Esc = key_event.code {
                break;
            }
        }
    }
    exit(&mut terminal)?;
    Ok(())
}
