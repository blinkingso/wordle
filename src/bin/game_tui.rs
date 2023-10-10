use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, KeyCode, KeyEvent, KeyEventKind, MouseEvent,
        MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::Paragraph};
use std::{io, ops::ControlFlow, time::Duration};
use wordle::{
    cmd::KEYBOARD_KEYS,
    error::WordError,
    tui::{
        ui::{init_keyboard, Keyboard},
        widgets::Button,
    },
};

fn main() -> Result<(), WordError> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let res = run_app(&mut terminal, KEYBOARD_KEYS.as_slice());

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, keyboards: &[char]) -> io::Result<()> {
    let mut keyboards2 = init_keyboard();
    loop {
        terminal.draw(|frame| ui(frame, keyboards, &mut keyboards2))?;
        if !event::poll(Duration::from_millis(100))? {
            continue;
        }
        match event::read()? {
            event::Event::Key(key) => {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                if handle_key_event(key).is_break() {
                    break;
                }
            }
            event::Event::Mouse(mouse) => {
                // todo handle mouse click
                handle_mouse_event(mouse, keyboards);
            }
            _ => {}
        }
    }
    Ok(())
}

fn ui<B: Backend>(
    f: &mut Frame<'_, B>,
    keyboards: &[char],
    keyboards2: &mut Vec<Vec<Keyboard<'_>>>,
) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(5),
            Constraint::Length(20),
            Constraint::Length(2),
            Constraint::Min(0),
        ])
        .split(f.size());
    f.render_widget(Paragraph::new("Keyboards Widgets:"), layout[0]);
    render_buttons(f, layout[1], keyboards);
    render_keyboards(f, layout[2], keyboards2);
    f.render_widget(Paragraph::new("esc: quit"), layout[3]);
}

fn render_keyboards<B: Backend>(
    f: &mut Frame<'_, B>,
    area: Rect,
    keyboards: &mut Vec<Vec<Keyboard<'_>>>,
) {
    let layout = Layout::default()
        .margin(1)
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(area);
    for (index, keyboard) in keyboards.iter_mut().enumerate() {
        let mut constraints = keyboard
            .iter()
            .map(|_| Constraint::Length(6))
            .collect::<Vec<_>>();
        constraints.push(Constraint::Min(0));
        let line_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(layout[index * 2]);
        for (idx, k) in keyboard.iter().enumerate() {
            let button_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(90), Constraint::Percentage(10)])
                .split(line_layout[idx]);
            f.render_widget(Button::new(k.label()), button_layout[0]);
        }
    }
}

fn render_buttons<B: Backend>(f: &mut Frame<'_, B>, area: Rect, keyboards: &[char]) {
    let mut constrants = keyboards
        .iter()
        .map(|_| Constraint::Length(5))
        .collect::<Vec<_>>();
    constrants.push(Constraint::Min(0));
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constrants)
        .split(area);
    for (index, keyboard) in keyboards.iter().enumerate() {
        let area = layout[index];
        let button_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
            .split(area);
        f.render_widget(Button::new(keyboard.to_string()), button_layout[0]);
    }
}

fn handle_key_event(keyevent: KeyEvent) -> ControlFlow<()> {
    match keyevent.code {
        KeyCode::Esc => {
            return ControlFlow::Break(());
        }
        _ => (),
    }
    ControlFlow::Continue(())
}

fn handle_mouse_event(event: MouseEvent, _keyboards: &[char]) {
    if event.kind == MouseEventKind::Moved {}
    // println!("mouse (x,y) => ({},{})", event.row, event.column);
}
