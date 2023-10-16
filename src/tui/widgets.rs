use ratatui::{prelude::*, widgets::*};

use crate::state::LetterState;

pub struct Button<'a> {
    label: Line<'a>,
    state: LetterState,
    theme: Theme,
}

impl Button<'_> {
    pub fn state(mut self, state: LetterState) -> Self {
        self.state = state;
        self
    }

    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }
}

#[derive(Debug, Clone)]
pub struct Theme {
    text: Color,
    background: Color,
    _highlight: Color,
}

/// 默认颜色
pub const THEME_X: Theme = Theme {
    text: Color::DarkGray,
    background: Color::Rgb(164, 174, 196),
    _highlight: Color::Rgb(196, 203, 221),
};

pub const THEME_R: Theme = Theme {
    text: Color::White,
    background: Color::LightRed,
    _highlight: Color::Rgb(196, 203, 221),
};

pub const THEME_Y: Theme = Theme {
    text: Color::White,
    background: Color::LightYellow,
    _highlight: Color::Rgb(196, 203, 221),
};

pub const THEME_G: Theme = Theme {
    text: Color::White,
    background: Color::LightGreen,
    _highlight: Color::Rgb(196, 203, 221),
};

impl Theme {
    pub fn new(fg: Color, bg: Color) -> Self {
        Theme {
            background: bg,
            text: fg,
            _highlight: Color::Black,
        }
    }
}

impl<'a> Button<'a> {
    pub fn new<T: Into<Line<'a>>>(label: T) -> Self {
        Button {
            label: label.into(),
            state: LetterState::X,
            theme: THEME_X,
        }
    }
}

impl<'a> Widget for Button<'a> {
    fn render(self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        buf.set_style(
            area,
            Style::default()
                .bg(self.theme.background)
                .fg(self.theme.text),
        );

        let width = (area.width as f32 * 0.8) as u16;
        let height = (area.height as f32 * 0.8) as u16;
        buf.set_line(
            area.x + (width.saturating_sub(self.label.width() as u16)) / 2,
            area.y + (height.saturating_sub(1)) / 2,
            &self.label,
            area.width,
        );
    }
}

pub enum KeyboardType {
    Char(char),
    Backspace,
    Enter,
}

pub struct Keyboard {
    _x: u16,
    _y: u16,
    pub size: u16,
    pub text: String,
    theme: Theme,
    state: LetterState,
    ktype: KeyboardType,
}

impl Keyboard {
    pub fn new(_x: u16, _y: u16, size: u16, ktype: KeyboardType) -> Self {
        let text = match ktype {
            KeyboardType::Char(ch) => ch.to_string(),
            KeyboardType::Backspace => "⇦".to_string(),
            KeyboardType::Enter => "Enter".to_string(),
        };
        Self {
            _x,
            _y,
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
