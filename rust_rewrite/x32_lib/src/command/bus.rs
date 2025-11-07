//! This module provides the command definitions for the X32 bus channels.
use super::{Command, CommandFlags, CommandType};

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

use osc_lib::OscArg;

/// Sets the name of a bus channel.
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
    let address = format!("/bus/{:02}/config/name", channel_num);
    (address, vec![OscArg::String(name.to_string())])
}

/// Sets the color of a bus channel.
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
    let address = format!("/bus/{:02}/config/color", channel_num);
    (address, vec![OscArg::Int(color)])
}

/// Sets the dyn on state of a bus channel.
///
/// # Arguments
///
/// * `channel_num` - The bus channel number (1-16).
/// * `on` - The new dyn on state for the channel (0 or 1).
///
/// ```
/// use x32_lib::command::bus;
///
/// let (address, args) = bus::set_dyn_on(1, 1);
/// assert_eq!(address, "/bus/01/dyn/on");
/// assert_eq!(args, vec![osc_lib::OscArg::Int(1)]);
/// ```
pub fn set_dyn_on(channel_num: u8, on: i32) -> (String, Vec<OscArg>) {
    let address = format!("/bus/{:02}/dyn/on", channel_num);
    (address, vec![OscArg::Int(on)])
}

/// Sets the eq band type of a bus channel.
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
    let address = format!("/bus/{:02}/eq/{}/type", channel_num, band);
    (address, vec![OscArg::Int(eq_type)])
}

/// Sets the fader level of a bus channel.
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
    let address = format!("/bus/{:02}/mix/fader", channel_num);
    (address, vec![OscArg::Float(level)])
}

/// Sets the on state of a bus channel.
///
/// # Arguments
///
/// * `channel_num` - The bus channel number (1-16).
/// * `on` - The new on state for the channel (0 or 1).
///
/// ```
/// use x32_lib::command::bus;
///
/// let (address, args) = bus::set_on(1, 1);
/// assert_eq!(address, "/bus/01/mix/on");
/// assert_eq!(args, vec![osc_lib::OscArg::Int(1)]);
/// ```
pub fn set_on(channel_num: u8, on: i32) -> (String, Vec<OscArg>) {
    let address = format!("/bus/{:02}/mix/on", channel_num);
    (address, vec![OscArg::Int(on)])
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
