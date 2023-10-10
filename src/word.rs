/// word 转换等
use crate::{
    error::{Result, WordError},
    state::{Letter, LetterState},
};

#[derive(Debug, Clone, Default)]
pub struct Word {
    pub letters: Vec<Letter>,
}

impl Word {
    pub const MAX_LENGTH: usize = 5;
    // 将输入的字符串转化成 `Word`类型
    pub fn parse(word: impl AsRef<str>) -> Result<Word> {
        let word = word.as_ref().trim().to_string();

        if word.len() != Self::MAX_LENGTH {
            return Err(WordError::InValidWord("word must be 5-letter".to_string()));
        }

        let letters = word.chars().map(Letter::new).collect();

        Ok(Word { letters })
    }

    pub fn set_state(&mut self, index: usize, state: LetterState) {
        let mut letter = self.letters[index];
        letter.set_state(state);
    }

    pub fn letters(&self) -> Vec<char> {
        self.letters.iter().map(|l| l.0).collect()
    }

    pub fn push(&mut self, ch: char) {
        if self.letters.len() < Self::MAX_LENGTH {
            self.letters.push(Letter::new(ch));
        }
    }

    pub fn pop(&mut self) {
        if !self.letters.is_empty() {
            self.letters.pop();
        }
    }
}
impl ToString for Word {
    fn to_string(&self) -> String {
        String::from_iter(self.letters.iter().map(|s| s.0))
    }
}

impl PartialEq for Word {
    fn eq(&self, other: &Self) -> bool {
        self.letters().eq(&other.letters())
    }
}

impl Eq for Word {}
