use std::collections::HashSet;
use std::fs::File;
use std::path::PathBuf;

use derive_builder::Builder;
use rand::{Rng, SeedableRng};
use std::io::{BufRead, BufReader};
use tokio::sync::mpsc::UnboundedSender;

use crate::command::Opt;
use crate::error::Result;
use crate::state::{LetterState, Mode};
use crate::states::States;
use crate::tui::action::Action;
use crate::tui::ui::UiState;
use crate::{state::Letter, word::Word};

// 游戏最大重试次数
pub const MAX_RETRY_TIMES: u32 = 6;

///
/// 游戏应用, 保存游戏状态, 提供游戏入口和逻辑
/// 游戏模式:
/// ### 1. 测试模式, 在终端中通过标准库输入输出
/// ### 2. 终端TUI界面, 包含输入区, 键盘区
/// ### 3. GUI模式, 在gui应用中模拟线上wordle游戏
/// ### 4. 提供WebAssembly用于Web绘制用户界面之canvas
///
#[derive(Debug, Default, Builder)]
pub struct Wordle {
    // 输入过的字符及状态
    pub cached_letter_states: HashSet<Letter>,
    // 历史词汇
    pub history_words: Vec<Word>,
    // 当前游戏的猜测词汇
    pub final_word: Word,
    // 用户输入词库
    pub acceptable_set: Vec<String>,
    // 答案生成词库
    pub final_set: Vec<String>,
    // 命令行参数列表
    #[builder(setter(skip))]
    pub opt: Opt,
    // 游戏模式: 交互模式, 测试模式, tui模式, gui模式
    pub mode: Mode,
    // 困难模式下猜测错误的字符
    pub difficult_error_letters: HashSet<(usize, Letter)>,
    // 游戏局数成功次数等统计
    pub statistics: WordleStatistic,
    // 当前游戏状态, 猜测的词汇, 猜测次数等.
    pub states: States,
    #[cfg(feature = "tui")]
    pub ui_state: UiState,
    #[cfg(feature = "tui")]
    pub sender: Option<UnboundedSender<Action>>,
}

impl Wordle {
    ///
    /// 检查输入的word是否在acceptable字典中
    ///
    pub fn is_acceptable_word(&self) -> bool {
        self.acceptable_set
            .binary_search(&self.states.current_word.to_string())
            .is_ok()
    }

    pub fn is_game_over(&self) -> bool {
        self.states.current_try_times >= MAX_RETRY_TIMES
    }

    pub fn is_final_word_valid(&self) -> bool {
        self.acceptable_set
            .binary_search(&self.final_word.to_string())
            .is_ok()
            && self
                .final_set
                .binary_search(&self.final_word.to_string())
                .is_ok()
    }

    ///
    /// 检查输入的`FINAL`单词是否在final_set中, 如果不在, 则询问是否继续
    ///
    pub fn is_final_word(&self) -> bool {
        self.final_set
            .binary_search(&self.states.current_word.to_string())
            .is_ok()
    }

    fn resolve_difficult(&mut self) -> bool {
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
                if self.states.current_word.letters[*index].ne(gl) {
                    difficult_error.insert((*index, **gl));
                }
            }
            // 黄色的字母仅需判断是否在本次猜测的字母数组中， 如果不存在，则记录到错误信息中
            for yl in yellow_letters {
                if !self.states.current_word.letters.contains(yl) {
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
    pub fn check_word(&mut self) -> CheckResult {
        if !self.is_acceptable_word() || !self.is_final_word() {
            return CheckResult::InValid;
        }

        let final_word = self.final_word.clone();

        let mut processed_letters: Vec<Letter> = vec![];
        for (index, letter) in self.states.current_word.letters.iter_mut().enumerate() {
            let current_char = final_word.letters[index];
            if current_char.0 == letter.0 {
                letter.set_state(LetterState::G);
            } else {
                // 位置不正确，且未处理的字符标记为yellow
                if final_word.letters().contains(&letter.0) && !processed_letters.contains(letter) {
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
        self.history_words.push(self.states.current_word.clone());

        // 困难模式下输入的必须包含y和g的字母， 且g的字母位置必须正确
        if !self.resolve_difficult() {
            return CheckResult::Difficult;
        }

        if final_word.eq(&self.states.current_word) {
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
        self.states.reset();
        if self.opt.random {
            let seed = if let Some(seed) = self.opt.seed {
                seed
            } else {
                2048
            };
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            let index = rng.gen_range(0..self.final_set.len());
            let word = Word::parse(self.final_set[index].as_str())?;
            if self.is_final_word_valid() {
                self.final_word = word;
            } else {
                self.reset()?;
            }
        }
        // tui模式下, 随机生成答案
        #[cfg(feature = "tui")]
        {
            let seed = if let Some(seed) = self.opt.seed {
                seed
            } else {
                2048
            };
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            let index = rng.gen_range(0..self.final_set.len());
            let word = Word::parse(self.final_set[index].as_str())?;
            if self.is_final_word_valid() {
                self.final_word = word;
            } else {
                self.reset()?;
            }
        }
        #[cfg(not(feature = "tui"))]
        if self.opt.word.is_none() && !self.opt.random {
            let mut stdin = std::io::stdin().lock();
            loop {
                println!("{}", "please enter the specified final word: ".blue());
                let mut w = String::new();
                stdin.read_line(&mut w)?;
                let word = Word::parse(&w)?;
                self.states.reset();
                self.states.current_word = word;
                if self.is_acceptable_word() && self.is_final_word() {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckResult {
    InValid,
    Success,
    Wrong,
    Difficult,
}

///
/// 游戏统计状态
///
#[derive(Default, Debug, Clone)]
pub struct WordleStatistic {
    // 总局数
    pub total: u32,
    // 成功次数
    pub success_total: u32,
    // 所有猜测中最频繁使用的5个词和次数
    pub high_frequency_words: Vec<(usize, Word)>,
}
