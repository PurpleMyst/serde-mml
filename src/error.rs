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

    #[error("{0}")]
    ParseCharError(#[from] std::char::ParseCharError),

    #[error("{0}")]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("{0}")]
    ParseBoolError(#[from] std::str::ParseBoolError),

    #[error("{0}")]
    ParseFloatError(#[from] std::num::ParseFloatError),

    #[error("{0}")]
    B64DecodeError(#[from] base64::DecodeError),

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

pub type Result<T, E = Error> = std::result::Result<T, E>;
