//! `x32_lib` is a shared library for interacting with Behringer X32 and Midas M32 consoles.
//! It provides common functionality for creating sockets, sending and receiving OSC messages,
//! and handling X32-specific errors.

pub mod dca;
pub mod fx;

pub mod bus;
pub mod common;

use std::io;
use std::net::{AddrParseError, SocketAddr, UdpSocket};
use std::time::Duration;
use osc_lib::{OscMessage, OscArg, OscError};

pub mod channel;
pub mod auxin;
pub mod fx_rtn;
pub use fx_rtn::*;


/// Represents the possible errors that can occur when interacting with an X32 console.
#[derive(Debug)]
pub enum X32Error {
    /// An I/O error occurred.
    Io(io::Error),
    /// An error occurred while parsing an IP address.
    AddrParse(AddrParseError),
    /// An error occurred while handling an OSC message.
    Osc(OscError),
    /// An unexpected response was received from the X32.
    UnexpectedResponse,
    /// A custom error message.
    Custom(String),
}

impl std::fmt::Display for X32Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            X32Error::Io(e) => write!(f, "I/O error: {}", e),
            X32Error::AddrParse(e) => write!(f, "Address parsing error: {}", e),
            X32Error::Osc(e) => write!(f, "OSC error: {}", e),
            X32Error::UnexpectedResponse => write!(f, "Unexpected response from X32"),
            X32Error::Custom(s) => write!(f, "{}", s),
        }
    }
}

impl std::error::Error for X32Error {}

impl From<io::Error> for X32Error {
    fn from(err: io::Error) -> Self {
        X32Error::Io(err)
    }
}

impl From<AddrParseError> for X32Error {
    fn from(err: AddrParseError) -> Self {
        X32Error::AddrParse(err)
    }
}

impl From<OscError> for X32Error {
    fn from(err: OscError) -> Self {
        X32Error::Osc(err)
    }
}

impl From<String> for X32Error {
    fn from(s: String) -> Self {
        X32Error::Custom(s)
    }
}

/// A type alias for `Result` with the error type `X32Error`.
pub type Result<T> = std::result::Result<T, X32Error>;

/// Creates a UDP socket, connects to the X32, and sets a read timeout.
///
/// # Arguments
///
/// * `ip` - The IP address of the X32 console.
/// * `timeout_ms` - The read timeout in milliseconds.
pub fn create_socket(ip: &str, timeout_ms: u64) -> Result<UdpSocket> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_read_timeout(Some(Duration::from_millis(timeout_ms)))?;

    let x32_addr: SocketAddr = ip.parse().unwrap_or_else(|_| format!("{}:10023", ip).parse().unwrap());
    socket.connect(x32_addr)?;

    Ok(socket)
}

/// Queries the X32 for the type of effect in a given FX slot.
///
/// # Arguments
///
/// * `socket` - The UDP socket connected to the X32.
/// * `slot` - The FX slot number (1-8).
pub fn get_fx_type(socket: &UdpSocket, slot: u8) -> Result<i32> {
    let msg = OscMessage::new(format!("/fx/{}/type", slot), vec![]);
    socket.send(&msg.to_bytes()?)?;
    let mut buf = [0; 512];
    let len = socket.recv(&mut buf)?;
    let response = OscMessage::from_bytes(&buf[..len])?;
    if let Some(OscArg::Int(fx_type)) = response.args.get(0) {
        Ok(*fx_type)
    } else {
        Err(X32Error::UnexpectedResponse)
    }
}

/// Queries the X32 for the current level of a fader.
///
/// # Arguments
///
/// * `socket` - The UDP socket connected to the X32.
/// * `fader_addr` - The OSC address of the fader to query.
pub fn get_fader_level(socket: &UdpSocket, fader_addr: &str) -> Result<f32> {
    let msg = OscMessage::new(fader_addr.to_string(), vec![]);
    socket.send(&msg.to_bytes()?)?;
    let mut buf = [0; 512];
    let len = socket.recv(&mut buf)?;
    let response = OscMessage::from_bytes(&buf[..len])?;
    if let Some(OscArg::Float(level)) = response.args.get(0) {
        Ok(*level)
    } else {
        Err(X32Error::UnexpectedResponse)
    }
}

/// Verifies that a specific FX slot contains the expected effect type.
///
/// # Arguments
///
/// * `socket` - The UDP socket connected to the X32.
/// * `slot` - The FX slot number (1-8).
/// * `expected_type` - A string representing the expected effect type (e.g., "GEQ2").
pub fn verify_fx_type(socket: &UdpSocket, slot: u8, expected_type: &str) -> Result<bool> {
    let msg = OscMessage::new("/node".to_string(), vec![OscArg::String(format!("fx/{}", slot))]);
    socket.send(&msg.to_bytes()?)?;
    let mut buf = [0; 512];
    let len = socket.recv(&mut buf)?;
    let response = OscMessage::from_bytes(&buf[..len])?;
    if let Some(OscArg::String(response_str)) = response.args.get(0) {
        Ok(response_str.contains(expected_type))
    } else {
        Ok(false)
    }
}

/// Gets the value of a floating-point parameter from the X32.
///
/// # Arguments
///
/// * `socket` - The UDP socket connected to the X32.
/// * `address` - The OSC address of the parameter to get.
pub fn get_parameter(socket: &UdpSocket, address: &str) -> Result<f32> {
    let msg = OscMessage::new(address.to_string(), vec![]);
    socket.send(&msg.to_bytes()?)?;
    let mut buf = [0; 512];
    let len = socket.recv(&mut buf)?;
    let response = OscMessage::from_bytes(&buf[..len])?;
    if let Some(OscArg::Float(value)) = response.args.get(0) {
        Ok(*value)
    } else {
        Err(X32Error::UnexpectedResponse)
    }
}

/// Sets the value of a floating-point parameter on the X32.
///
/// # Arguments
///
/// * `socket` - The UDP socket connected to the X32.
/// * `address` - The OSC address of the parameter to set.
/// * `value` - The new value for the parameter.
pub fn set_parameter(socket: &UdpSocket, address: &str, value: f32) -> Result<()> {
    let msg = OscMessage::new(address.to_string(), vec![OscArg::Float(value)]);
    socket.send(&msg.to_bytes()?)?;
    Ok(())
}
