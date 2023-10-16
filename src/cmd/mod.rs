use crate::{
    error::Result,
    state::{Letter, LetterState, Mode},
    word::Word,
    wordle::{CheckResult, Wordle, MAX_RETRY_TIMES},
};
use colored::Colorize;
use std::io::{self, BufRead};

/// 键盘上26个字母
pub const KEYBOARD_KEYS: [char; 26] = [
    'Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P', 'A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L',
    'Z', 'X', 'C', 'V', 'B', 'N', 'M',
];

impl Wordle {
    pub fn print(&self) {
        match self.mode {
            Mode::Test => {
                // SSSSS AAAAAAAAAAAAAAAAAAAAAAAAAA
                let guessed = self.states.current_word.to_string();

                let keyboards = KEYBOARD_KEYS
                    .map(|key| {
                        let cached_letter = self
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
                for word in self.history_words.iter() {
                    for letter in word.get_letters().iter() {
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
                if self.opt.difficult && !self.difficult_error_letters.is_empty() {
                    let mut has_green = false;
                    self.difficult_error_letters
                        .iter()
                        .filter(|green| green.1 .1 == LetterState::G)
                        .for_each(|(index, letter)| {
                            println!("{}th letter must be {}", *index + 1, letter.0);
                            has_green = true;
                        });
                    if !has_green {
                        let letters = self
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

    pub fn run(mut self) -> Result<()> {
        // check final word
        let mut stdin = io::stdin().lock();
        // 随机模式中，添加--day/-d 用于指定开始时的局数，-d 5表示从第5局开始， 跳过前4局
        if self.opt.random && self.opt.day.is_some() {
            self.states.current_try_times = self.opt.day.unwrap();
        }

        loop {
            self.states.current_try_times += 1;
            if self.states.current_try_times > MAX_RETRY_TIMES {
                eprintln!("{} {}", "FAILED".red(), self.final_word.to_string().green());
                break;
            }
            println!(
                "{} {} {}",
                "Please enter the 5-letter(only 26 letters) word for your".green(),
                self.states.current_try_times,
                "attempt!".green()
            );
            let mut word = String::new();
            stdin.read_line(&mut word)?;
            if let Ok(w) = Word::parse(word.trim()) {
                self.states.current_word = w;
                // word 在final set 中并且在acceptable set中， 判断word是否正确， 以及各个位置的字母是否符合要求
                let check_result = self.check_word();

                {
                    match check_result {
                        CheckResult::InValid => {
                            // 不消耗次数
                            self.states.current_try_times -= 1;
                            eprintln!("INVALID");
                            continue;
                        }
                        CheckResult::Success => {
                            self.print();
                            println!("{} {}", "CORRECT".green(), self.states.current_try_times);
                            break;
                        }
                        CheckResult::Wrong => {
                            self.print();
                        }
                        CheckResult::Difficult => {
                            if self.opt.difficult {
                                self.print();
                                self.states.current_try_times -= 1;
                                continue;
                            }
                        }
                    }
                }
            } else {
                eprintln!("INVALID");
            }
        }
        if self.opt.word.is_none() {
            // 非指定单词模式下， 询问是否开始下一句
            let mut w = String::new();
            {
                println!("new game? type `y` to continue, enter any other letters to finish!");
                stdin.read_line(&mut w)?;
                // release stdin lock.
                drop(stdin);
            }
            if w.contains('y') {
                self.reset()?;
                self.run()?;
            }
        }

        Ok(())
    }
}
