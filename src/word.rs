/// word 转换等
use crate::{
    error::{Result, WordError},
    state::{Letter, LetterState},
};

#[derive(Debug, Clone, Default)]
pub struct Word {
    letters: Vec<Letter>,
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

    pub fn get_letters(&self) -> &[Letter] {
        &self.letters
    }

    pub fn get_mut_letters(&mut self) -> &mut Vec<Letter> {
        &mut self.letters
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

    pub fn is_full(&self) -> bool {
        self.letters.len() == Self::MAX_LENGTH
    }

    pub fn is_empty(&self) -> bool {
        self.letters.is_empty()
    }

    pub fn whitespace_word_for_render() -> Self {
        Word {
            letters: vec![
                Letter::new(' '),
                Letter::new(' '),
                Letter::new(' '),
                Letter::new(' '),
                Letter::new(' '),
            ],
        }
    }

    pub fn diff(&mut self, final_word: &Word) {
        let mut final_word = final_word.to_string().as_bytes().to_owned();
        let input = self.to_string();
        let input = input.as_bytes();
        // set green
        for (pos, &letter) in input.iter().enumerate() {
            if final_word[pos] == letter {
                final_word[pos] = 0; // letters only match once.
                self.letters[pos].set_state(LetterState::G);
            }
        }

        // set yellow
        for (pos, &letter) in input.iter().enumerate() {
            if self.letters[pos].1 != LetterState::X {
                continue;
            }

            if let Some(j) = final_word.iter().position(|&x| x == letter) {
                final_word[j] = 0;
                self.letters[pos].set_state(LetterState::Y);
            }
        }

        // set red
        for l in self.letters.iter_mut() {
            if l.1 == LetterState::X {
                l.set_state(LetterState::R);
            }
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
