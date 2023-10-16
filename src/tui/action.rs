use crossterm::event::KeyCode;

use crate::wordle::{Wordle, MAX_RETRY_TIMES};

use super::{event::Event, ui::UiState};

pub enum Action {
    Init,
    Tick,
    Render,
    InputChar(char),
    RemoveChar,
    CheckWord,
    Error,
    Quit,
    None,
}

pub fn get_action(_wordle: &Wordle, event: Event) -> Action {
    match event {
        Event::Init => Action::Init,
        Event::Quit => Action::Quit,
        Event::Error => Action::None,
        Event::Tick => Action::Tick,
        Event::Render => Action::Render,
        Event::Key(key) => match key.code {
            KeyCode::Char(ch) if ch.is_alphabetic() => Action::InputChar(ch),
            KeyCode::Backspace => Action::RemoveChar,
            KeyCode::Enter => Action::CheckWord,
            _ => Action::None,
        },
        Event::Mouse(_) => Action::None,
    }
}

pub fn update(wordle: &mut Wordle, action: Action) {
    match action {
        Action::Init => {
            wordle.ui_state = UiState::Init;
        }
        Action::Tick => {
            // log...
        }
        Action::Error => {}
        Action::Render => match wordle.ui_state {
            UiState::Init => todo!(),
            UiState::PopUp => todo!(),
            UiState::EditLine(_) => todo!(),
        },
        Action::Quit => wordle.states.current_try_times = MAX_RETRY_TIMES,
        Action::None => {}
        Action::RemoveChar => match wordle.ui_state {
            UiState::Init => wordle.final_word.pop(),
            UiState::EditLine(_) => wordle.states.current_word.pop(),
            _ => {}
        },
        Action::CheckWord => match wordle.ui_state {
            UiState::Init => {
                if wordle.is_final_word_valid() {
                    wordle.ui_state = UiState::EditLine(0);
                }
            }
            UiState::EditLine(_) => {
                if wordle.states.current_word.is_full() {
                    let check_result = wordle.check_word();
                    wordle.states.current_checked_result = Some(check_result);
                }
            }
            _ => {}
        },
        Action::InputChar(ch) => match wordle.ui_state {
            UiState::Init => {
                if !wordle.final_word.is_full() {
                    wordle.final_word.push(ch);
                }
            }
            UiState::EditLine(_) => {
                if !wordle.states.current_word.is_full() {
                    wordle.states.current_word.push(ch);
                }
            }
            _ => {}
        },
    }
}
