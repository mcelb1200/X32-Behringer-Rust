//! # Auxin Module
//!
//! Controls the 8 auxiliary inputs on the X32/M32.
//!
//! This module provides functions for controlling various aspects of the auxiliary inputs,
//! including configuration, EQ, and mix settings.

use crate::common::{Color, EqType, On};
use osc_lib::OscArg;

// Config
/// Sets the name for a specific auxiliary input.
///
/// # Arguments
///
/// * `auxin_id` - The ID of the auxiliary input (1-8).
/// * `name` - The name to set.
///
/// # Example
///
/// ```
/// use x32_lib::auxin;
/// use osc_lib::OscArg;
///
/// let (address, args) = auxin::set_name(1, "CD Player");
/// assert_eq!(address, "/auxin/01/config/name");
/// assert_eq!(args, vec![OscArg::String("CD Player".to_string())]);
/// ```
pub fn set_name(auxin_id: u8, name: &str) -> (String, Vec<OscArg>) {
    let address = format!("/auxin/{:02}/config/name", auxin_id);
    let args = vec![OscArg::String(name.to_string())];
    (address, args)
}

/// Sets the color for a specific auxiliary input.
///
/// # Arguments
///
/// * `auxin_id` - The ID of the auxiliary input (1-8).
/// * `color` - The color to set.
///
/// # Example
///
/// ```
/// use x32_lib::auxin;
/// use x32_lib::common::Color;
/// use osc_lib::OscArg;
///
/// let (address, args) = auxin::set_color(1, Color::Red);
/// assert_eq!(address, "/auxin/01/config/color");
/// assert_eq!(args, vec![OscArg::Int(1)]);
/// ```
pub fn set_color(auxin_id: u8, color: Color) -> (String, Vec<OscArg>) {
    let address = format!("/auxin/{:02}/config/color", auxin_id);
    let args = vec![OscArg::Int(color as i32)];
    (address, args)
}

// EQ
/// Sets the EQ band type for a specific auxiliary input.
///
/// # Arguments
///
/// * `auxin_id` - The ID of the auxiliary input (1-8).
/// * `band` - The EQ band (1-4).
/// * `eq_type` - The EQ type.
///
/// # Example
///
/// ```
/// use x32_lib::auxin;
/// use x32_lib::common::EqType;
/// use osc_lib::OscArg;
///
/// let (address, args) = auxin::set_eq_band_type(1, 1, EqType::Peq);
/// assert_eq!(address, "/auxin/01/eq/1/type");
/// assert_eq!(args, vec![OscArg::Int(2)]);
/// ```
pub fn set_eq_band_type(auxin_id: u8, band: u8, eq_type: EqType) -> (String, Vec<OscArg>) {
    let address = format!("/auxin/{:02}/eq/{}/type", auxin_id, band);
    let args = vec![OscArg::Int(eq_type as i32)];
    (address, args)
}

// Mix
/// Sets the fader level for a specific auxiliary input.
///
/// # Arguments
///
/// * `auxin_id` - The ID of the auxiliary input (1-8).
/// * `level` - The fader level (0.0 to 1.0).
///
/// # Example
///
/// ```
/// use x32_lib::auxin;
/// use osc_lib::OscArg;
///
/// let (address, args) = auxin::set_fader(1, 0.75);
/// assert_eq!(address, "/auxin/01/mix/fader");
/// assert_eq!(args, vec![OscArg::Float(0.75)]);
/// ```
pub fn set_fader(auxin_id: u8, level: f32) -> (String, Vec<OscArg>) {
    let address = format!("/auxin/{:02}/mix/fader", auxin_id);
    let args = vec![OscArg::Float(level)];
    (address, args)
}

/// Sets the mute state for a specific auxiliary input.
///
/// # Arguments
///
/// * `auxin_id` - The ID of the auxiliary input (1-8).
/// * `on` - The mute state (`On::On` for muted, `On::Off` for unmuted).
///
/// # Example
///
/// ```
/// use x32_lib::auxin;
/// use x32_lib::common::On;
/// use osc_lib::OscArg;
///
/// let (address, args) = auxin::set_on(1, On::On);
/// assert_eq!(address, "/auxin/01/mix/on");
/// assert_eq!(args, vec![OscArg::Int(1)]);
/// ```
pub fn set_on(auxin_id: u8, on: On) -> (String, Vec<OscArg>) {
    let address = format!("/auxin/{:02}/mix/on", auxin_id);
    let args = vec![OscArg::Int(on as i32)];
    (address, args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_fader() {
        let (address, args) = set_fader(1, 0.75);
        assert_eq!(address, "/auxin/01/mix/fader");
        assert_eq!(args, vec![OscArg::Float(0.75)]);
    }

    #[test]
    fn test_set_on() {
        let (address, args) = set_on(1, On::On);
        assert_eq!(address, "/auxin/01/mix/on");
        assert_eq!(args, vec![OscArg::Int(1)]);
    }
}
