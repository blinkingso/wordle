//! tui 模式
#![allow(dead_code)]
use std::io;

use crossterm::{event::EnableMouseCapture, execute, terminal::EnterAlternateScreen};
fn ui() -> Result<(), io::Error> {
    let mut stdout = io::stdout();
    let _ = execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    Ok(())
}

pub mod ui;
pub mod widgets;
