use std::result;

#[derive(Debug)]
pub enum NoterError {
    LoadError(std::io::Error),
    ParseError(serde_json::Error),
    Mess(&'static str),
}

impl From<std::io::Error> for NoterError {
    fn from(e: std::io::Error) -> Self {
        NoterError::LoadError(e)
    }
}

impl From<serde_json::Error> for NoterError {
    fn from(e: serde_json::Error) -> Self {
        NoterError::ParseError(e)
    }
}

impl From<&'static str> for NoterError {
    fn from(e: &'static str) -> Self {
        NoterError::Mess(e)
    }
}

pub type Result<T> = result::Result<T, NoterError>;
