//! `x32_lib`'s custom `Error` and `Result` types.

use osc_lib::OscError;
use std::fmt;
use std::io;
use std::net::AddrParseError;

/// A type alias for `Result` with the error type `X32Error`.
pub type Result<T> = std::result::Result<T, X32Error>;

/// The error type for operations within the `x32_lib`.
///
/// This enum encapsulates the various kinds of errors that can occur,
/// including I/O errors, OSC message parsing errors, and network
/// address parsing errors.
#[derive(Debug)]
pub enum X32Error {
    /// An I/O error, typically from reading or writing to a UDP socket.
    Io(io::Error),
    /// An error that occurred while parsing a network IP address.
    AddrParse(AddrParseError),
    /// An error from the underlying `osc_lib`, such as a malformed
    /// OSC message or an unsupported type tag.
    Osc(OscError),
    /// A custom, application-level error.
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
