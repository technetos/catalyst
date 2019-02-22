use std::error::Error as StdError;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    SerdeJson(serde_json::Error),
    H2(h2::Error),
    Http(http::Error),
    FromUtf8(std::string::FromUtf8Error),
    Str(String),
    Io(std::io::Error),
}

impl From<h2::Error> for Error {
    fn from(e: h2::Error) -> Self {
        Error::H2(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::SerdeJson(e)
    }
}

impl From<http::Error> for Error {
    fn from(e: http::Error) -> Self {
        Error::Http(e)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Error::FromUtf8(e)
    }
}

impl From<&str> for Error {
    fn from(e: &str) -> Self {
        Error::Str(e.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::SerdeJson(e) => f.write_str(e.description()),
            Error::H2(e) => f.write_str(e.description()),
            Error::Http(e) => f.write_str(e.description()),
            Error::FromUtf8(e) => f.write_str(e.description()),
            Error::Str(e) => f.write_str(e),
            Error::Io(e) => f.write_str(e.description()),
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match self {
            Error::SerdeJson(e) => e.description(),
            Error::H2(e) => e.description(),
            Error::Http(e) => e.description(),
            Error::FromUtf8(e) => e.description(),
            Error::Str(e) => &e[..],
            Error::Io(e) => e.description(),
        }
    }
}
