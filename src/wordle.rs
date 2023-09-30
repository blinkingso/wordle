use std::collections::HashSet;
use std::fs::File;
use std::path::PathBuf;

use colored::Colorize;
use rand::Rng;
use std::io::{BufRead, BufReader};

use crate::command::Opt;
use crate::error::Result;
use crate::state::{LetterState, Mode};
use crate::{state::Letter, word::Word};

// 游戏最大重试次数
const MAX_RETRY_TIMES: usize = 6;

///
/// 游戏应用, 保存游戏状态, 提供游戏入口和逻辑
/// 游戏模式:
/// ### 1. 测试模式, 在终端中通过标准库输入输出
/// ### 2. 终端TUI界面, 包含输入区, 键盘区
/// ### 3. GUI模式, 在gui应用中模拟线上wordle游戏
/// ### 4. 提供WebAssembly用于Web绘制用户界面之canvas
///
pub struct Wordle {
    // 输入过的字符及状态
    pub cached_letter_states: HashSet<Letter>,
    pub history_words: Vec<Word>,
    pub final_word: Word,
    pub acceptable_set: Vec<String>,
    pub final_set: Vec<String>,
    pub opt: Opt,
    pub mode: Mode,
    pub difficult_error_letters: HashSet<(usize, Letter)>,
}
impl Wordle {
    ///
    /// 检查输入的word是否在acceptable字典中
    ///
    fn is_acceptable_word(&self, word: &Word) -> bool {
        self.acceptable_set.binary_search(&word.word).is_ok()
    }

    ///
    /// 检查输入的`FINAL`单词是否在final_set中, 如果不在, 则询问是否继续
    ///
    fn is_final_word(&self, word: &Word) -> bool {
        self.final_set.binary_search(&word.word).is_ok()
    }

    ///
    /// 启动游戏逻辑
    ///
    pub fn run(mut self) -> Result<()> {
        // check final word
        if !self.is_acceptable_word(&self.final_word) || !self.is_final_word(&self.final_word) {
            return Err(crate::error::WordError::CustomError(format!(
                "word `{}` is not valid",
                self.final_word.word
            )));
        }
        let mut stdin = std::io::stdin().lock();
        let mut times = 0;
        loop {
            times += 1;
            if times > MAX_RETRY_TIMES {
                eprintln!("{} {}", "FAILED".red(), self.final_word.word.green());
                break;
            }
            println!(
                "{} {} {}",
                "Please enter the 5-letter(only 26 letters) word for your".green(),
                times,
                "attempt!".green()
            );
            let mut word = String::new();
            stdin.read_line(&mut word)?;
            // 去除换行符
            word.remove(word.len() - 1);
            if let Ok(mut w) = Word::parse(word.as_str()) {
                // word 在final set 中并且在acceptable set中， 判断word是否正确， 以及各个位置的字母是否符合要求
                match self.check_word(&mut w) {
                    CheckResult::InValid => {
                        // 不消耗次数
                        times -= 1;
                        eprintln!("INVALID");
                        continue;
                    }
                    CheckResult::Success => {
                        w.print(&self);
                        println!("{} {}", "CORRECT".green(), times);
                        break;
                    }
                    CheckResult::Wrong => {
                        w.print(&self);
                    }
                    CheckResult::Difficult => {
                        if self.opt.difficult {
                            w.print(&self);
                            times -= 1;
                            continue;
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

    fn resolve_difficult(&mut self, word: &Word) -> bool {
        // 困难模式下输入的必须包含y和g的字母， 且g的字母位置必须正确
        if self.opt.difficult {
            self.difficult_error_letters.clear();
            // 找出y和g的字母及索引位置
            let meaningful_letters = self
                .history_words
                .iter()
                .flat_map(|word| {
                    word.letters
                        .iter()
                        .enumerate()
                        .filter_map(|(index, letter)| {
                            if letter.1 == LetterState::G || letter.1 == LetterState::Y {
                                return Some((index, letter));
                            }
                            None
                        })
                })
                .collect::<Vec<_>>();

            let mut difficult_error = HashSet::new();
            let green_letters: HashSet<_> = meaningful_letters
                .iter()
                .filter(|(_, letter)| letter.1 == LetterState::G)
                .collect();
            let yellow_letters: HashSet<_> = meaningful_letters
                .iter()
                .filter(|(_, letter)| letter.1 == LetterState::Y)
                .map(|(_, letter)| letter)
                .collect();
            // 绿色位置不正确时， 将不正确的字母和位置记录到错误信息中
            for (index, gl) in green_letters.into_iter() {
                if word.letters[*index].ne(gl) {
                    difficult_error.insert((*index, **gl));
                }
            }
            // 黄色的字母仅需判断是否在本次猜测的字母数组中， 如果不存在，则记录到错误信息中
            for yl in yellow_letters {
                if !word.letters.contains(yl) {
                    difficult_error.insert((0, **yl));
                }
            }
            if !difficult_error.is_empty() {
                self.difficult_error_letters.extend(difficult_error);
                return false;
            }
        }
        true
    }

    /// 检查结果
    fn check_word(&mut self, word: &mut Word) -> CheckResult {
        if !self.is_acceptable_word(word) || !self.is_final_word(word) {
            return CheckResult::InValid;
        }

        let final_word = self.final_word.clone();

        let mut processed_letters: Vec<Letter> = vec![];
        for (index, letter) in word.letters.iter_mut().enumerate() {
            let current_char = final_word.letters[index];
            if current_char.0 == letter.0 {
                letter.set_state(LetterState::G);
            } else {
                // 位置不正确，且未处理的字符标记为yellow
                if final_word.word.contains(&letter.0.to_string())
                    && !processed_letters.contains(letter)
                {
                    // yellow or red
                    letter.set_state(LetterState::Y);
                } else {
                    letter.set_state(LetterState::R);
                }
            }
            processed_letters.push(*letter);
        }
        processed_letters.sort();
        let cached_letters: HashSet<Letter> = HashSet::from_iter(processed_letters);
        self.cached_letter_states.extend(cached_letters);
        self.history_words.push(word.clone());

        // 困难模式下输入的必须包含y和g的字母， 且g的字母位置必须正确
        if !self.resolve_difficult(word) {
            return CheckResult::Difficult;
        }

        if final_word.eq(word) {
            return CheckResult::Success;
        }
        CheckResult::Wrong
    }

    pub fn read_input_file(path: &PathBuf) -> Result<Vec<String>> {
        let file = File::open(path)?;
        let buf_reader = BufReader::new(file);
        let mut set: Vec<String> = buf_reader
            .lines()
            .filter_map(|line| {
                let mut line = line.unwrap();
                if line.contains('\n') {
                    // 去除换行符
                    line.remove(line.len() - 1);
                }
                if line.len() != 5 {
                    return None;
                }
                Some(line)
            })
            .collect();
        if set.is_empty() {
            return Err(crate::error::WordError::CustomError(
                "input file is empty or invalid".to_string(),
            ));
        }
        set.sort();
        Ok(set)
    }

    ///
    /// 重新设置游戏状态， 当继续开始新的游戏时执行当前操作。
    ///
    pub fn reset(&mut self) -> Result<()> {
        self.cached_letter_states.clear();
        self.history_words.clear();
        self.difficult_error_letters.clear();
        if self.opt.random {
            let mut rng = rand::thread_rng();
            let index = rng.gen_range(0..self.final_set.len());
            let word = Word::parse(self.final_set[index].as_str())?;
            if self.is_acceptable_word(&word) {
                self.final_word = word;
            } else {
                self.reset()?;
            }
        }
        if self.opt.word.is_none() && !self.opt.random {
            let mut stdin = std::io::stdin().lock();
            loop {
                println!("{}", "please enter the specified final word: ".blue());
                let mut w = String::new();
                stdin.read_line(&mut w)?;
                let word = Word::parse(&w)?;
                if self.is_acceptable_word(&word) && self.is_final_word(&word) {
                    drop(stdin);
                    break;
                } else {
                    eprintln!("{}", "INVALID final word.".red());
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
enum CheckResult {
    InValid,
    Success,
    Wrong,
    Difficult,
}
