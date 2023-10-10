pub mod buildin_words;
// #[cfg(not(any(feature = "tui", feature = "gui")))]
#[cfg(feature = "cmd")]
pub mod cmd;
pub mod command;
pub mod error;
#[cfg(feature = "gui")]
pub mod gui;
pub mod state;
pub mod states;
#[cfg(feature = "tui")]
pub mod tui;
pub mod word;
pub mod wordle;
