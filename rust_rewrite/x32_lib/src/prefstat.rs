//! # Preferences & Status Module
//!
//! Controls the preferences and status settings on the X32/M32.
//!
//! This module provides functions for controlling various console-wide settings and monitoring status.

use crate::common::On;
use osc_lib::OscArg;

/// Represents the clock rate of the console.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum ClockRate {
    Rate48k = 0,
    Rate44k,
}

impl ClockRate {
    pub fn from_id(id: u8) -> Option<Self> {
        match id {
            0 => Some(ClockRate::Rate48k),
            1 => Some(ClockRate::Rate44k),
            _ => None,
        }
    }
}

/// Represents the clock source for the console.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum ClockSource {
    Internal = 0,
    Aes50A,
    Aes50B,
    Card,
}

impl ClockSource {
    pub fn from_id(id: u8) -> Option<Self> {
        match id {
            0 => Some(ClockSource::Internal),
            1 => Some(ClockSource::Aes50A),
            2 => Some(ClockSource::Aes50B),
            3 => Some(ClockSource::Card),
            _ => None,
        }
    }
}

/// Sets the clock rate of the console.
///
/// # Arguments
///
/// * `rate` - The clock rate to set.
pub fn set_clock_rate(rate: ClockRate) -> (String, Vec<OscArg>) {
    let address = "/-prefs/clockrate".to_string();
    let args = vec![OscArg::Int(rate as i32)];
    (address, args)
}

/// Sets the clock source for the console.
///
/// # Arguments
///
/// * `source` - The clock source to set.
pub fn set_clock_source(source: ClockSource) -> (String, Vec<OscArg>) {
    let address = "/-prefs/clocksource".to_string();
    let args = vec![OscArg::Int(source as i32)];
    (address, args)
}

/// Sets the name of the console.
///
/// # Arguments
///
/// * `name` - The name to set.
///
/// # Example
///
/// ```
/// use x32_lib::prefstat;
/// use osc_lib::OscArg;
///
/// let (address, args) = prefstat::set_name("My Console");
/// assert_eq!(address, "/-prefs/name");
/// assert_eq!(args, vec![OscArg::String("My Console".to_string())]);
/// ```
pub fn set_name(name: &str) -> (String, Vec<OscArg>) {
    let address = "/-prefs/name".to_string();
    let args = vec![OscArg::String(name.to_string())];
    (address, args)
}

/// Turns the solo bus on.
pub fn solo_on() -> (String, Vec<OscArg>) {
    let address = "/-stat/solo".to_string();
    let args = vec![OscArg::Int(1)];
    (address, args)
}

/// Turns the solo bus off.
pub fn solo_off() -> (String, Vec<OscArg>) {
    let address = "/-stat/solo".to_string();
    let args = vec![OscArg::Int(0)];
    (address, args)
}

/// Sets the solo state for a specific channel.
///
/// # Arguments
///
/// * `channel` - The channel ID (1-80).
/// * `on` - The solo state (`On::On` or `On::Off`).
///
/// # Example
///
/// ```
/// use x32_lib::prefstat;
/// use x32_lib::common::On;
/// use osc_lib::OscArg;
///
/// let (address, args) = prefstat::set_solo_channel(1, On::On);
/// assert_eq!(address, "/-stat/solosw/1");
/// assert_eq!(args, vec![OscArg::Int(1)]);
/// ```
pub fn set_solo_channel(channel: u8, on: On) -> (String, Vec<OscArg>) {
    let address = format!("/-stat/solosw/{}", channel);
    let args = vec![OscArg::Int(on as i32)];
    (address, args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_name() {
        let (address, args) = set_name("My Console");
        assert_eq!(address, "/-prefs/name");
        assert_eq!(args, vec![OscArg::String("My Console".to_string())]);
    }

    #[test]
    fn test_solo_on() {
        let (address, args) = solo_on();
        assert_eq!(address, "/-stat/solo");
        assert_eq!(args, vec![OscArg::Int(1)]);
    }

    #[test]
    fn test_set_solo_channel() {
        let (address, args) = set_solo_channel(1, On::On);
        assert_eq!(address, "/-stat/solosw/1");
        assert_eq!(args, vec![OscArg::Int(1)]);
    }
}
