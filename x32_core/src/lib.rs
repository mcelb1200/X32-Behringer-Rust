//! `x32_core` provides the core logic for an X32/M32 mixer emulator.
//!
//! This crate is designed to simulate the behavior of a real X32 mixer, allowing for
//! offline testing and development of X32 control applications without needing physical
//! hardware. The central component is the `Mixer` struct, which maintains the state
//! of the mixer and processes incoming OSC messages.
//!
//! # Key Features
//!
//! - **Stateful Emulation:** The `Mixer` struct uses a `HashMap` to store the current
//!   value of every parameter, creating a persistent state that can be modified and
//!   queried.
//! - **OSC Message Dispatching:** The `dispatch` method processes raw OSC byte messages,
//!   interpreting them as either requests for data or commands to change a setting.
//! - **Seedable State:** The `seed_from_lines` method allows you to initialize the
//!   mixer's state from a predefined set of OSC commands, making it easy to set up
//!   specific scenarios for testing.
//!
//! # Credits
//!
//! *   **Original concept and work on the C library:** Patrick-Gilles Maillot
//! *   **Additional concepts by:** [User]
//! *   **Rust implementation by:** [User]
//!
//! # Example: Creating and Interacting with a Mixer Emulator
//!
//! ```
//! use x32_core::Mixer;
//! use osc_lib::{OscMessage, OscArg};
//! use std::net::{SocketAddr, IpAddr, Ipv4Addr};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut mixer = Mixer::new();
//!
//!     // Seed the mixer with an initial fader level for channel 1
//!     let seed_data = vec!["/ch/01/mix/fader,f    0.75"];
//!     mixer.seed_from_lines(seed_data);
//!
//!     // Create an OSC message to request the fader level of channel 1
//!     let request_msg = OscMessage::new("/ch/01/mix/fader".to_string(), vec![]);
//!     let request_bytes = request_msg.to_bytes()?;
//!
//!     let test_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 10023);
//!
//!     // Dispatch the message to the mixer
//!     let responses = mixer.dispatch(&request_bytes, test_addr)?;
//!     if let Some((addr, response_bytes)) = responses.first() {
//!         let response_msg = OscMessage::from_bytes(response_bytes)?;
//!         assert_eq!(response_msg.path, "/ch/01/mix/fader");
//!         assert_eq!(response_msg.args, vec![OscArg::Float(0.75)]);
//!         println!("Successfully retrieved channel 1 fader level: 0.75");
//!     }
//!
//!     Ok(())
//! }
//! ```

use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use osc_lib::{OscArg, OscMessage};

#[cfg(test)]
mod tests;

// --- Static Data for Mixer Parameters ---

// The following static arrays define the string representations for various
// enumerated parameters on the X32 console.

/// String representations for boolean "OFF" and "ON" states.
pub static OFF_ON: &[&str] = &[" OFF", " ON"];
/// String representations for automix group assignments.
pub static XAMXGRP: &[&str] = &[" OFF", " X", " Y"];
/// String representations for scribble strip colors.
pub static XCOLORS: &[&str] = &[
    " OFF", " RD", " GN", " YE", " BL", " MG", " CY", " WH", " OFFi", " RDi", " GNi", " YEi",
    " BLi", " MGi", " CYi", " WHi",
];
/// String representations for monitor mix modes (LR+Mono vs LCR).
pub static XMNMODE: &[&str] = &[" LR+M", " LCR"];
/// String representations for channel solo modes (PFL vs AFL).
pub static XCHMODE: &[&str] = &[" PFL", " AFL"];
/// String representations for high-pass filter slopes.
pub static XHSLP: &[&str] = &[" 12", " 18", " 24"];
/// String representations for gate modes.
pub static XGMODE: &[&str] = &[" EXP2", " EXP3", " EXP", " GATE", " DUCK"];
/// String representations for dynamics modes (compressor/expander).
pub static XDYMODE: &[&str] = &[" COMP", " EXP"];
/// String representations for dynamics detector types (Peak/RMS).
pub static XDYDET: &[&str] = &[" PEAK", " RMS"];
/// String representations for dynamics envelope types (Linear/Logarithmic).
pub static XDYENV: &[&str] = &[" LIN", " LOG"];
/// String representations for dynamics ratio settings.
pub static XDYRAT: &[&str] = &[
    " 1.1", " 1.3", " 1.5", " 2.0", " 2.5", " 3.0", " 4.0", " 5.0", " 7.0", " 10", " 20", " 100",
];
/// String representations for dynamics filter types.
pub static XDYFTYP: &[&str] = &[
    " LC6", " LC12", " HC6", " HC12", " 1.0", " 2.0", " 3.0", " 5.0", " 10.0",
];
/// String representations for dynamics processing position (pre/post).
pub static XDYPPOS: &[&str] = &[" PRE", " POST"];
/// String representations for insert selections.
pub static XISEL: &[&str] = &[
    " OFF", " FX1L", " FX1R", " FX2L", " FX2R", " FX3L", " FX3R", " FX4L", " FX4R", " FX5L",
    " FX5R", " FX6L", " FX6R", " FX7L", " FX7R", " FX8L", " FX8R", " AUX1", " AUX2", " AUX3",
    " AUX4", " AUX5", " AUX6",
];
/// String representations for EQ types.
pub static XEQTY1: &[&str] = &[" LCut", " LShv", " PEQ", " VEQ", " HShv", " HCut"];
// ... and so on for the rest of the static arrays ...

/// Represents the internal state of the mixer.
#[derive(Debug, Clone)]
pub struct MixerState {
    values: HashMap<String, OscArg>,
}

impl Default for MixerState {
    fn default() -> Self {
        Self::new()
    }
}

impl MixerState {
    /// Creates a new, empty `MixerState`.
    ///
    /// # Returns
    ///
    /// A new `MixerState` instance.
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    /// Sets a value in the mixer's state.
    ///
    /// # Arguments
    ///
    /// * `path` - The OSC address path of the parameter.
    /// * `arg` - The new value for the parameter.
    pub fn set(&mut self, path: &str, arg: OscArg) {
        self.values.insert(path.to_string(), arg);
    }

    /// Gets a value from the mixer's state.
    ///
    /// # Arguments
    ///
    /// * `path` - The OSC address path of the parameter.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the value if it exists.
    pub fn get(&self, path: &str) -> Option<&OscArg> {
        self.values.get(path)
    }
}

/// A struct that emulates the behavior of an X32 mixer.
pub struct Mixer {
    state: MixerState,
    clients: Vec<(SocketAddr, Instant)>,
}

impl Default for Mixer {
    fn default() -> Self {
        Self::new()
    }
}

impl Mixer {
    /// Creates a new `Mixer` with a default, empty state.
    ///
    /// # Returns
    ///
    /// A new `Mixer` instance.
    pub fn new() -> Self {
        Self {
            state: MixerState::new(),
            clients: Vec::new(),
        }
    }

    /// Seeds the mixer's state from a vector of OSC command strings.
    ///
    /// This is useful for setting up a specific state for testing. Each string
    /// should be in the format: `/osc/path,t    value`, where `t` is the OSC type
    /// tag (`i`, `f`, or `s`).
    ///
    /// # Arguments
    ///
    /// * `lines` - A vector of strings containing OSC commands.
    pub fn seed_from_lines(&mut self, lines: Vec<&str>) {
        for line in lines {
            let parts: Vec<&str> = line.splitn(2, ',').collect();
            if parts.len() == 2 {
                let path = parts[0].trim();
                let arg_parts: Vec<&str> = parts[1].trim().splitn(2, '\t').collect();
                if arg_parts.len() == 2 {
                    let arg_type = arg_parts[0];
                    let arg_value = arg_parts[1];
                    let arg = match arg_type {
                        "i" => OscArg::Int(arg_value.parse().unwrap()),
                        "f" => OscArg::Float(arg_value.parse().unwrap()),
                        "s" => OscArg::String(arg_value.to_string()),
                        _ => continue,
                    };
                    self.state.set(path, arg);
                }
            }
        }
    }

    /// Dispatches an incoming OSC message and returns a list of responses to send to specific clients.
    ///
    /// This is the core method of the emulator. It takes a raw byte slice representing
    /// an OSC message, parses it, and then either updates the internal state or generates
    /// a response based on the current state.
    ///
    /// # Arguments
    ///
    /// * `msg` - A byte slice containing the OSC message.
    /// * `remote_addr` - The socket address of the client sending the message.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `Vec<(SocketAddr, Vec<u8>)>` with the responses to send
    /// to clients. If the message was a request for data or an info query, the response
    /// is sent back to the requester. If it was a set command, the new value is broadcasted
    /// to all registered `/xremote` clients.
    pub fn dispatch(
        &mut self,
        msg: &[u8],
        remote_addr: SocketAddr,
    ) -> Result<Vec<(SocketAddr, Vec<u8>)>, Box<dyn std::error::Error>> {
        let osc_msg = OscMessage::from_bytes(msg)?;
        let mut responses = Vec::new();

        // Expire old clients before processing
        let now = Instant::now();
        self.clients.retain(|&(_, expiry)| now < expiry);

        if osc_msg.path == "/xremote" {
            // Check if already registered
            let mut found = false;
            for client in &mut self.clients {
                if client.0 == remote_addr {
                    client.1 = now + Duration::from_secs(10);
                    found = true;
                    break;
                }
            }

            if !found {
                if self.clients.len() < 4 {
                    self.clients
                        .push((remote_addr, now + Duration::from_secs(10)));
                } else {
                    eprintln!("maximum client capacity reached");
                }
            }
            return Ok(responses);
        }

        // Handle the /info command, which is a request for mixer information.
        if osc_msg.path == "/info" {
            let arg1 = OscArg::String("V2.07".to_string());
            let arg2 = OscArg::String("X32 Emulator".to_string());
            let arg3 = OscArg::String("X32".to_string());
            let arg4 = OscArg::String("4.06".to_string());
            let bytes = OscMessage::serialize_to_bytes("/info", [&arg1, &arg2, &arg3, &arg4])?;
            responses.push((remote_addr, bytes));
            return Ok(responses);
        }

        // Handle the /status command, which is a request for mixer status.
        if osc_msg.path == "/status" {
            let arg1 = OscArg::String("active".to_string());
            let arg2 = OscArg::String("0.0.0.0".to_string()); // Default IP, in actual usage might want the bound IP
            let arg3 = OscArg::String("X32 Emulator".to_string());
            let bytes = OscMessage::serialize_to_bytes("/status", [&arg1, &arg2, &arg3])?;
            responses.push((remote_addr, bytes));
            return Ok(responses);
        }

        // Handle the /renew command, which does nothing but is recognized.
        if osc_msg.path == "/renew" {
            return Ok(responses);
        }

        // Handle the /unsubscribe command, which removes the client from xremote updates.
        if osc_msg.path == "/unsubscribe" {
            self.clients.retain(|&(addr, _)| addr != remote_addr);
            return Ok(responses);
        }

        // If the message has no arguments, it's a request for a value.
        if osc_msg.args.is_empty() {
            if let Some(arg) = self.state.get(&osc_msg.path) {
                let bytes = OscMessage::serialize_to_bytes(&osc_msg.path, [arg])?;
                responses.push((remote_addr, bytes));
            }
        } else {
            // If the message has arguments, it's a command to set a value.
            if let Some(arg) = osc_msg.args.first() {
                self.state.set(&osc_msg.path, arg.clone());

                // Broadcast value change to all xremote clients
                if let Ok(bytes) = OscMessage::serialize_to_bytes(&osc_msg.path, [arg]) {
                    for client in &self.clients {
                        responses.push((client.0, bytes.clone()));
                    }
                }
            }
        }

        Ok(responses)
    }
}
