//! # x32_lib
//!
//! `x32_lib` is a library for controlling the Behringer X32 and Midas M32 series of digital mixers.
//! It provides a set of functions that map directly to the OSC (Open Sound Control) commands
//! supported by the mixers, allowing for programmatic control over nearly every aspect of the hardware.
//!
//! This library is a Rust rewrite of the original C library by Patrick-Gilles Maillot.
//!
//! ## Features
//!
//! - **Comprehensive Control:** Access channels, buses, DCAs, effects, routing, and more.
//! - **Idiomatic Rust:** Built with a focus on safety, simplicity, and reliability.
//! - **Dynamic OSC Generation:** Functions return OSC address strings and arguments, ready to be sent.
//!
//! ## Example
//!
//! The following example demonstrates how to generate an OSC message to set the fader level of channel 1.
//!
//! ```
//! use x32_lib::channel;
//! use osc_lib::OscArg;
//!
//! fn main() {
//!     let (address, args) = channel::set_fader(1, 0.75);
//!
//!     assert_eq!(address, "/ch/01/mix/fader");
//!     assert_eq!(args, vec![OscArg::Float(0.75)]);
//!
//!     // This address and args can now be sent to the X32 using an OSC client.
//! }
//! ```
//!
//! ## Modules
//!
//! The library is organized into modules that correspond to the major sections of the X32/M32 console:
//!
//! - `channel`: Input channels (1-32)
//! - `auxin`: Auxiliary inputs
//! - `bus`: Mix buses (1-16)
//! - `mtx`: Matrix outputs (1-6)
//! - `main_bus`: Main stereo and mono/center buses
//! - `dca`: Digitally Controlled Amplifiers (1-8)
//! - `fx`: Effects processors (1-8)
//! - `fxrtn`: Effects returns
//! - `output`: Physical output routing
//! - `headamp`: Headamp (preamp) control
//! - `show`: Show, scene, cue, and snippet management
//! - `misc`: Miscellaneous settings
//! - `prefstat`: Preferences and status
//! - `libs`: Library presets
//!
#[cfg(test)]
mod tests;

pub mod dca;
pub mod fx;
pub mod bus;
pub mod auxin;
pub mod channel;
pub mod common;
pub mod error;
pub mod fxrtn;
pub mod headamp;
pub mod libs;
pub mod main_bus;
pub mod misc;
pub mod mtx;
pub mod output;
pub mod prefstat;
pub mod show;
pub mod cfg_main;


use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;
pub use crate::error::Result;
use osc_lib::{OscMessage, OscArg, OscError};

/// Creates a UDP socket and connects to the X32/M32 console.
///
/// # Arguments
///
/// * `ip` - The IP address of the console.
/// * `timeout` - The read timeout for the socket in milliseconds.
pub fn create_socket(ip: &str, timeout: u64) -> Result<UdpSocket> {
    let remote_addr: SocketAddr = format!("{}:10023", ip).parse()?;
    let local_addr: SocketAddr = format!("{}:10024", ip).parse()?;

    let socket = UdpSocket::bind(local_addr)?;
    socket.connect(remote_addr)?;
    socket.set_read_timeout(Some(Duration::from_millis(timeout)))?;

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
        Err(OscError::UnexpectedResponse.into())
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
        Err(OscError::UnexpectedResponse.into())
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
        Err(OscError::UnexpectedResponse.into())
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


