
use reqwest;
use std;
use std::result;
use serde_json;
use url;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    NoResultsReturned,
    InvalidOptions,
    NotSuccess,
    Reqwest(reqwest::Error),
    Json(serde_json::Error),
    Io(std::io::Error),
    Url(url::ParseError)
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        Error::Reqwest(err)
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

impl From<url::ParseError> for Error {
    fn from(err: url::ParseError) -> Error {
        Error::Url(err)
    }
}
