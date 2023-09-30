use std::path::PathBuf;

use structopt::StructOpt;

use crate::error::{Result, WordError};

#[derive(StructOpt, Debug)]
#[structopt(author = "yaphets", about = "wordle game in terminal usage.")]
pub struct Opt {
    // 指定答案模式
    #[structopt(short, long, help = "a specified word for guessing, default ``")]
    pub word: Option<String>,

    // 随机模式
    #[structopt(
        short,
        long,
        help = "whether or not startup final word in gen-random model, default `false`"
    )]
    pub random: bool,

    /// 困难模式
    #[structopt(
        short = "D",
        long,
        help = "whether or not startup difficult model, default `false`"
    )]
    pub difficult: bool,

    #[structopt(short = "f", long = "final-set", help = "final set from an input file")]
    pub final_set: Option<PathBuf>,

    #[structopt(
        short = "a",
        long = "acceptable-set",
        help = "acceptable set from an input file"
    )]
    pub acceptable_set: Option<PathBuf>,

    #[structopt(short = "t", long, help = "print states in test model")]
    pub stats: bool,

    #[structopt(
        short = "S",
        long,
        help = "save or load game state from path `state.json`"
    )]
    pub state: Option<PathBuf>,

    #[structopt(
        short = "d",
        long = "day",
        parse(try_from_str = parse_day),
        help = "start time when start default is 1"
    )]
    pub day: Option<u32>,
    #[structopt(short = "s", long, help = "seed for rand")]
    pub seed: Option<u64>,
}

fn parse_day(src: &str) -> Result<u32> {
    let target = src.parse::<u32>()?;
    if !(1..=6).contains(&target) {
        return Err(WordError::CustomError("day must be in 1..=6".to_string()));
    }
    Ok(target)
}
