use std::result;
use unitypack;
use glob::{GlobError, PatternError};
use serde_json;
use std::io;
use std::num;

#[derive(Debug)]
pub enum Error {
    UnityPackError(Box<unitypack::error::Error>),
    PathError(Box<GlobError>),
    JsonError(Box<serde_json::Error>),
    IOError(Box<io::Error>),
    CardNotFoundError,
    AssetNotFoundError(String),
    InvalidCardError,
    ObjectTypeError,
    SFMLError,
    NotImplementedError(String),
    InternalError,
}

impl From<unitypack::error::Error> for Error {
    fn from(error: unitypack::error::Error) -> Error {
        Error::UnityPackError(Box::new(error))
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Error {
        Error::JsonError(Box::new(error))
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error::IOError(Box::new(error))
    }
}

impl From<num::ParseIntError> for Error {
    fn from(_: num::ParseIntError) -> Error {
        Error::InternalError
    }
}

impl From<PatternError> for Error {
    fn from(_: PatternError) -> Error {
        Error::InternalError
    }
}

impl From<GlobError> for Error {
    fn from(error: GlobError) -> Error {
        Error::PathError(Box::new(error))
    }
}

pub type Result<T> = result::Result<T, Error>;
