//! # Headamp Module
//!
//! Controls the 128 headamps (preamps) on the X32/M32.
//!
//! This module provides functions for controlling the gain and phantom power of the headamps.

use crate::common::On;
use osc_lib::OscArg;

/// Sets the gain for a specific headamp.
///
/// # Arguments
///
/// * `headamp_id` - The ID of the headamp (0-127).
/// * `gain` - The gain level (-12.0 to 60.0).
///
/// # Example
///
/// ```
/// use x32_lib::headamp;
/// use osc_lib::OscArg;
///
/// let (address, args) = headamp::set_gain(0, 0.5);
/// assert_eq!(address, "/headamp/000/gain");
/// assert_eq!(args, vec![OscArg::Float(0.5)]);
/// ```
pub fn set_gain(headamp_id: u8, gain: f32) -> (String, Vec<OscArg>) {
    let address = format!("/headamp/{:03}/gain", headamp_id);
    let args = vec![OscArg::Float(gain)];
    (address, args)
}

/// Sets the phantom power for a specific headamp.
///
/// # Arguments
///
/// * `headamp_id` - The ID of the headamp (0-127).
/// * `on` - The phantom power state (`On::On` or `On::Off`).
///
/// # Example
///
/// ```
/// use x32_lib::headamp;
/// use x32_lib::common::On;
/// use osc_lib::OscArg;
///
/// let (address, args) = headamp::set_phantom(0, On::On);
/// assert_eq!(address, "/headamp/000/phantom");
/// assert_eq!(args, vec![OscArg::Int(1)]);
/// ```
pub fn set_phantom(headamp_id: u8, on: On) -> (String, Vec<OscArg>) {
    let address = format!("/headamp/{:03}/phantom", headamp_id);
    let args = vec![OscArg::Int(on as i32)];
    (address, args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_gain() {
        let (address, args) = set_gain(0, 0.5);
        assert_eq!(address, "/headamp/000/gain");
        assert_eq!(args, vec![OscArg::Float(0.5)]);
    }

    #[test]
    fn test_set_phantom() {
        let (address, args) = set_phantom(0, On::On);
        assert_eq!(address, "/headamp/000/phantom");
        assert_eq!(args, vec![OscArg::Int(1)]);
    }
}
