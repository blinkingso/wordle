use crossterm::event::KeyCode;

use crate::error::Result;
use crate::word::Word;
use crate::wordle::{Wordle, MAX_RETRY_TIMES};

use super::ui::MainState;
use super::{event::Event, ui::UiState};

#[derive(Debug, Clone)]
pub enum Action {
    Init,
    Tick,
    Render,
    InputChar(char),
    PopUp,
    RemoveChar,
    // 回车键
    Enter,
    // 进入MainState::Main
    EnterMain,
    ReNew,
    Error,
    Quit,
    None,
}

pub fn get_action(wordle: &Wordle, event: Event) -> Action {
    match event {
        Event::Init if wordle.final_word.is_empty() => Action::Init,
        Event::Quit => Action::Quit,
        Event::Tick => Action::Tick,
        Event::Render => Action::Render,
        Event::Key(key) => match key.code {
            KeyCode::Char(ch) if ch.is_alphabetic() => Action::InputChar(ch),
            KeyCode::Backspace => Action::RemoveChar,
            KeyCode::Enter => match wordle.ui_state {
                UiState::Init => Action::Init,
                UiState::Main(main_state) => match main_state {
                    MainState::Main => {
                        if !wordle.is_game_over() {
                            return Action::Enter;
                        }
                        Action::PopUp
                    }
                    MainState::Difficult => Action::EnterMain,
                    MainState::GameOver => Action::ReNew,
                },
            },
            KeyCode::Esc => Action::Quit,
            _ => Action::None,
        },
        _ => Action::None,
    }
}

pub fn update(wordle: &mut Wordle, action: Action) -> Result<()> {
    match action {
        Action::Init => {
            if wordle.final_word.is_full() {
                if wordle.is_final_word_valid() {
                    wordle.ui_state = UiState::Main(MainState::Main);
                } else {
                    wordle.final_word = Word::default();
                }
            }
        }
        Action::Quit => {
            wordle.states.current_try_times = MAX_RETRY_TIMES;
            wordle.game_over();
            wordle.exit = true;
        }
        Action::RemoveChar => match wordle.ui_state {
            UiState::Init => wordle.final_word.pop(),
            UiState::Main(MainState::Main) => wordle.states.current_word.pop(),
            _ => {}
        },
        // 用户按了enter键时更新操作
        Action::Enter => match wordle.ui_state {
            // 校验输入单词是否满足final word
            UiState::Init => {
                if wordle.is_final_word_valid() {
                    wordle.ui_state = UiState::Main(MainState::Main);
                }
            }
            UiState::Main(main_state) => match main_state {
                // 校验输入的单词是否和final word一致
                MainState::Main => {
                    if wordle.states.current_word.is_full() {
                        if !wordle.is_game_over() {
                            let check_result = wordle.check_word();
                            wordle.states.current_checked_result = Some(check_result);
                            match check_result {
                                crate::wordle::CheckResult::Success => wordle.game_over = true,
                                crate::wordle::CheckResult::Wrong => {
                                    if wordle.states.current_try_times == MAX_RETRY_TIMES - 1 {
                                        wordle.game_over = true;
                                    } else {
                                        wordle.states.next_state();
                                    }
                                }
                                crate::wordle::CheckResult::Difficult => {
                                    wordle.ui_state = UiState::Main(MainState::Difficult);
                                }
                                crate::wordle::CheckResult::InValid => {}
                            }
                        } else {
                            wordle.ui_state = UiState::Main(MainState::GameOver);
                        }
                    }
                }
                MainState::Difficult => {
                    wordle.ui_state = UiState::Main(MainState::Main);
                }
                MainState::GameOver => {
                    wordle.ui_state = UiState::Init;
                }
            },
        },
        Action::InputChar(ch) => match wordle.ui_state {
            UiState::Init => {
                wordle.final_word.push(ch);
            }
            UiState::Main(MainState::Main) => {
                wordle.states.current_word.push(ch);
            }
            _ => {}
        },
        Action::ReNew => {
            wordle.reset()?;
            if wordle.final_word.is_empty() {
                wordle.ui_state = UiState::Init;
            } else {
                wordle.ui_state = UiState::Main(MainState::Main);
            }
        }
        Action::PopUp => {
            wordle.ui_state = UiState::Main(MainState::GameOver);
        }
        Action::EnterMain => {
            wordle.ui_state = UiState::Main(MainState::Main);
        }
        _ => {}
    }
    Ok(())
}
