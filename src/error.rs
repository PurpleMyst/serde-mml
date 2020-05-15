use std::fmt;
use std::io;

use serde::{de, ser};

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    CustomSerializeError(String),
    CustomDeserializeError(String),
    UnexpectedEOF,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::IOError(error) => error.fmt(f),
            Error::CustomSerializeError(msg) | Error::CustomDeserializeError(msg) => msg.fmt(f),
            Error::UnexpectedEOF => f.pad("Unexpected EOF"),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::IOError(error)
    }
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
