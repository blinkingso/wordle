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
    highlight: Color,
}

/// 默认颜色
pub const THEME_X: Theme = Theme {
    text: Color::DarkGray,
    background: Color::Rgb(164, 174, 196),
    highlight: Color::Rgb(196, 203, 221),
};

pub const THEME_R: Theme = Theme {
    text: Color::White,
    background: Color::LightRed,
    highlight: Color::Rgb(196, 203, 221),
};

pub const THEME_Y: Theme = Theme {
    text: Color::White,
    background: Color::LightYellow,
    highlight: Color::Rgb(196, 203, 221),
};

pub const THEME_G: Theme = Theme {
    text: Color::White,
    background: Color::LightGreen,
    highlight: Color::Rgb(196, 203, 221),
};

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

        buf.set_line(
            area.x + (area.width.saturating_sub(self.label.width() as u16)) / 2,
            area.y + (area.height.saturating_sub(1)) / 2,
            &self.label,
            area.width,
        );
    }
}
