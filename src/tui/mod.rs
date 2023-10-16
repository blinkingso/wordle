//! tui 模式
use std::{ops::ControlFlow};

use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use ratatui::{
    prelude::*,
    style::Color,
    style::Style,
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::{
    error::Result,
    word::Word,
    wordle::{CheckResult, Wordle, MAX_RETRY_TIMES},
};


use widgets::Button;



use self::ui::{centered_rect, get_style, get_theme, Keyboard, UiState};

pub mod action;
pub mod event;
pub mod tui;
pub mod ui;
pub mod widgets;

impl Wordle {
    // pub async fn run_app(mut self) -> Result<()> {
    //     let keyboards = init_keyboard();
    //     loop {
    //         terminal.draw(|f| self.ui(f, &keyboards))?;
    //         if !event::poll(Duration::from_millis(250))? {
    //             continue;
    //         }
    //         match event::read()? {
    //             event::Event::Key(key_event) => {
    //                 if self.handle_key_event(key_event)?.is_break() {
    //                     break;
    //                 }
    //             }
    //             event::Event::Mouse(mouse_event) => self.handle_mouse_event(mouse_event),
    //             _ => {}
    //         }
    //     }

    //     Ok(())
    // }

    pub fn check_states(&mut self) -> Option<CheckResult> {
        if self.states.current_word.is_full() {
            Some(self.check_word())
        } else {
            None
        }
    }

    pub fn ui<B: Backend>(&mut self, frame: &mut Frame<'_, B>, keyboards: &Vec<Vec<Keyboard>>) {
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
                Constraint::Min(18),
                Constraint::Min(9),
                Constraint::Length(3),
            ])
            .split(inner);
        let gussing_area_block = Block::new()
            .title("Gussing Area")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::LightCyan))
            .title_style(Style::default().fg(Color::Red));
        let gussing_inner = gussing_area_block.inner(layout[0]);
        frame.render_widget(gussing_area_block, layout[0]);
        let table_row_layout = Layout::default()
            .direction(ratatui::prelude::Direction::Vertical)
            .constraints(vec![
                Constraint::Length(2),
                Constraint::Length(2),
                Constraint::Length(2),
                Constraint::Length(2),
                Constraint::Length(2),
                Constraint::Length(2),
                Constraint::Min(0),
            ])
            .split(gussing_inner);

        let default_word = Word::empty();
        for idx in self.history_words.len()..MAX_RETRY_TIMES as usize {
            self.render_buttons(frame, table_row_layout[idx], &default_word);
        }
        // 历史记录渲染
        for (idx, word) in self.history_words.iter().enumerate() {
            self.render_buttons(frame, table_row_layout[idx], word);
        }

        // 当前行渲染
        self.render_buttons(
            frame,
            table_row_layout[self.states.current_try_times as usize],
            &self.states.current_word,
        );

        // status render
        let status_bar = "Status: ".light_green();
        let mut status = vec![status_bar];
        let current_times = self.states.current_try_times;
        let current_word = self.states.current_word.to_string();
        let mut s = String::new();
        if let Some(result) = self.states.current_checked_result {
            match result {
                CheckResult::InValid => {
                    s = format!("Invalid word: `{}`", current_word);
                }
                CheckResult::Success => {
                    status.push("Success".light_green());
                    self.ui_state = UiState::PopUp;
                }
                CheckResult::Wrong => {
                    s = format!("Wrong word with `{}` times tests.", current_times);
                    if self.is_game_over() {
                        self.ui_state = UiState::PopUp;
                    }
                }
                CheckResult::Difficult => unreachable!(),
            }
        } else {
            s = "Nothing to do now!".to_string();
        }
        status.push(s.light_yellow());
        let footer = Line::from(status);
        frame.render_widget(Paragraph::new(footer), layout[2]);

        // render keyboards
        self.render_keyboards(frame, keyboards, layout[1]);

        if self.ui_state == UiState::PopUp {
            let popup_block = Block::new()
                .title("Continue Or Quit")
                .title_style(Style::default().fg(Color::Yellow))
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Black))
                .style(
                    Style::default()
                        .bg(Color::Rgb(135, 206, 235))
                        .fg(Color::Gray),
                );
            let paragraph = Paragraph::new("Press Esc to quit or Enter to continue")
                .alignment(Alignment::Center)
                .block(popup_block);
            let area = centered_rect(40, 10, frame.size());
            frame.render_widget(Clear, area);
            frame.render_widget(paragraph, area);
        }
    }

    fn render_keyboards<B: Backend>(
        &self,
        frame: &mut Frame<'_, B>,
        keyboards: &Vec<Vec<Keyboard>>,
        area: Rect,
    ) {
        let block = Block::new()
            .title("Keyboards Area")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::LightCyan));
        let inner = block.inner(area);
        frame.render_widget(block, area);
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(inner);
        for i in 0..3 {
            let keys = &keyboards[i];
            let mut constraints: Vec<Constraint> = keys
                .iter()
                .map(|key| Constraint::Length(key.size))
                .collect();
            constraints.push(Constraint::Min(0));
            let key_layout = Layout::new()
                .direction(Direction::Horizontal)
                .constraints(constraints)
                .split(layout[i]);
            for (idx, key) in keys.into_iter().enumerate() {
                let state = if let Some(ch) = key.letter() {
                    self.cached_letter_states.iter().find(|l| l.0 == ch)
                } else {
                    None
                };
                if state.is_some() {
                    let style = get_style(&state.unwrap().1);
                    frame.render_widget(
                        Paragraph::new(format!("{}i", key.text.as_str())).style(style),
                        key_layout[idx],
                    );
                } else {
                    frame.render_widget(Paragraph::new(key.text.as_str()), key_layout[idx]);
                }
            }
        }
    }

    fn render_buttons<B: Backend>(&self, frame: &mut Frame<'_, B>, area: Rect, word: &Word) {
        let layout = Layout::default()
            .constraints(vec![Constraint::Percentage(80), Constraint::Percentage(20)])
            .direction(Direction::Vertical)
            .split(area);
        let table_col_layout = Layout::default()
            .direction(ratatui::prelude::Direction::Horizontal)
            .constraints(vec![
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Min(0),
            ])
            .split(layout[0]);
        for (index, letter) in word.letters.iter().enumerate() {
            let block_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![Constraint::Percentage(80), Constraint::Percentage(20)])
                .split(table_col_layout[index]);
            frame.render_widget(
                Button::new(letter.0.to_string()).theme(get_theme(&letter.1)),
                block_layout[0],
            );
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
            KeyCode::Enter => match self.ui_state {
                UiState::PopUp => {
                    self.ui_state = UiState::EditLine(0);
                    self.reset()?;
                }
                UiState::EditLine(_) => {
                    if let Some(checked_result) = self.check_states() {
                        self.states.current_checked_result = Some(checked_result);
                        if checked_result == CheckResult::Wrong {
                            self.states.next_state();
                        }
                    }
                }
                UiState::Init => {
                    //将输入的字符保存到final_word中即指定final word模式
                }
            },
            _ => {}
        }
        Ok(ControlFlow::Continue(()))
    }
    fn handle_mouse_event(&mut self, _mouse_event: MouseEvent) {}
}
