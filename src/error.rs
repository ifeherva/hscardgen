
use std::result;
use unitypack;
use glob::GlobError;

#[derive(Debug)]
pub enum Error {
    UnityPackError(Box<unitypack::error::Error>),
    PathError(Box<GlobError>),
}

impl From<unitypack::error::Error> for Error {
    fn from(error: unitypack::error::Error) -> Error {
        Error::UnityPackError(Box::new(error))
    }
}

pub type Result<T> = result::Result<T, Error>;