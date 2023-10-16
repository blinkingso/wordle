


use ratatui::{
    prelude::*,
    style::Color,
    style::Style,
};

use crate::{
    state::LetterState,
};

use super::widgets::{Theme, THEME_X};

pub enum KeyboardType {
    Char(char),
    Backspace,
    Enter,
}

pub struct Keyboard {
    x: u16,
    y: u16,
    pub size: u16,
    pub text: String,
    theme: Theme,
    state: LetterState,
    ktype: KeyboardType,
}

impl Keyboard {
    pub fn new(x: u16, y: u16, size: u16, ktype: KeyboardType) -> Self {
        let text = match ktype {
            KeyboardType::Char(ch) => ch.to_string(),
            KeyboardType::Backspace => "⇦".to_string(),
            KeyboardType::Enter => "Enter".to_string(),
        };
        Self {
            x,
            y,
            size,
            text,
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

    pub fn letter(&self) -> Option<char> {
        if let KeyboardType::Char(ch) = self.ktype {
            return Some(ch);
        }
        None
    }
}

pub fn init_keyboard() -> Vec<Vec<Keyboard>> {
    let mut res = vec![];
    res.push(
        ['Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P']
            .into_iter()
            .enumerate()
            .map(|(index, ch)| Keyboard::new(0, index as u16, 3, KeyboardType::Char(ch)))
            .collect(),
    );
    res.push(
        ['A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L']
            .into_iter()
            .enumerate()
            .map(|(index, ch)| Keyboard::new(1, index as u16, 3, KeyboardType::Char(ch)))
            .collect(),
    );
    let mut last = vec![Keyboard::new(2, 0, 6, KeyboardType::Backspace)];
    last.extend(
        ['Z', 'X', 'C', 'V', 'B', 'N', 'M']
            .into_iter()
            .enumerate()
            .map(|(index, ch)| Keyboard::new(2, (index + 1) as u16, 3, KeyboardType::Char(ch))),
    );
    last.push(Keyboard::new(2, 8, 6, KeyboardType::Enter));
    res.push(last);

    res
}

enum CheckedResult {
    Ok,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiState {
    Init,
    PopUp,
    EditLine(usize),
}

impl Default for UiState {
    fn default() -> Self {
        Self::EditLine(0)
    }
}

pub fn get_style(letter_state: &LetterState) -> Style {
    Style::default()
        .bg(match letter_state {
            LetterState::G => Color::Green,
            LetterState::Y => Color::Yellow,
            LetterState::R => Color::Red,
            LetterState::X => Color::White,
        })
        .fg(match letter_state {
            LetterState::G => Color::White,
            LetterState::Y => Color::White,
            LetterState::R => Color::White,
            LetterState::X => Color::Black,
        })
        .bold()
}

pub fn get_theme(letter_state: &LetterState) -> Theme {
    match letter_state {
        LetterState::G => Theme::new(Color::White, Color::Green),
        LetterState::Y => Theme::new(Color::White, Color::Yellow),
        LetterState::R => Theme::new(Color::White, Color::Red),
        LetterState::X => Theme::new(Color::White, Color::Black),
    }
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
