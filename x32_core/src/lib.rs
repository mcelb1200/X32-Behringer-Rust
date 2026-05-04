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
//! *   **Additional concepts by:** mcelb1200
//! *   **Rust implementation by:** mcelb1200
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
use std::sync::Arc;
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
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    /// Sets a value in the mixer's state.
    pub fn set(&mut self, path: &str, arg: OscArg) {
        self.values.insert(path.to_string(), arg);
    }

    /// Gets a value from the mixer's state.
    pub fn get(&self, path: &str) -> Option<&OscArg> {
        self.values.get(path)
    }
}

/// A struct that emulates the behavior of an X32 mixer.
pub struct Mixer {
    state: MixerState,
    clients: Vec<(SocketAddr, Instant)>,
    // Track active meters per client. Map of (client_addr, meter_idx) -> expiry time
    active_meters: HashMap<(SocketAddr, u8), Instant>,
}

impl Default for Mixer {
    fn default() -> Self {
        Self::new()
    }
}

impl Mixer {
    /// Creates a new `Mixer` with a default, empty state.
    pub fn new() -> Self {
        Self {
            state: MixerState::new(),
            clients: Vec::new(),
            active_meters: HashMap::new(),
        }
    }

    /// Seeds the mixer's state from a vector of OSC command strings.
    pub fn tick(&mut self) -> Vec<(SocketAddr, Arc<[u8]>)> {
        let mut responses = Vec::new();
        let now = Instant::now();

        // Expire old meters
        self.active_meters.retain(|_, expiry| now < *expiry);

        // Generate meter blobs for each active subscription
        for (&(addr, meter_idx), _) in &self.active_meters {
            // Number of floats expected per meter index (based on C code)
            let num_floats = match meter_idx {
                0 => 70, 1 => 96, 2 => 49, 3 => 22, 4 => 82,
                5 => 27, 6 => 4, 7 => 16, 8 => 6, 9 => 32,
                10 => 32, 11 => 5, 12 => 4, 13 => 48, 14 => 80,
                15 => 50, 16 => 48, _ => 0,
            };

            if num_floats > 0 {
                // Generate a dummy blob of 0.0 floats
                let blob_size = num_floats * 4;
                let blob = vec![0u8; blob_size];

                let path = format!("/meters/{}", meter_idx);
                if let Ok(bytes) = OscMessage::serialize_to_bytes(&path, [&OscArg::Blob(blob)]) {
                    responses.push((addr, bytes.into()));
                }
            }
        }

        responses
    }

    /// Seeds the mixer's state from a vector of OSC command strings.
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
                        "i" => arg_value.parse().ok().map(OscArg::Int),
                        "f" => arg_value.parse().ok().map(OscArg::Float),
                        "s" => Some(OscArg::String(arg_value.to_string())),
                        _ => None,
                    };
                    if let Some(a) = arg {
                        self.state.set(path, a);
                    }
                }
            }
        }
    }

    /// Dispatches an incoming OSC message and returns a list of responses to send to specific clients.
    #[allow(clippy::type_complexity)]
    pub fn dispatch(
        &mut self,
        msg: &[u8],
        remote_addr: SocketAddr,
    ) -> Result<Vec<(SocketAddr, Arc<[u8]>)>, Box<dyn std::error::Error>> {
        let osc_msg = OscMessage::from_bytes(msg)?;
        let mut responses = Vec::new();

        // Expire old clients before processing
        let now = Instant::now();
        self.clients.retain(|&(_, expiry)| now < expiry);

        if osc_msg.path == "/xremote" {
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

        // Handle the /info command
        if osc_msg.path == "/info" {
            let arg1 = OscArg::String("V2.07".to_string());
            let arg2 = OscArg::String("X32 Emulator".to_string());
            let arg3 = OscArg::String("X32".to_string());
            let arg4 = OscArg::String("4.06".to_string());
            let bytes = OscMessage::serialize_to_bytes("/info", [&arg1, &arg2, &arg3, &arg4])?;
            responses.push((remote_addr, bytes.into()));
            return Ok(responses);
        }

        // Handle the /status command
        if osc_msg.path == "/status" {
            let arg1 = OscArg::String("active".to_string());
            let arg2 = OscArg::String("0.0.0.0".to_string());
            let arg3 = OscArg::String("X32 Emulator".to_string());
            let bytes = OscMessage::serialize_to_bytes("/status", [&arg1, &arg2, &arg3])?;
            responses.push((remote_addr, bytes.into()));
            return Ok(responses);
        }

        // Handle the /renew command
        if osc_msg.path == "/renew" {
            for client in &mut self.clients {
                if client.0 == remote_addr {
                    client.1 = now + Duration::from_secs(10);
                }
            }
            return Ok(responses);
        }

        // Handle the /unsubscribe command
        if osc_msg.path == "/unsubscribe" {
            self.clients.retain(|&(addr, _)| addr != remote_addr);
            return Ok(responses);
        }

        // Handle /meters subscriptions
        if osc_msg.path.starts_with("/meters/") {
            if let Ok(meter_idx) = osc_msg.path[8..].parse::<u8>() {
                if meter_idx <= 16 {
                    self.active_meters.insert((remote_addr, meter_idx), now + Duration::from_secs(10));
                }
            }
            return Ok(responses);
        }

        // Handle system administration commands: /copy, /add, /load, /save, /delete
        if osc_msg.path == "/copy" {
            let mut success = false;
            if osc_msg.args.len() >= 4 {
                if let (
                    OscArg::String(item_type),
                    OscArg::Int(src_idx),
                    OscArg::Int(dst_idx),
                    OscArg::Int(mask),
                ) = (
                    &osc_msg.args[0],
                    &osc_msg.args[1],
                    &osc_msg.args[2],
                    &osc_msg.args[3],
                ) {
                    if item_type == "libchan" && *src_idx >= 0 && *src_idx < 32 && *dst_idx >= 0 && *dst_idx < 32 {
                        let src_prefix = format!("/ch/{:02}/", src_idx + 1);
                        let dst_prefix = format!("/ch/{:02}/", dst_idx + 1);

                        // C_CONFIG = 0x0002
                        // C_HA = 0x0001
                        // C_GATE = 0x0004
                        // C_DYN = 0x0008
                        // C_EQ = 0x0010
                        // C_SEND = 0x0020

                        let copy_config = (mask & 0x0002) != 0 || *mask == -1;
                        let copy_ha     = (mask & 0x0001) != 0 || *mask == -1;
                        let copy_gate   = (mask & 0x0004) != 0 || *mask == -1;
                        let copy_dyn    = (mask & 0x0008) != 0 || *mask == -1;
                        let copy_eq     = (mask & 0x0010) != 0 || *mask == -1;
                        let copy_send   = (mask & 0x0020) != 0 || *mask == -1;

                        // We will collect keys to clone to avoid borrow checker issues with mut state
                        let mut to_copy = Vec::new();
                        for (key, val) in self.state.values.iter() {
                            if key.starts_with(&src_prefix) {
                                let suffix = &key[src_prefix.len()..];

                                let should_copy = if suffix.starts_with("config/") { copy_config }
                                else if suffix.starts_with("preamp/") { copy_ha }
                                else if suffix.starts_with("gate/") { copy_gate }
                                else if suffix.starts_with("dyn/") { copy_dyn }
                                else if suffix.starts_with("eq/") { copy_eq }
                                else if suffix.starts_with("mix/") { copy_send } // mix includes sends, panning, fader
                                else { *mask == -1 }; // copy all if mask is -1

                                if should_copy {
                                    let new_key = format!("{}{}", dst_prefix, suffix);
                                    to_copy.push((new_key, val.clone()));
                                }
                            }
                        }

                        for (k, v) in to_copy {
                            self.state.set(&k, v.clone());
                            // Need to broadcast to all clients
                            if let Ok(b) = OscMessage::serialize_to_bytes(&k, [&v]) {
                                let arc_b: Arc<[u8]> = b.into();
                                for client in &self.clients {
                                    responses.push((client.0, arc_b.clone()));
                                }
                            }
                        }
                        success = true;
                    }
                }
            }

            let arg_type = osc_msg.args.first().cloned().unwrap_or(OscArg::String("libchan".to_string()));
            let arg_res = OscArg::Int(if success { 1 } else { 0 });
            let bytes = OscMessage::serialize_to_bytes(&osc_msg.path, [&arg_type, &arg_res])?;
            responses.push((remote_addr, bytes.into()));
            return Ok(responses);
        }

        if osc_msg.path == "/save" {
            let mut success = false;
            if osc_msg.args.len() >= 4 {
                if let (
                    OscArg::String(item_type),
                    OscArg::Int(idx),
                    OscArg::String(name),
                    OscArg::String(note),
                ) = (
                    &osc_msg.args[0],
                    &osc_msg.args[1],
                    &osc_msg.args[2],
                    &osc_msg.args[3],
                ) {
                    if item_type == "scene" || item_type == "snippet" {
                        let name_path = format!("/-show/showfile/{}/{:03}/name", item_type, idx);
                        let note_path = format!("/-show/showfile/{}/{:03}/note", item_type, idx);

                        self.state.set(&name_path, OscArg::String(name.clone()));
                        self.state.set(&note_path, OscArg::String(note.clone()));

                        if let Ok(b) = OscMessage::serialize_to_bytes(&name_path, [&OscArg::String(name.clone())]) {
                            let arc_b: Arc<[u8]> = b.into();
                            for client in &self.clients { responses.push((client.0, arc_b.clone())); }
                        }
                        if let Ok(b) = OscMessage::serialize_to_bytes(&note_path, [&OscArg::String(note.clone())]) {
                            let arc_b: Arc<[u8]> = b.into();
                            for client in &self.clients { responses.push((client.0, arc_b.clone())); }
                        }
                        success = true;
                    }
                }
            }
            let arg_type = osc_msg.args.first().cloned().unwrap_or(OscArg::String("scene".to_string()));
            let arg_res = OscArg::Int(if success { 1 } else { 0 });
            let bytes = OscMessage::serialize_to_bytes(&osc_msg.path, [&arg_type, &arg_res])?;
            responses.push((remote_addr, bytes.into()));
            return Ok(responses);
        }

        if osc_msg.path == "/add"
            || osc_msg.path == "/load"
            || osc_msg.path == "/delete"
        {
            if let Some(OscArg::String(ref item_type)) = osc_msg.args.first() {
                let arg1 = OscArg::String(item_type.clone());
                let arg2 = OscArg::Int(1);
                let bytes = OscMessage::serialize_to_bytes(&osc_msg.path, [&arg1, &arg2])?;
                responses.push((remote_addr, bytes.into()));
            }
            return Ok(responses);
        }

        // If the message has no arguments, it's a request for a value.
        if osc_msg.args.is_empty() {
            if let Some(arg) = self.state.get(&osc_msg.path) {
                let bytes = OscMessage::serialize_to_bytes(&osc_msg.path, [arg])?;
                responses.push((remote_addr, bytes.into()));
            }
        } else {
            // If the message has arguments, it's a command to set a value.
            if let Some(arg) = osc_msg.args.first() {
                self.state.set(&osc_msg.path, arg.clone());

                // Broadcast value change to all xremote clients
                if let Ok(bytes) = OscMessage::serialize_to_bytes(&osc_msg.path, [arg]) {
                    let arc_bytes: Arc<[u8]> = bytes.into();
                    for client in &self.clients {
                        responses.push((client.0, arc_bytes.clone()));
                    }
                }
            }
        }

        Ok(responses)
    }
}
