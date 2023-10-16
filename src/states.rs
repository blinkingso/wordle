use crate::{state::Letter, word::Word, wordle::CheckResult};
use derive_builder::Builder;

#[derive(Debug, Default, Clone, Builder)]
pub struct States {
    pub current_word: Word,
    pub current_word_checked: bool,
    pub current_try_times: u32,
    pub current_checked_result: Option<CheckResult>,
    pub should_quit: bool,
}

impl States {
    pub fn pop(&mut self) {
        if self.current_word_checked || self.current_word.letters.is_empty() {
            return;
        }
        self.current_word.letters.pop();
    }

    pub fn push(&mut self, ch: char) {
        if !self.current_word_checked && self.current_word.letters.len() != 5 {
            self.current_word.letters.push(Letter::new(ch));
        }
    }

    pub fn next_state(&mut self) {
        *self = States {
            current_try_times: self.current_try_times + 1,
            current_checked_result: self.current_checked_result,
            ..Default::default()
        }
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}
