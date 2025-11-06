//! # Error Module
//!
//! Defines the custom error type for the `x32_lib`.

use std::net::AddrParseError;
use std::io;
use osc_lib::OscError;
use std::fmt;

pub type Result<T> = std::result::Result<T, X32Error>;

#[derive(Debug)]
pub enum X32Error {
    Io(io::Error),
    AddrParse(AddrParseError),
    Osc(OscError),
    String(String),
}

impl From<io::Error> for X32Error {
    fn from(err: io::Error) -> X32Error {
        X32Error::Io(err)
    }
}

impl From<AddrParseError> for X32Error {
    fn from(err: AddrParseError) -> X32Error {
        X32Error::AddrParse(err)
    }
}

impl From<OscError> for X32Error {
    fn from(err: OscError) -> X32Error {
        X32Error::Osc(err)
    }
}

impl From<String> for X32Error {
    fn from(err: String) -> X32Error {
        X32Error::String(err)
    }
}

impl fmt::Display for X32Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            X32Error::Io(ref err) => write!(f, "IO error: {}", err),
            X32Error::AddrParse(ref err) => write!(f, "Address parse error: {}", err),
            X32Error::Osc(ref err) => write!(f, "OSC error: {}", err),
            X32Error::String(ref err) => write!(f, "Error: {}", err),
        }
    }
}

impl std::error::Error for X32Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            X32Error::Io(ref err) => Some(err),
            X32Error::AddrParse(ref err) => Some(err),
            X32Error::Osc(ref err) => Some(err),
            X32Error::String(_) => None,
        }
    }
}
