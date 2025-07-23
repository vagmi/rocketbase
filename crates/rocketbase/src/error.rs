use thiserror::Error;
use std::io;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO Error: {0}")]
    Io(#[from] io::Error),
    #[error("Failed to parse schema: {0}")]
    SchemaPest(#[from] pest::error::Error<crate::schema::Rule>),
    #[error("Failed to parse query: {0}")]
    QueryPest(#[from] pest::error::Error<crate::query::Rule>),
}

pub type Result<T> = std::result::Result<T, Error>;
