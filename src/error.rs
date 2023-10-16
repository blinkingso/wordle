use std::{io, num::ParseIntError};

use thiserror::Error as ThisError;

pub type Result<T> = std::result::Result<T, WordError>;

#[derive(Debug, ThisError)]
pub enum WordError {
    #[error("io error")]
    IoError(#[from] io::Error),
    #[error("json error")]
    JsonError,
    #[error("your word `{0}` is not in the acceptable word list")]
    InValidWord(String),
    #[error("parse int error")]
    ParseIntError(#[from] ParseIntError),
    #[error("custom error for: {0}")]
    CustomError(String),
    #[error("eyre error")]
    EyreError(#[from] color_eyre::eyre::Error),
    #[error("unknown error")]
    UnknownError,
}
