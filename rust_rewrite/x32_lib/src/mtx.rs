//! # Matrix Module
//!
//! Controls the 6 matrix outputs on the X32/M32.
//!
//! This module provides functions for controlling various aspects of the matrix outputs,
//! including configuration, dynamics, EQ, and mix settings.

use crate::common::{Color, EqType, On};
use osc_lib::OscArg;

// Config
/// Sets the name for a specific matrix output.
///
/// # Arguments
///
/// * `mtx_id` - The ID of the matrix output (1-6).
/// * `name` - The name to set.
///
/// # Example
///
/// ```
/// use x32_lib::mtx;
/// use osc_lib::OscArg;
///
/// let (address, args) = mtx::set_name(1, "Side Fills");
/// assert_eq!(address, "/mtx/01/config/name");
/// assert_eq!(args, vec![OscArg::String("Side Fills".to_string())]);
/// ```
pub fn set_name(mtx_id: u8, name: &str) -> (String, Vec<OscArg>) {
    let address = format!("/mtx/{:02}/config/name", mtx_id);
    let args = vec![OscArg::String(name.to_string())];
    (address, args)
}

/// Sets the color for a specific matrix output.
///
/// # Arguments
///
/// * `mtx_id` - The ID of the matrix output (1-6).
/// * `color` - The color to set.
///
/// # Example
///
/// ```
/// use x32_lib::mtx;
/// use x32_lib::common::Color;
/// use osc_lib::OscArg;
///
/// let (address, args) = mtx::set_color(1, Color::Cyan);
/// assert_eq!(address, "/mtx/01/config/color");
/// assert_eq!(args, vec![OscArg::Int(6)]);
/// ```
pub fn set_color(mtx_id: u8, color: Color) -> (String, Vec<OscArg>) {
    let address = format!("/mtx/{:02}/config/color", mtx_id);
    let args = vec![OscArg::Int(color as i32)];
    (address, args)
}

// Dynamics
/// Sets the dynamics on or off for a specific matrix output.
///
/// # Arguments
///
/// * `mtx_id` - The ID of the matrix output (1-6).
/// * `on` - The dynamics state (`On::On` or `On::Off`).
pub fn set_dyn_on(mtx_id: u8, on: On) -> (String, Vec<OscArg>) {
    let address = format!("/mtx/{:02}/dyn/on", mtx_id);
    let args = vec![OscArg::Int(on as i32)];
    (address, args)
}

// EQ
/// Sets the EQ band type for a specific matrix output.
///
/// # Arguments
///
/// * `mtx_id` - The ID of the matrix output (1-6).
/// * `band` - The EQ band (1-6).
/// * `eq_type` - The EQ type.
pub fn set_eq_band_type(mtx_id: u8, band: u8, eq_type: EqType) -> (String, Vec<OscArg>) {
    let address = format!("/mtx/{:02}/eq/{}/type", mtx_id, band);
    let args = vec![OscArg::Int(eq_type as i32)];
    (address, args)
}

// Mix
/// Sets the fader level for a specific matrix output.
///
/// # Arguments
///
/// * `mtx_id` - The ID of the matrix output (1-6).
/// * `level` - The fader level (0.0 to 1.0).
///
/// # Example
///
/// ```
/// use x32_lib::mtx;
/// use osc_lib::OscArg;
///
/// let (address, args) = mtx::set_fader(1, 0.75);
/// assert_eq!(address, "/mtx/01/mix/fader");
/// assert_eq!(args, vec![OscArg::Float(0.75)]);
/// ```
pub fn set_fader(mtx_id: u8, level: f32) -> (String, Vec<OscArg>) {
    let address = format!("/mtx/{:02}/mix/fader", mtx_id);
    let args = vec![OscArg::Float(level)];
    (address, args)
}

/// Sets the mute state for a specific matrix output.
///
/// # Arguments
///
/// * `mtx_id` - The ID of the matrix output (1-6).
/// * `on` - The mute state (`On::On` for muted, `On::Off` for unmuted).
///
/// # Example
///
/// ```
/// use x32_lib::mtx;
/// use x32_lib::common::On;
/// use osc_lib::OscArg;
///
/// let (address, args) = mtx::set_on(1, On::On);
/// assert_eq!(address, "/mtx/01/mix/on");
/// assert_eq!(args, vec![OscArg::Int(1)]);
/// ```
pub fn set_on(mtx_id: u8, on: On) -> (String, Vec<OscArg>) {
    let address = format!("/mtx/{:02}/mix/on", mtx_id);
    let args = vec![OscArg::Int(on as i32)];
    (address, args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_fader() {
        let (address, args) = set_fader(1, 0.75);
        assert_eq!(address, "/mtx/01/mix/fader");
        assert_eq!(args, vec![OscArg::Float(0.75)]);
    }

    #[test]
    fn test_set_on() {
        let (address, args) = set_on(1, On::On);
        assert_eq!(address, "/mtx/01/mix/on");
        assert_eq!(args, vec![OscArg::Int(1)]);
    }
}
