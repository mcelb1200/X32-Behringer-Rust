//! # Error Module
//!
//! Defines the custom error type for the `x32_lib`.

use std::net::AddrParseError;
use std::io;
use osc_lib::OscError;

pub type Result<T> = std::result::Result<T, X32Error>;

use std::fmt;

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
            X32Error::Custom(s) => write!(f, "X32 error: {}", s),
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

impl std::fmt::Display for X32Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            X32Error::Io(e) => write!(f, "IO Error: {}", e),
            X32Error::AddrParse(e) => write!(f, "Address Parse Error: {}", e),
            X32Error::Osc(e) => write!(f, "OSC Error: {}", e),
            X32Error::String(s) => write!(f, "X32 Error: {}", s),
        }
    }
}

impl std::error::Error for X32Error {}
