//! `x32_lib` is a library for controlling Behringer X32 and Midas M32 series of digital mixers.
//!
//! This library provides a comprehensive set of functions that map directly to the OSC
//! (Open Sound Control) commands supported by the mixers. It allows for programmatic
//! control over nearly every aspect of the hardware, from faders and mutes to effects
//! and routing.
//!
//! # Getting Started
//!
//! To begin, create a UDP socket connected to the mixer's IP address. The `create_socket`
//! function is provided for this purpose. Once connected, you can use the various functions
//! in the `command` module to build and send OSC messages.
//!
//! # Example: Set Channel Fader Level
//!
//! The following example demonstrates how to connect to an X32, set the fader level of
//! channel 1 to 75%, and then print the new level.
//!
//! ```no_run
//! use x32_lib::{create_socket, get_fader_level};
//! use x32_lib::command::channel;
//! use osc_lib::OscMessage;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let ip_address = "192.168.1.64";
//!     let socket = create_socket(ip_address, 1000)?;
//!
//!     let (address, args) = channel::set_fader(1, 0.75);
//!     let msg = OscMessage::new(address, args);
//!
//!     socket.send(&msg.to_bytes()?)?;
//!
//!     let new_level = get_fader_level(&socket, &channel::fader_level(1))?;
//!     println!("Channel 1 fader level set to: {}", new_level);
//!
//!     Ok(())
//! }
//! ```
//!
//! # Library Structure
//!
//! The library is organized into modules that correspond to the major sections of the
//! X32/M32 console:
//!
//! - [`command`]: Contains functions for generating OSC command strings and arguments for
//!   various mixer controls.
//! - [`error`]: Defines the custom `X32Error` type and `Result` alias for robust error
//!   handling.
//! - [`common`]: Provides common utilities and helper functions used throughout the library.
//!
#[cfg(test)]
mod tests;

pub mod command;
pub mod common;
pub mod error;
pub mod main_bus;

use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;

pub use crate::error::{Result, X32Error};
pub use osc_lib::{OscArg, OscError, OscMessage};

/// Creates a UDP socket and connects to an X32/M32 console.
///
/// This function handles the boilerplate of creating a UDP socket, binding it to a
/// dynamic local port, and connecting it to the mixer's IP address and default port
/// (10023). It also sets a read timeout to prevent blocking indefinitely on receive
/// operations.
///
/// # Arguments
///
/// * `ip` - The IP address of the console (e.g., "192.168.1.64").
/// * `timeout` - The read timeout for the socket in milliseconds.
///
/// # Returns
///
/// A `Result` containing the configured `UdpSocket` or an `X32Error` if the
/// connection fails.
pub fn create_socket(ip: &str, timeout: u64) -> Result<UdpSocket> {
    // If the IP address does not contain a port, add the default X32 port.
    let full_ip = if ip.contains(':') && !ip.starts_with('[') {
        ip.to_string()
    } else if ip.contains("]:") {
        ip.to_string()
    } else {
        format!("{}:10023", ip)
    };
    let remote_addr: SocketAddr = full_ip.parse()?;

    // Bind to a local address compatible with the remote address family.
    let local_addr: SocketAddr = if remote_addr.is_ipv4() {
        "0.0.0.0:0".parse()?
    } else {
        "[::]:0".parse()?
    };

    let socket = UdpSocket::bind(local_addr)?;
    socket.connect(remote_addr)?;
    socket.set_read_timeout(Some(Duration::from_millis(timeout)))?;

    Ok(socket)
}

/// Queries the mixer for the type of effect in a given FX slot.
///
/// # Arguments
///
/// * `socket` - A `UdpSocket` connected to the mixer.
/// * `slot` - The FX slot number (1-8).
///
/// # Returns
///
/// A `Result` containing the effect type as an integer, or an `X32Error` on failure.
pub fn get_fx_type(socket: &UdpSocket, slot: u8) -> Result<i32> {
    let msg = OscMessage::new(format!("/fx/{}/type", slot), vec![]);
    socket.send(&msg.to_bytes()?)?;
    let mut buf = [0; 512];
    let len = socket.recv(&mut buf)?;
    let response = OscMessage::from_bytes(&buf[..len])?;
    if let Some(OscArg::Int(fx_type)) = response.args.get(0) {
        Ok(*fx_type)
    } else {
        Err(OscError::ParseError("Unexpected response from mixer".to_string()).into())
    }
}

/// Queries the mixer for the current level of a fader.
///
/// # Arguments
///
/// * `socket` - A `UdpSocket` connected to the mixer.
/// * `fader_addr` - The OSC address of the fader to query (e.g., "/ch/01/mix/fader").
///
/// # Returns
///
/// A `Result` containing the fader level as a float (0.0 to 1.0), or an `X32Error`.
pub fn get_fader_level(socket: &UdpSocket, fader_addr: &str) -> Result<f32> {
    let msg = OscMessage::new(fader_addr.to_string(), vec![]);
    socket.send(&msg.to_bytes()?)?;
    let mut buf = [0; 512];
    let len = socket.recv(&mut buf)?;
    let response = OscMessage::from_bytes(&buf[..len])?;
    if let Some(OscArg::Float(level)) = response.args.get(0) {
        Ok(*level)
    } else {
        Err(OscError::ParseError("Unexpected response from mixer".to_string()).into())
    }
}

/// Verifies that a specific FX slot contains the expected effect type.
///
/// This is useful for ensuring that a script is targeting the correct effect before
/// attempting to modify its parameters.
///
/// # Arguments
///
/// * `socket` - A `UdpSocket` connected to the mixer.
/// * `slot` - The FX slot number (1-8).
/// * `expected_type` - A string representing the expected effect type (e.g., "GEQ2").
///
/// # Returns
///
/// A `Result` containing `true` if the effect type matches, or `false` otherwise.
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

/// Gets the value of a floating-point parameter from the mixer.
///
/// # Arguments
///
/// * `socket` - A `UdpSocket` connected to the mixer.
/// * `address` - The OSC address of the parameter to get.
///
/// # Returns
///
/// A `Result` containing the parameter's value as a float.
pub fn get_parameter(socket: &UdpSocket, address: &str) -> Result<f32> {
    let msg = OscMessage::new(address.to_string(), vec![]);
    socket.send(&msg.to_bytes()?)?;
    let mut buf = [0; 512];
    let len = socket.recv(&mut buf)?;
    let response = OscMessage::from_bytes(&buf[..len])?;
    if let Some(OscArg::Float(value)) = response.args.get(0) {
        Ok(*value)
    } else {
        Err(OscError::ParseError("Unexpected response from mixer".to_string()).into())
    }
}

/// Sets the value of a floating-point parameter on the mixer.
///
/// # Arguments
///
/// * `socket` - A `UdpSocket` connected to the mixer.
/// * `address` - The OSC address of the parameter to set.
/// * `value` - The new value for the parameter.
pub fn set_parameter(socket: &UdpSocket, address: &str, value: f32) -> Result<()> {
    let msg = OscMessage::new(address.to_string(), vec![OscArg::Float(value)]);
    socket.send(&msg.to_bytes()?)?;
    Ok(())
}
