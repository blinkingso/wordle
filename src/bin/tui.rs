use wordle::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(feature = "tui")]
    {
        use rand::{Rng, SeedableRng};

        use structopt::StructOpt;
        use wordle::{
            buildin_words::{ACCEPTABLE, FINAL},
            command::Opt,
            state::Mode,
            word::Word,
            wordle::Wordle,
        };
        std::env::set_var("RUST_LOG", "info");
        pretty_env_logger::init();
        let opt = Opt::from_args();

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

        // 随机答案模式
        let final_word = if opt.random {
            let seed = if let Some(seed) = opt.seed {
                seed
            } else {
                2048
            };
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            let index: usize = rng.gen_range(0..final_set.len());
            final_set[index].to_string()
        } else {
            // 指定答案模式
            opt.word.clone().unwrap_or(String::new())
        };

        let mut wordle = Wordle {
            opt,
            mode: Mode::Tui,
            acceptable_set,
            final_set,
            final_word: Word::parse(final_word).unwrap_or(Word::default()),
            ..Default::default()
        };

        wordle::tui::controller::run(&mut wordle).await?;
        return Ok(());
    }

    eprintln!("请开启feature -> tui");
    return Ok(());
}
