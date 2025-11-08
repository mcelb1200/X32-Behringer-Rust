//! Provides functions for generating OSC commands to control X32/M32 mix buses (1-16).
//!
//! Mix buses are used for creating monitor mixes, subgroup mixes, or feeding effects processors.
//! This module provides functions to control their configuration, dynamics, EQ, and fader levels.
use super::{Command, CommandFlags, CommandType};
use osc_lib::OscArg;

/// A static array of all available commands for the mix bus channels.
pub const XBUS_COMMANDS: &[Command] = &[
    Command { path: "/bus/01/mix/fader", command_type: CommandType::Float, flags: CommandFlags::GET.union(CommandFlags::SET), nodes: None },
    Command { path: "/bus/01/mix/on", command_type: CommandType::Enum, flags: CommandFlags::GET.union(CommandFlags::SET), nodes: None },
    Command { path: "/bus/02/mix/fader", command_type: CommandType::Float, flags: CommandFlags::GET.union(CommandFlags::SET), nodes: None },
    Command { path: "/bus/02/mix/on", command_type: CommandType::Enum, flags: CommandFlags::GET.union(CommandFlags::SET), nodes: None },
    Command { path: "/bus/03/mix/fader", command_type: CommandType::Float, flags: CommandFlags::GET.union(CommandFlags::SET), nodes: None },
    Command { path: "/bus/03/mix/on", command_type: CommandType::Enum, flags: CommandFlags::GET.union(CommandFlags::SET), nodes: None },
    Command { path: "/bus/04/mix/fader", command_type: CommandType::Float, flags: CommandFlags::GET.union(CommandFlags::SET), nodes: None },
    Command { path: "/bus/04/mix/on", command_type: CommandType::Enum, flags: CommandFlags::GET.union(CommandFlags::SET), nodes: None },
    Command { path: "/bus/05/mix/fader", command_type: CommandType::Float, flags: CommandFlags::GET.union(CommandFlags::SET), nodes: None },
    Command { path: "/bus/05/mix/on", command_type: CommandType::Enum, flags: CommandFlags::GET.union(CommandFlags::SET), nodes: None },
    Command { path: "/bus/06/mix/fader", command_type: CommandType::Float, flags: CommandFlags::GET.union(CommandFlags::SET), nodes: None },
    Command { path: "/bus/06/mix/on", command_type: CommandType::Enum, flags: CommandFlags::GET.union(CommandFlags::SET), nodes: None },
    Command { path: "/bus/07/mix/fader", command_type: CommandType::Float, flags: CommandFlags::GET.union(CommandFlags::SET), nodes: None },
    Command { path: "/bus/07/mix/on", command_type: CommandType::Enum, flags: CommandFlags::GET.union(CommandFlags::SET), nodes: None },
    Command { path: "/bus/08/mix/fader", command_type: CommandType::Float, flags: CommandFlags::GET.union(CommandFlags::SET), nodes: None },
    Command { path: "/bus/08/mix/on", command_type: CommandType::Enum, flags: CommandFlags::GET.union(CommandFlags::SET), nodes: None },
];


// --- Address String Getters ---

/// Returns the OSC address for a bus channel's name.
pub fn name(channel_num: u8) -> String {
    format!("/bus/{:02}/config/name", channel_num)
}

/// Returns the OSC address for a bus channel's color.
pub fn color(channel_num: u8) -> String {
    format!("/bus/{:02}/config/color", channel_num)
}

/// Returns the OSC address for a bus channel's dynamics on/off state.
pub fn dyn_on(channel_num: u8) -> String {
    format!("/bus/{:02}/dyn/on", channel_num)
}

/// Returns the OSC address for a bus channel's EQ band type.
pub fn eq_band_type(channel_num: u8, band: u8) -> String {
    format!("/bus/{:02}/eq/{}/type", channel_num, band)
}

/// Returns the OSC address for a bus channel's fader level.
pub fn fader_level(channel_num: u8) -> String {
    format!("/bus/{:02}/mix/fader", channel_num)
}

/// Returns the OSC address for a bus channel's on/off (mute) state.
pub fn on(channel_num: u8) -> String {
    format!("/bus/{:02}/mix/on", channel_num)
}


// --- OSC Message Setters ---

/// Creates an OSC message to set the name of a bus channel.
///
/// # Arguments
///
/// * `channel_num` - The bus channel number (1-16).
/// * `name` - The new name for the channel.
///
/// ```
/// use x32_lib::command::bus;
///
/// let (address, args) = bus::set_name(1, "Test");
/// assert_eq!(address, "/bus/01/config/name");
/// assert_eq!(args, vec![osc_lib::OscArg::String("Test".to_string())]);
/// ```
pub fn set_name(channel_num: u8, name: &str) -> (String, Vec<OscArg>) {
    (self::name(channel_num), vec![OscArg::String(name.to_string())])
}

/// Creates an OSC message to set the color of a bus channel.
///
/// # Arguments
///
/// * `channel_num` - The bus channel number (1-16).
/// * `color` - The new color for the channel (0-15).
///
/// ```
/// use x32_lib::command::bus;
///
/// let (address, args) = bus::set_color(1, 2);
/// assert_eq!(address, "/bus/01/config/color");
/// assert_eq!(args, vec![osc_lib::OscArg::Int(2)]);
/// ```
pub fn set_color(channel_num: u8, color: i32) -> (String, Vec<OscArg>) {
    (self::color(channel_num), vec![OscArg::Int(color)])
}

/// Creates an OSC message to set the dyn on state of a bus channel.
///
/// # Arguments
///
/// * `channel_num` - The bus channel number (1-16).
/// * `on` - The new dyn on state for the channel (0 for Off, 1 for On).
///
/// ```
/// use x32_lib::command::bus;
///
/// let (address, args) = bus::set_dyn_on(1, 1);
/// assert_eq!(address, "/bus/01/dyn/on");
/// assert_eq!(args, vec![osc_lib::OscArg::Int(1)]);
/// ```
pub fn set_dyn_on(channel_num: u8, on: i32) -> (String, Vec<OscArg>) {
    (self::dyn_on(channel_num), vec![OscArg::Int(on)])
}

/// Creates an OSC message to set the eq band type of a bus channel.
///
/// # Arguments
///
/// * `channel_num` - The bus channel number (1-16).
/// * `band` - The eq band (1-6).
/// * `eq_type` - The new eq type for the channel (0-5).
///
/// ```
/// use x32_lib::command::bus;
///
/// let (address, args) = bus::set_eq_band_type(1, 1, 2);
/// assert_eq!(address, "/bus/01/eq/1/type");
/// assert_eq!(args, vec![osc_lib::OscArg::Int(2)]);
/// ```
pub fn set_eq_band_type(channel_num: u8, band: u8, eq_type: i32) -> (String, Vec<OscArg>) {
    (self::eq_band_type(channel_num, band), vec![OscArg::Int(eq_type)])
}

/// Creates an OSC message to set the fader level of a bus channel.
///
/// # Arguments
///
/// * `channel_num` - The bus channel number (1-16).
/// * `level` - The new fader level (0.0-1.0).
///
/// ```
/// use x32_lib::command::bus;
///
/// let (address, args) = bus::set_fader(1, 0.75);
/// assert_eq!(address, "/bus/01/mix/fader");
/// assert_eq!(args, vec![osc_lib::OscArg::Float(0.75)]);
/// ```
pub fn set_fader(channel_num: u8, level: f32) -> (String, Vec<OscArg>) {
    (fader_level(channel_num), vec![OscArg::Float(level)])
}

/// Creates an OSC message to set the on state of a bus channel.
///
/// # Arguments
///
/// * `channel_num` - The bus channel number (1-16).
/// * `on` - The new on state for the channel (0 for Off, 1 for On).
///
/// ```
/// use x32_lib::command::bus;
///
/// let (address, args) = bus::set_on(1, 1);
/// assert_eq!(address, "/bus/01/mix/on");
/// assert_eq!(args, vec![osc_lib::OscArg::Int(1)]);
/// ```
pub fn set_on(channel_num: u8, on: i32) -> (String, Vec<OscArg>) {
    (self::on(channel_num), vec![OscArg::Int(on)])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_fader() {
        let (address, args) = set_fader(1, 0.5);
        assert_eq!(address, "/bus/01/mix/fader");
        assert_eq!(args, vec![OscArg::Float(0.5)]);
    }

    #[test]
    fn test_set_on() {
        let (address, args) = set_on(2, 1);
        assert_eq!(address, "/bus/02/mix/on");
        assert_eq!(args, vec![OscArg::Int(1)]);
    }
}
