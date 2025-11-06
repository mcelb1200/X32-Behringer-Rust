//! # FX Return Module
//!
//! Controls the 8 FX return channels on the X32/M32.
//!
//! This module provides functions for controlling various aspects of the FX return channels,
//! including configuration, EQ, and mix settings.

use crate::common::{Color, EqType, On};
use osc_lib::OscArg;

// Config
/// Sets the name for a specific FX return channel.
///
/// # Arguments
///
/// * `fxrtn_id` - The ID of the FX return channel (1-8).
/// * `name` - The name to set.
///
/// # Example
///
/// ```
/// use x32_lib::fxrtn;
/// use osc_lib::OscArg;
///
/// let (address, args) = fxrtn::set_name(1, "Reverb");
/// assert_eq!(address, "/fxrtn/01/config/name");
/// assert_eq!(args, vec![OscArg::String("Reverb".to_string())]);
/// ```
pub fn set_name(fxrtn_id: u8, name: &str) -> (String, Vec<OscArg>) {
    let address = format!("/fxrtn/{:02}/config/name", fxrtn_id);
    let args = vec![OscArg::String(name.to_string())];
    (address, args)
}

/// Sets the icon for a specific FX return channel.
///
/// # Arguments
///
/// * `fxrtn_id` - The ID of the FX return channel (1-8).
/// * `icon` - The icon ID.
pub fn set_icon(fxrtn_id: u8, icon: u8) -> (String, Vec<OscArg>) {
    let address = format!("/fxrtn/{:02}/config/icon", fxrtn_id);
    let args = vec![OscArg::Int(icon as i32)];
    (address, args)
}

/// Sets the color for a specific FX return channel.
///
/// # Arguments
///
/// * `fxrtn_id` - The ID of the FX return channel (1-8).
/// * `color` - The color to set.
///
/// # Example
///
/// ```
/// use x32_lib::fxrtn;
/// use x32_lib::common::Color;
/// use osc_lib::OscArg;
///
/// let (address, args) = fxrtn::set_color(1, Color::Red);
/// assert_eq!(address, "/fxrtn/01/config/color");
/// assert_eq!(args, vec![OscArg::Int(1)]);
/// ```
pub fn set_color(fxrtn_id: u8, color: Color) -> (String, Vec<OscArg>) {
    let address = format!("/fxrtn/{:02}/config/color", fxrtn_id);
    let args = vec![OscArg::Int(color as i32)];
    (address, args)
}

// EQ
/// Sets the EQ on or off for a specific FX return channel.
///
/// # Arguments
///
/// * `fxrtn_id` - The ID of the FX return channel (1-8).
/// * `on` - The EQ state (`On::On` or `On::Off`).
pub fn set_eq_on(fxrtn_id: u8, on: On) -> (String, Vec<OscArg>) {
    let address = format!("/fxrtn/{:02}/eq/on", fxrtn_id);
    let args = vec![OscArg::Int(on as i32)];
    (address, args)
}

/// Sets the EQ band type for a specific FX return channel.
///
/// # Arguments
///
/// * `fxrtn_id` - The ID of the FX return channel (1-8).
/// * `band` - The EQ band (1-4).
/// * `eq_type` - The EQ type.
pub fn set_eq_band_type(fxrtn_id: u8, band: u8, eq_type: EqType) -> (String, Vec<OscArg>) {
    let address = format!("/fxrtn/{:02}/eq/{}/type", fxrtn_id, band);
    let args = vec![OscArg::Int(eq_type as i32)];
    (address, args)
}

/// Sets the frequency for a specific EQ band on an FX return channel.
///
/// # Arguments
///
/// * `fxrtn_id` - The ID of the FX return channel (1-8).
/// * `band` - The EQ band (1-4).
/// * `f` - The frequency value.
pub fn set_eq_band_f(fxrtn_id: u8, band: u8, f: f32) -> (String, Vec<OscArg>) {
    let address = format!("/fxrtn/{:02}/eq/{}/f", fxrtn_id, band);
    let args = vec![OscArg::Float(f)];
    (address, args)
}

/// Sets the gain for a specific EQ band on an FX return channel.
///
/// # Arguments
///
/// * `fxrtn_id` - The ID of the FX return channel (1-8).
/// * `band` - The EQ band (1-4).
/// * `g` - The gain value.
pub fn set_eq_band_g(fxrtn_id: u8, band: u8, g: f32) -> (String, Vec<OscArg>) {
    let address = format!("/fxrtn/{:02}/eq/{}/g", fxrtn_id, band);
    let args = vec![OscArg::Float(g)];
    (address, args)
}

/// Sets the Q value for a specific EQ band on an FX return channel.
///
/// # Arguments
///
/// * `fxrtn_id` - The ID of the FX return channel (1-8).
/// * `band` - The EQ band (1-4).
/// * `q` - The Q value.
pub fn set_eq_band_q(fxrtn_id: u8, band: u8, q: f32) -> (String, Vec<OscArg>) {
    let address = format!("/fxrtn/{:02}/eq/{}/q", fxrtn_id, band);
    let args = vec![OscArg::Float(q)];
    (address, args)
}

// Mix
/// Sets the mix on or off for a specific FX return channel.
///
/// # Arguments
///
/// * `fxrtn_id` - The ID of the FX return channel (1-8).
/// * `on` - The mix state (`On::On` or `On::Off`).
pub fn set_mix_on(fxrtn_id: u8, on: On) -> (String, Vec<OscArg>) {
    let address = format!("/fxrtn/{:02}/mix/on", fxrtn_id);
    let args = vec![OscArg::Int(on as i32)];
    (address, args)
}

/// Sets the fader level for a specific FX return channel.
///
/// # Arguments
///
/// * `fxrtn_id` - The ID of the FX return channel (1-8).
/// * `level` - The fader level (0.0 to 1.0).
///
/// # Example
///
/// ```
/// use x32_lib::fxrtn;
/// use osc_lib::OscArg;
///
/// let (address, args) = fxrtn::set_fader(1, 0.75);
/// assert_eq!(address, "/fxrtn/01/mix/fader");
/// assert_eq!(args, vec![OscArg::Float(0.75)]);
/// ```
pub fn set_fader(fxrtn_id: u8, level: f32) -> (String, Vec<OscArg>) {
    let address = format!("/fxrtn/{:02}/mix/fader", fxrtn_id);
    let args = vec![OscArg::Float(level)];
    (address, args)
}

/// Sets the stereo mix on or off for a specific FX return channel.
///
/// # Arguments
///
/// * `fxrtn_id` - The ID of the FX return channel (1-8).
/// * `on` - The stereo mix state (`On::On` or `On::Off`).
pub fn set_st_on(fxrtn_id: u8, on: On) -> (String, Vec<OscArg>) {
    let address = format!("/fxrtn/{:02}/mix/st", fxrtn_id);
    let args = vec![OscArg::Int(on as i32)];
    (address, args)
}

/// Sets the pan for a specific FX return channel.
///
/// # Arguments
///
/// * `fxrtn_id` - The ID of the FX return channel (1-8).
/// * `pan` - The pan value (-1.0 to 1.0).
pub fn set_pan(fxrtn_id: u8, pan: f32) -> (String, Vec<OscArg>) {
    let address = format!("/fxrtn/{:02}/mix/pan", fxrtn_id);
    let args = vec![OscArg::Float(pan)];
    (address, args)
}

/// Sets the mono mix on or off for a specific FX return channel.
///
/// # Arguments
///
/// * `fxrtn_id` - The ID of the FX return channel (1-8).
/// * `on` - The mono mix state (`On::On` or `On::Off`).
pub fn set_mono_on(fxrtn_id: u8, on: On) -> (String, Vec<OscArg>) {
    let address = format!("/fxrtn/{:02}/mix/mono", fxrtn_id);
    let args = vec![OscArg::Int(on as i32)];
    (address, args)
}

/// Sets the mono mix level for a specific FX return channel.
///
/// # Arguments
///
/// * `fxrtn_id` - The ID of the FX return channel (1-8).
/// * `level` - The mono mix level (0.0 to 1.0).
pub fn set_mono_level(fxrtn_id: u8, level: f32) -> (String, Vec<OscArg>) {
    let address = format!("/fxrtn/{:02}/mix/mlevel", fxrtn_id);
    let args = vec![OscArg::Float(level)];
    (address, args)
}

// Bus Send
/// Sets the bus send on or off for a specific FX return channel.
///
/// # Arguments
///
/// * `fxrtn_id` - The ID of the FX return channel (1-8).
/// * `bus_id` - The ID of the bus (1-16).
/// * `on` - The bus send state (`On::On` or `On::Off`).
pub fn set_bus_send_on(fxrtn_id: u8, bus_id: u8, on: On) -> (String, Vec<OscArg>) {
    let address = format!("/fxrtn/{:02}/mix/{:02}/on", fxrtn_id, bus_id);
    let args = vec![OscArg::Int(on as i32)];
    (address, args)
}

/// Sets the bus send level for a specific FX return channel.
///
/// # Arguments
///
/// * `fxrtn_id` - The ID of the FX return channel (1-8).
/// * `bus_id` - The ID of the bus (1-16).
/// * `level` - The bus send level (0.0 to 1.0).
///
/// # Example
///
/// ```
/// use x32_lib::fxrtn;
/// use osc_lib::OscArg;
///
/// let (address, args) = fxrtn::set_bus_send_level(1, 1, 0.5);
/// assert_eq!(address, "/fxrtn/01/mix/01/level");
/// assert_eq!(args, vec![OscArg::Float(0.5)]);
/// ```
pub fn set_bus_send_level(fxrtn_id: u8, bus_id: u8, level: f32) -> (String, Vec<OscArg>) {
    let address = format!("/fxrtn/{:02}/mix/{:02}/level", fxrtn_id, bus_id);
    let args = vec![OscArg::Float(level)];
    (address, args)
}

// Group
/// Assigns a DCA group to a specific FX return channel.
///
/// # Arguments
///
/// * `fxrtn_id` - The ID of the FX return channel (1-8).
/// * `dca_group` - The DCA group ID (1-8).
pub fn set_dca_group(fxrtn_id: u8, dca_group: u8) -> (String, Vec<OscArg>) {
    let address = format!("/fxrtn/{:02}/grp/dca", fxrtn_id);
    let args = vec![OscArg::Int(dca_group as i32)];
    (address, args)
}

/// Assigns a mute group to a specific FX return channel.
///
/// # Arguments
///
/// * `fxrtn_id` - The ID of the FX return channel (1-8).
/// * `mute_group` - The mute group ID (1-8).
pub fn set_mute_group(fxrtn_id: u8, mute_group: u8) -> (String, Vec<OscArg>) {
    let address = format!("/fxrtn/{:02}/grp/mute", fxrtn_id);
    let args = vec![OscArg::Int(mute_group as i32)];
    (address, args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_name() {
        let (address, args) = set_name(1, "FXR 1");
        assert_eq!(address, "/fxrtn/01/config/name");
        assert_eq!(args, vec![OscArg::String("FXR 1".to_string())]);
    }

    #[test]
    fn test_set_fader() {
        let (address, args) = set_fader(1, 0.75);
        assert_eq!(address, "/fxrtn/01/mix/fader");
        assert_eq!(args, vec![OscArg::Float(0.75)]);
    }

    #[test]
    fn test_set_bus_send_level() {
        let (address, args) = set_bus_send_level(1, 1, 0.5);
        assert_eq!(address, "/fxrtn/01/mix/01/level");
        assert_eq!(args, vec![OscArg::Float(0.5)]);
    }
}
