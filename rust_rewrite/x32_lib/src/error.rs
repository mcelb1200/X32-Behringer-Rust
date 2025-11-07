use std::io;
use std::net::AddrParseError;
use osc_lib::OscError;

pub type Result<T> = std::result::Result<T, X32Error>;

#[derive(Debug)]
pub enum X32Error {
    Io(io::Error),
    AddrParse(AddrParseError),
    Osc(OscError),
    Custom(String),
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

impl std::fmt::Display for X32Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            X32Error::Io(ref err) => write!(f, "IO error: {}", err),
            X32Error::AddrParse(ref err) => write!(f, "Address parse error: {}", err),
            X32Error::Osc(ref err) => write!(f, "OSC error: {}", err),
            X32Error::Custom(ref err) => write!(f, "X32 error: {}", err),
        }
    }
}

impl std::error::Error for X32Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            X32Error::Io(ref err) => Some(err),
            X32Error::AddrParse(ref err) => Some(err),
            X32Error::Osc(ref err) => Some(err),
            X32Error::Custom(_) => None,
        }
    }
}
