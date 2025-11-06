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
pub mod auxin;
pub mod bus;
pub mod channel;
pub mod common;
pub mod dca;
pub mod error;
pub mod fx;
pub mod fxrtn;
pub mod headamp;
pub mod libs;
pub mod main_bus;
pub mod misc;
pub mod mtx;
pub mod output;
pub mod prefstat;
pub mod show;

use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;
use crate::error::Result;


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
