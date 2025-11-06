//! # Channel Module
//!
//! Controls the 32 input channels on the X32/M32.
//!
//! This module provides functions for controlling various aspects of the input channels,
//! including configuration, preamp, gate, dynamics, EQ, and mix settings.

use crate::common::{Color, EqType, InsertPosition, InsertSelection, On, CommandFlags, CommandFormat, CommandValue, X32Command};
use osc_lib::OscArg;

// Config
/// Sets the name for a specific channel.
///
/// # Arguments
///
/// * `channel_id` - The ID of the channel (1-32).
/// * `name` - The name to set.
///
/// # Example
///
/// ```
/// use x32_lib::channel;
/// use osc_lib::OscArg;
///
/// let (address, args) = channel::set_name(1, "Vocal");
/// assert_eq!(address, "/ch/01/config/name");
/// assert_eq!(args, vec![OscArg::String("Vocal".to_string())]);
/// ```
pub fn set_name(channel_id: u8, name: &str) -> (String, Vec<OscArg>) {
    let address = format!("/ch/{:02}/config/name", channel_id);
    let args = vec![OscArg::String(name.to_string())];
    (address, args)
}

/// Sets the color for a specific channel.
///
/// # Arguments
///
/// * `channel_id` - The ID of the channel (1-32).
/// * `color` - The color to set.
///
/// # Example
///
/// ```
/// use x32_lib::channel;
/// use x32_lib::common::Color;
/// use osc_lib::OscArg;
///
/// let (address, args) = channel::set_color(1, Color::Blue);
/// assert_eq!(address, "/ch/01/config/color");
/// assert_eq!(args, vec![OscArg::Int(4)]);
/// ```
pub fn set_color(channel_id: u8, color: Color) -> (String, Vec<OscArg>) {
    let address = format!("/ch/{:02}/config/color", channel_id);
    let args = vec![OscArg::Int(color as i32)];
    (address, args)
}

// Preamp
/// Sets the input source for a specific channel.
///
/// # Arguments
///
/// * `channel_id` - The ID of the channel (1-32).
/// * `source` - The input source ID.
///
/// # Example
///
/// ```
/// use x32_lib::channel;
/// use osc_lib::OscArg;
///
/// let (address, args) = channel::set_input_source(1, 0);
/// assert_eq!(address, "/ch/01/config/insrc");
/// assert_eq!(args, vec![OscArg::Int(0)]);
/// ```
pub fn set_input_source(channel_id: u8, source: u8) -> (String, Vec<OscArg>) {
    let address = format!("/ch/{:02}/config/insrc", channel_id);
    let args = vec![OscArg::Int(source as i32)];
    (address, args)
}

// Delay
/// Sets the delay on or off for a specific channel.
///
/// # Arguments
///
/// * `channel_id` - The ID of the channel (1-32).
/// * `on` - The delay state (`On::On` or `On::Off`).
///
/// # Example
///
/// ```
/// use x32_lib::channel;
/// use x32_lib::common::On;
/// use osc_lib::OscArg;
///
/// let (address, args) = channel::set_delay_on(1, On::On);
/// assert_eq!(address, "/ch/01/delay/on");
/// assert_eq!(args, vec![OscArg::Int(1)]);
/// ```
pub fn set_delay_on(channel_id: u8, on: On) -> (String, Vec<OscArg>) {
    let address = format!("/ch/{:02}/delay/on", channel_id);
    let args = vec![OscArg::Int(on as i32)];
    (address, args)
}

/// Sets the delay time for a specific channel.
///
/// # Arguments
///
/// * `channel_id` - The ID of the channel (1-32).
/// * `time` - The delay time in milliseconds (0.0 to 500.0).
///
/// # Example
///
/// ```
/// use x32_lib::channel;
/// use osc_lib::OscArg;
///
/// let (address, args) = channel::set_delay_time(1, 100.0);
/// assert_eq!(address, "/ch/01/delay/time");
/// assert_eq!(args, vec![OscArg::Float(100.0)]);
/// ```
pub fn set_delay_time(channel_id: u8, time: f32) -> (String, Vec<OscArg>) {
    let address = format!("/ch/{:02}/delay/time", channel_id);
    let args = vec![OscArg::Float(time)];
    (address, args)
}

// Mix
/// Sets the fader level for a specific channel.
///
/// # Arguments
///
/// * `channel_id` - The ID of the channel (1-32).
/// * `level` - The fader level (0.0 to 1.0).
///
/// # Example
///
/// ```
/// use x32_lib::channel;
/// use osc_lib::OscArg;
///
/// let (address, args) = channel::set_fader(1, 0.75);
/// assert_eq!(address, "/ch/01/mix/fader");
/// assert_eq!(args, vec![OscArg::Float(0.75)]);
/// ```
pub fn set_fader(channel_id: u8, level: f32) -> (String, Vec<OscArg>) {
    let address = format!("/ch/{:02}/mix/fader", channel_id);
    let args = vec![OscArg::Float(level)];
    (address, args)
}

/// Sets the mute state for a specific channel.
///
/// # Arguments
///
/// * `channel_id` - The ID of the channel (1-32).
/// * `on` - The mute state (`On::On` for muted, `On::Off` for unmuted).
///
/// # Example
///
/// ```
/// use x32_lib::channel;
/// use x32_lib::common::On;
/// use osc_lib::OscArg;
///
/// let (address, args) = channel::set_on(1, On::On);
/// assert_eq!(address, "/ch/01/mix/on");
/// assert_eq!(args, vec![OscArg::Int(1)]);
/// ```
pub fn set_on(channel_id: u8, on: On) -> (String, Vec<OscArg>) {
    let address = format!("/ch/{:02}/mix/on", channel_id);
    let args = vec![OscArg::Int(on as i32)];
    (address, args)
}

// Insert
/// Sets the insert on or off for a specific channel.
///
/// # Arguments
///
/// * `channel_id` - The ID of the channel (1-32).
/// * `on` - The insert state (`On::On` or `On::Off`).
///
/// # Example
///
/// ```
/// use x32_lib::channel;
/// use x32_lib::common::On;
/// use osc_lib::OscArg;
///
/// let (address, args) = channel::set_insert_on(1, On::On);
/// assert_eq!(address, "/ch/01/insert/on");
/// assert_eq!(args, vec![OscArg::Int(1)]);
/// ```
pub fn set_insert_on(channel_id: u8, on: On) -> (String, Vec<OscArg>) {
    let address = format!("/ch/{:02}/insert/on", channel_id);
    let args = vec![OscArg::Int(on as i32)];
    (address, args)
}

/// Sets the insert position for a specific channel.
///
/// # Arguments
///
/// * `channel_id` - The ID of the channel (1-32).
/// * `pos` - The insert position (`InsertPosition::Pre` or `InsertPosition::Post`).
///
/// # Example
///
/// ```
/// use x32_lib::channel;
/// use x32_lib::common::InsertPosition;
/// use osc_lib::OscArg;
///
/// let (address, args) = channel::set_insert_pos(1, InsertPosition::Pre);
/// assert_eq!(address, "/ch/01/insert/pos");
/// assert_eq!(args, vec![OscArg::Int(0)]);
/// ```
pub fn set_insert_pos(channel_id: u8, pos: InsertPosition) -> (String, Vec<OscArg>) {
    let address = format!("/ch/{:02}/insert/pos", channel_id);
    let args = vec![OscArg::Int(pos as i32)];
    (address, args)
}

/// Sets the insert selection for a specific channel.
///
/// # Arguments
///
/// * `channel_id` - The ID of the channel (1-32).
/// * `sel` - The insert selection.
///
/// # Example
///
/// ```
/// use x32_lib::channel;
/// use x32_lib::common::InsertSelection;
/// use osc_lib::OscArg;
///
/// let (address, args) = channel::set_insert_sel(1, InsertSelection::Fx1L);
/// assert_eq!(address, "/ch/01/insert/sel");
/// assert_eq!(args, vec![OscArg::Int(1)]);
/// ```
pub fn set_insert_sel(channel_id: u8, sel: InsertSelection) -> (String, Vec<OscArg>) {
    let address = format!("/ch/{:02}/insert/sel", channel_id);
    let args = vec![OscArg::Int(sel as i32)];
    (address, args)
}

// EQ
/// Sets the EQ band type for a specific channel.
///
/// # Arguments
///
/// * `channel_id` - The ID of the channel (1-32).
/// * `band` - The EQ band (1-4).
/// * `eq_type` - The EQ type.
///
/// # Example
///
/// ```
/// use x32_lib::channel;
/// use x32_lib::common::EqType;
/// use osc_lib::OscArg;
///
/// let (address, args) = channel::set_eq_band_type(1, 1, EqType::Peq);
/// assert_eq!(address, "/ch/01/eq/1/type");
/// assert_eq!(args, vec![OscArg::Int(2)]);
/// ```
pub fn set_eq_band_type(channel_id: u8, band: u8, eq_type: EqType) -> (String, Vec<OscArg>) {
    let address = format!("/ch/{:02}/eq/{}/type", channel_id, band);
    let args = vec![OscArg::Int(eq_type as i32)];
    (address, args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_fader() {
        let (address, args) = set_fader(1, 0.75);
        assert_eq!(address, "/ch/01/mix/fader");
        assert_eq!(args, vec![OscArg::Float(0.75)]);
    }

    #[test]
    fn test_set_on() {
        let (address, args) = set_on(1, On::On);
        assert_eq!(address, "/ch/01/mix/on");
        assert_eq!(args, vec![OscArg::Int(1)]);
    }
}