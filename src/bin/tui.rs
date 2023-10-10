use std::io::{self, BufRead};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::{Rng, SeedableRng};
use ratatui::{prelude::CrosstermBackend, Terminal};
use structopt::StructOpt;
use wordle::{
    buildin_words::{ACCEPTABLE, FINAL},
    command::Opt,
    error::Result,
    state::Mode,
    word::Word,
    wordle::Wordle,
};

fn main() -> Result<()> {
    enable_raw_mode()?;
    crossterm::execute!(io::stderr(), EnterAlternateScreen, EnableMouseCapture)?;
    let opt = Opt::from_args();
    let mode = if atty::is(atty::Stream::Stdout) {
        Mode::Interactive
    } else {
        Mode::Test
    };
    // get acceptable words
    let acceptable_set = match opt.acceptable_set {
        Some(ref path) => Wordle::read_input_file(path)?,
        None => ACCEPTABLE.iter().map(|s| s.to_string()).collect(),
    };
    // get final words
    let final_set = match opt.final_set {
        Some(ref path) => Wordle::read_input_file(path)?,
        None => FINAL.iter().map(|s| s.to_string()).collect(),
    };

    let backend = CrosstermBackend::new(io::stderr());
    let mut terminal = Terminal::new(backend)?;

    // 随机答案模式
    let final_word: String = if opt.random {
        let seed = if let Some(seed) = opt.seed {
            seed
        } else {
            2048
        };
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let index: usize = rng.gen_range(0..ACCEPTABLE.len());
        ACCEPTABLE[index].to_string()
    } else {
        // 指定答案模式
        if let Some(ref word) = opt.word {
            word.to_string()
        } else {
            // 默认情况下取子集中最后一个单词
            let mut stdin = std::io::stdin().lock();
            let mut final_word = String::new();
            stdin.read_line(&mut final_word)?;
            final_word.remove(final_word.len() - 1);
            final_word
        }
    };
    let mut wordle = Wordle::default();
    wordle.final_word = Word::parse(final_word)?;
    wordle.mode = mode;
    wordle.acceptable_set = acceptable_set;
    wordle.final_set = final_set;
    wordle.run_app(&mut terminal)?;

    disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    Ok(())
}
