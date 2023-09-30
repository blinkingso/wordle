use colored::Colorize;

/// word 转换等
use crate::{
    buildin_words::KEYBOARD_KEYS,
    error::{Result, WordError},
    state::{Letter, LetterState, Mode},
    wordle::Wordle,
};

#[derive(Debug, Clone)]
pub struct Word {
    pub word: String,
    pub letters: Vec<Letter>,
}

impl Word {
    // 将输入的字符串转化成 `Word`类型
    pub fn parse(word: impl AsRef<str>) -> Result<Word> {
        let mut word = word.as_ref().to_string();
        if word.contains('\n') {
            word.remove(word.len() - 1);
        }

        if word.len() != 5 {
            return Err(WordError::InValidWord("word must be 5-letter".to_string()));
        }

        let letters = word.chars().map(Letter::new).collect();

        Ok(Word { word, letters })
    }

    pub fn set_state(&mut self, index: usize, state: LetterState) {
        let mut letter = self.letters[index];
        letter.set_state(state);
    }

    pub fn print(&self, wordle: &Wordle) {
        match wordle.mode {
            Mode::Test => {
                // SSSSS AAAAAAAAAAAAAAAAAAAAAAAAAA
                let guessed = self
                    .letters
                    .iter()
                    .map(|letter| format!("{:?}", letter.1))
                    .collect::<Vec<_>>()
                    .join("");
                let keyboards = KEYBOARD_KEYS
                    .map(|key| {
                        let cached_letter = wordle
                            .cached_letter_states
                            .iter()
                            .find(|l| l.0.eq_ignore_ascii_case(&key));
                        let current_letter = if let Some(l) = cached_letter {
                            *l
                        } else {
                            Letter::new(key)
                        };
                        format!("{:?}", current_letter.1)
                    })
                    .join("");
                println!("{} {}", guessed, keyboards);
            }
            Mode::Interactive => {
                // 输出历史单词
                for word in wordle.history_words.iter() {
                    for letter in word.letters.iter() {
                        let formated_string = match letter.1 {
                            crate::state::LetterState::G => letter.0.to_string().green(),
                            crate::state::LetterState::Y => letter.0.to_string().yellow(),
                            crate::state::LetterState::R => letter.0.to_string().red(),
                            crate::state::LetterState::X => letter.0.to_string().black(),
                        };
                        print!("{}", formated_string);
                    }
                    println!();
                }
                if wordle.opt.difficult && !wordle.difficult_error_letters.is_empty() {
                    let mut has_green = false;
                    wordle
                        .difficult_error_letters
                        .iter()
                        .filter(|green| green.1 .1 == LetterState::G)
                        .for_each(|(index, letter)| {
                            println!("{}th letter must be {}", *index + 1, letter.0);
                            has_green = true;
                        });
                    if !has_green {
                        let letters = wordle
                            .difficult_error_letters
                            .iter()
                            .filter(|green| green.1 .1 == LetterState::Y)
                            .map(|(_, letter)| letter.0.to_string())
                            .collect::<Vec<_>>()
                            .join(",");
                        if !letters.is_empty() {
                            println!("the word must contain {}", letters);
                        }
                    }
                }
            }
        }
    }
}

impl PartialEq for Word {
    fn eq(&self, other: &Self) -> bool {
        self.word == other.word
    }
}
