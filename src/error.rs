use thiserror::Error;
use std::io;

use crate::schema::Rule;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO Error: {0}")]
    Io(#[from] io::Error),
    #[error("Failed to parse schema: {0}")]
    Pest(#[from] pest::error::Error<Rule>),
}

pub type Result<T> = std::result::Result<T, Error>;
