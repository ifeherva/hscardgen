
use std::result;
use unitypack;
use serde_xml_rs;
use glob::GlobError;

#[derive(Debug)]
pub enum Error {
    UnityPackError(Box<unitypack::error::Error>),
    PathError(Box<GlobError>),
    XmlError(Box<serde_xml_rs::Error>),
    ItemNotFoundError,
    ObjectTypeError,
}

impl From<unitypack::error::Error> for Error {
    fn from(error: unitypack::error::Error) -> Error {
        Error::UnityPackError(Box::new(error))
    }
}

impl From<serde_xml_rs::Error> for Error {
    fn from(error: serde_xml_rs::Error) -> Error {
        Error::XmlError(Box::new(error))
    }
}

pub type Result<T> = result::Result<T, Error>;