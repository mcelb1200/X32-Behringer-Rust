//! # Main Bus Module
//!
//! Controls the main stereo and mono/center buses on the X32/M32.
//!
//! This module provides functions for controlling the main output buses.

use crate::common::{Color, EqType, On};
use osc_lib::OscArg;

// Config
/// Sets the name for the main stereo bus.
///
/// # Arguments
///
/// * `name` - The name to set.
pub fn set_st_name(name: &str) -> (String, Vec<OscArg>) {
    let address = "/main/st/config/name".to_string();
    let args = vec![OscArg::String(name.to_string())];
    (address, args)
}

/// Sets the color for the main stereo bus.
///
/// # Arguments
///
/// * `color` - The color to set.
pub fn set_st_color(color: Color) -> (String, Vec<OscArg>) {
    let address = "/main/st/config/color".to_string();
    let args = vec![OscArg::Int(color as i32)];
    (address, args)
}

/// Sets the name for the mono/center bus.
///
/// # Arguments
///
/// * `name` - The name to set.
pub fn set_mono_name(name: &str) -> (String, Vec<OscArg>) {
    let address = "/main/m/config/name".to_string();
    let args = vec![OscArg::String(name.to_string())];
    (address, args)
}

/// Sets the color for the mono/center bus.
///
/// # Arguments
///
/// * `color` - The color to set.
pub fn set_mono_color(color: Color) -> (String, Vec<OscArg>) {
    let address = "/main/m/config/color".to_string();
    let args = vec![OscArg::Int(color as i32)];
    (address, args)
}

// Dynamics
/// Sets the dynamics on or off for the main stereo bus.
///
/// # Arguments
///
/// * `on` - The dynamics state (`On::On` or `On::Off`).
pub fn set_st_dyn_on(on: On) -> (String, Vec<OscArg>) {
    let address = "/main/st/dyn/on".to_string();
    let args = vec![OscArg::Int(on as i32)];
    (address, args)
}

/// Sets the dynamics on or off for the mono/center bus.
///
/// # Arguments
///
/// * `on` - The dynamics state (`On::On` or `On::Off`).
pub fn set_mono_dyn_on(on: On) -> (String, Vec<OscArg>) {
    let address = "/main/m/dyn/on".to_string();
    let args = vec![OscArg::Int(on as i32)];
    (address, args)
}

// EQ
/// Sets the EQ band type for the main stereo bus.
///
/// # Arguments
///
/// * `band` - The EQ band (1-6).
/// * `eq_type` - The EQ type.
pub fn set_st_eq_band_type(band: u8, eq_type: EqType) -> (String, Vec<OscArg>) {
    let address = format!("/main/st/eq/{}/type", band);
    let args = vec![OscArg::Int(eq_type as i32)];
    (address, args)
}

/// Sets the EQ band type for the mono/center bus.
///
/// # Arguments
///
/// * `band` - The EQ band (1-4).
/// * `eq_type` - The EQ type.
pub fn set_mono_eq_band_type(band: u8, eq_type: EqType) -> (String, Vec<OscArg>) {
    let address = format!("/main/m/eq/{}/type", band);
    let args = vec![OscArg::Int(eq_type as i32)];
    (address, args)
}

// Mix
/// Sets the fader level for the main stereo bus.
///
/// # Arguments
///
/// * `level` - The fader level (0.0 to 1.0).
pub fn set_st_fader(level: f32) -> (String, Vec<OscArg>) {
    let address = "/main/st/mix/fader".to_string();
    let args = vec![OscArg::Float(level)];
    (address, args)
}

/// Sets the mute state for the main stereo bus.
///
/// # Arguments
///
/// * `on` - The mute state (`On::On` for muted, `On::Off` for unmuted).
pub fn set_st_on(on: On) -> (String, Vec<OscArg>) {
    let address = "/main/st/mix/on".to_string();
    let args = vec![OscArg::Int(on as i32)];
    (address, args)
}

/// Sets the fader level for the mono/center bus.
///
/// # Arguments
///
/// * `level` - The fader level (0.0 to 1.0).
pub fn set_mono_fader(level: f32) -> (String, Vec<OscArg>) {
    let address = "/main/m/mix/fader".to_string();
    let args = vec![OscArg::Float(level)];
    (address, args)
}

/// Sets the mute state for the mono/center bus.
///
/// # Arguments
///
/// * `on` - The mute state (`On::On` for muted, `On::Off` for unmuted).
pub fn set_mono_on(on: On) -> (String, Vec<OscArg>) {
    let address = "/main/m/mix/on".to_string();
    let args = vec![OscArg::Int(on as i32)];
    (address, args)
}
