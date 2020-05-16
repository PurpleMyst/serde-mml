use std::fmt;
use std::io;

use serde::{de, ser};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    IOError(#[from] io::Error),

    #[error("{0}")]
    CustomSerializeError(String),

    #[error("{0}")]
    CustomDeserializeError(String),

    #[error("{0}")]
    TypeParseError(#[from] crate::ty::ParseError),

    #[error("Unexpected EOF")]
    UnexpectedEOF,
}

impl ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Self::CustomSerializeError(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Self::CustomDeserializeError(msg.to_string())
    }
}
