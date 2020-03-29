use serde::de;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    Reqwest(reqwest::Error),
    Server(String),
    Client(String),
    Serde(String),
    Csv(csv::Error),
    Json(serde_json::Error),
    Io(std::io::Error),
    Utf8(std::str::Utf8Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Reqwest(ref err) => write!(f, "Reqwest error: {}", err),
            Error::Server(ref s) => write!(f, "Server error: {}", s),
            Error::Client(ref s) => write!(f, "Client error: {}", s),
            Error::Serde(ref s) => write!(f, "Serde error: {}", s),
            Error::Csv(ref err) => write!(f, "Csv error: {}", err),
            Error::Json(ref err) => write!(f, "Json error: {}", err),
            Error::Io(ref err) => write!(f, "Io error: {}", err),
            Error::Utf8(ref err) => write!(f, "Utf8 error: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Error::Reqwest(ref err) => Some(err),
            Error::Server(..) => None,
            Error::Client(..) => None,
            Error::Serde(..) => None,
            Error::Csv(ref err) => Some(err),
            Error::Json(ref err) => Some(err),
            Error::Io(ref err) => Some(err),
            Error::Utf8(ref err) => Some(err),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        Error::Reqwest(err)
    }
}

impl From<csv::Error> for Error {
    fn from(err: csv::Error) -> Error {
        Error::Csv(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::Json(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(err: std::string::FromUtf8Error) -> Error {
        Error::Utf8(err.utf8_error())
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(err: std::num::ParseIntError) -> Error {
        Error::Server(format!("{}", err))
    }
}

/// when deserailizing, the serde framework requires
/// the ability to convert local error to serde::de::Error
impl de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Error::Serde(format!("{}", msg))
    }
}
