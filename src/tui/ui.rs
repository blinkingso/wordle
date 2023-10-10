use std::{ops::ControlFlow, time::Duration};

use crossterm::event::{self, KeyCode, KeyEvent, MouseEvent};
use ratatui::{
    prelude::{Backend, Constraint, Layout},
    style::Color,
    style::Style,
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};

use crate::{
    error::Result,
    state::LetterState,
    word::Word,
    wordle::{CheckResult, Wordle},
};

use super::widgets::{Theme, THEME_X};

pub enum KeyboardType {
    Char(char),
    Backspace,
    Enter,
}

pub struct Keyboard<'a> {
    x: u16,
    y: u16,
    label: Line<'a>,
    theme: Theme,
    state: LetterState,
    ktype: KeyboardType,
}

impl<'a> Keyboard<'a> {
    pub fn new(x: u16, y: u16, ktype: KeyboardType) -> Self {
        let label = match ktype {
            KeyboardType::Char(ch) => ch.to_string(),
            KeyboardType::Backspace => "⇦".to_string(),
            KeyboardType::Enter => "Enter".to_string(),
        };
        Self {
            x,
            y,
            label: label.into(),
            theme: THEME_X,
            state: LetterState::X,
            ktype,
        }
    }

    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn state(mut self, state: LetterState) -> Self {
        self.state = state;
        self
    }

    pub fn label(&self) -> Line<'_> {
        self.label.clone()
    }
}

pub fn init_keyboard<'a>() -> Vec<Vec<Keyboard<'a>>> {
    let mut res = vec![];
    res.push(
        ['Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P']
            .into_iter()
            .enumerate()
            .map(|(index, ch)| Keyboard::new(0, index as u16, KeyboardType::Char(ch)))
            .collect(),
    );
    res.push(
        ['A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L']
            .into_iter()
            .enumerate()
            .map(|(index, ch)| Keyboard::new(1, index as u16, KeyboardType::Char(ch)))
            .collect(),
    );
    let mut last = vec![Keyboard::new(2, 0, KeyboardType::Backspace)];
    last.extend(
        ['Z', 'X', 'C', 'V', 'B', 'N', 'M']
            .into_iter()
            .enumerate()
            .map(|(index, ch)| Keyboard::new(2, (index + 1) as u16, KeyboardType::Char(ch))),
    );
    last.push(Keyboard::new(2, 8, KeyboardType::Enter));
    res.push(last);

    res
}

enum CheckedResult {
    Ok,
    Failed,
}

#[cfg(feature = "tui")]
impl Wordle {
    pub fn run_app<B: Backend>(mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;
            if !event::poll(Duration::from_millis(250))? {
                continue;
            }
            match event::read()? {
                event::Event::Key(key_event) => {
                    if self.handle_key_event(key_event)?.is_break() {
                        break;
                    }
                }
                event::Event::Mouse(mouse_event) => self.handle_mouse_event(mouse_event),
                _ => {}
            }
        }

        Ok(())
    }

    pub fn check_states(&mut self) -> Option<(CheckResult, Word)> {
        if self.tui_states_mut().current_word.letters.len() == 5 {
            let checked_result = self.check_word();
            let returned_word = self.states.current_word.clone();
            self.tui_states_mut().next_states();
            return Some((checked_result, returned_word));
        }
        None
    }

    pub fn ui<B: Backend>(&mut self, frame: &mut Frame<'_, B>) {
        let block = Block::new()
            .title("Wordle Game")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::LightCyan))
            .title_style(Style::default().fg(Color::LightGreen));
        let inner = block.inner(frame.size());
        frame.render_widget(block, frame.size());
        let layout = Layout::default()
            .direction(ratatui::prelude::Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage(50),
                Constraint::Percentage(40),
                Constraint::Percentage(10),
            ])
            .split(inner);
        let table_row_layout = Layout::default()
            .direction(ratatui::prelude::Direction::Vertical)
            .constraints(vec![
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(layout[0]);

        for i in 0..self.states.current_try_times {
            let table_col_layout = Layout::default()
                .direction(ratatui::prelude::Direction::Horizontal)
                .constraints(vec![
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(0),
                ])
                .split(table_row_layout[i as usize]);
            for word in self.history_words.iter() {
                for (index, letter) in word.letters.iter().enumerate() {
                    frame.render_widget(
                        Paragraph::new(letter.0.to_string()).style(get_style(&letter.1)),
                        table_col_layout[index],
                    );
                }
            }
        }
        let table_col_layout = Layout::default()
            .direction(ratatui::prelude::Direction::Horizontal)
            .constraints(vec![
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(table_row_layout[self.states.current_try_times as usize]);
        for (index, letter) in self.states.current_word.letters.iter().enumerate() {
            frame.render_widget(
                Paragraph::new(letter.0.to_string()).style(get_style(&letter.1)),
                table_col_layout[index],
            );
        }
        frame.render_widget(Paragraph::new("test2"), layout[1]);
        if let Some(result) = self.states.current_checked_result {
            match result {
                CheckResult::InValid => frame.render_widget(
                    Paragraph::new("invalid word").style(Style::default().fg(Color::LightRed)),
                    layout[2],
                ),
                CheckResult::Success => frame.render_widget(
                    Paragraph::new("success").style(Style::default().fg(Color::LightGreen)),
                    layout[2],
                ),
                CheckResult::Wrong => frame.render_widget(
                    Paragraph::new("wrong word").style(Style::default().fg(Color::LightYellow)),
                    layout[2],
                ),
                CheckResult::Difficult => unreachable!(),
            }
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<ControlFlow<()>> {
        match key_event.code {
            KeyCode::Esc => {
                return Ok(ControlFlow::Break(()));
            }
            KeyCode::Char(ch) if ch.is_alphabetic() => {
                self.states.push(ch);
            }
            KeyCode::Backspace => {
                self.states.pop();
            }
            KeyCode::Enter => {
                if let Some((checked_result, _word)) = self.check_states() {
                    self.states.current_checked_result = Some(checked_result);
                    if CheckResult::Success == checked_result
                        || CheckResult::Wrong == checked_result
                    {
                        self.states.current_try_times += 1;
                    }
                }
            }
            _ => {}
        }
        Ok(ControlFlow::Continue(()))
    }
    fn handle_mouse_event(&mut self, _mouse_event: MouseEvent) {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiState {
    PopUp,
    EditLine(usize),
}

impl Default for UiState {
    fn default() -> Self {
        Self::EditLine(0)
    }
}

fn get_style(letter_state: &LetterState) -> Style {
    Style::default().fg(match letter_state {
        LetterState::G => Color::Green,
        LetterState::Y => Color::Yellow,
        LetterState::R => Color::Red,
        LetterState::X => Color::White,
    })
}
