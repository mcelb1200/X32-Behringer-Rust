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
    Custom(String),
}

impl fmt::Display for X32Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            X32Error::Io(e) => write!(f, "IO error: {}", e),
            X32Error::AddrParse(e) => write!(f, "Address parse error: {}", e),
            X32Error::Osc(e) => write!(f, "OSC error: {}", e),
            X32Error::Custom(s) => write!(f, "{}", s),
        }
    }
}

impl std::error::Error for X32Error {}

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
        X32Error::Custom(err)
    }
}
