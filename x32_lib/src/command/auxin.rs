//! Provides functions for generating OSC commands to control X32/M32 auxiliary inputs (AuxIn 1-8).
//!
//! The AuxIn channels have a more limited set of controls compared to the main input channels,
//! primarily focusing on configuration (name, color), EQ, and mix/fader settings.
//!
//! As with other `command` modules, you can use `set_*` functions to generate OSC messages
//! for setting parameters, and the corresponding address getter functions to retrieve the
//! OSC address for querying values.
use super::{Command, CommandFlags, CommandType};
use osc_lib::OscArg;

/// A static array of all available commands for the AuxIn channels.
pub const XAUXIN_COMMANDS: &[Command] = &[
    Command {
        path: "/auxin/01/mix/fader",
        command_type: CommandType::Float,
        flags: CommandFlags::GET.union(CommandFlags::SET),
        nodes: None,
    },
    Command {
        path: "/auxin/01/mix/on",
        command_type: CommandType::Enum,
        flags: CommandFlags::GET.union(CommandFlags::SET),
        nodes: None,
    },
    Command {
        path: "/auxin/02/mix/fader",
        command_type: CommandType::Float,
        flags: CommandFlags::GET.union(CommandFlags::SET),
        nodes: None,
    },
    Command {
        path: "/auxin/02/mix/on",
        command_type: CommandType::Enum,
        flags: CommandFlags::GET.union(CommandFlags::SET),
        nodes: None,
    },
    Command {
        path: "/auxin/03/mix/fader",
        command_type: CommandType::Float,
        flags: CommandFlags::GET.union(CommandFlags::SET),
        nodes: None,
    },
    Command {
        path: "/auxin/03/mix/on",
        command_type: CommandType::Enum,
        flags: CommandFlags::GET.union(CommandFlags::SET),
        nodes: None,
    },
    Command {
        path: "/auxin/04/mix/fader",
        command_type: CommandType::Float,
        flags: CommandFlags::GET.union(CommandFlags::SET),
        nodes: None,
    },
    Command {
        path: "/auxin/04/mix/on",
        command_type: CommandType::Enum,
        flags: CommandFlags::GET.union(CommandFlags::SET),
        nodes: None,
    },
];

// --- Address String Getters ---

/// Returns the OSC address for an auxin channel's name.
pub fn name(channel_num: u8) -> String {
    format!("/auxin/{:02}/config/name", channel_num)
}

/// Returns the OSC address for an auxin channel's color.
pub fn color(channel_num: u8) -> String {
    format!("/auxin/{:02}/config/color", channel_num)
}

/// Returns the OSC address for an auxin channel's EQ band type.
pub fn eq_band_type(channel_num: u8, band: u8) -> String {
    format!("/auxin/{:02}/eq/{}/type", channel_num, band)
}

/// Returns the OSC address for an auxin channel's fader level.
pub fn fader_level(channel_num: u8) -> String {
    format!("/auxin/{:02}/mix/fader", channel_num)
}

/// Returns the OSC address for an auxin channel's on/off (mute) state.
pub fn on(channel_num: u8) -> String {
    format!("/auxin/{:02}/mix/on", channel_num)
}

// --- OSC Message Setters ---

/// Creates an OSC message to set the name of an auxin channel.
///
/// # Arguments
///
/// * `channel_num` - The auxin channel number (1-8).
/// * `name` - The new name for the channel.
///
/// ```
/// use x32_lib::command::auxin;
///
/// let (address, args) = auxin::set_name(1, "Test");
/// assert_eq!(address, "/auxin/01/config/name");
/// assert_eq!(args, vec![osc_lib::OscArg::String("Test".to_string())]);
/// ```
pub fn set_name(channel_num: u8, name: &str) -> (String, Vec<OscArg>) {
    (
        self::name(channel_num),
        vec![OscArg::String(name.to_string())],
    )
}

/// Creates an OSC message to set the color of an auxin channel.
///
/// # Arguments
///
/// * `channel_num` - The auxin channel number (1-8).
/// * `color` - The new color for the channel (0-15).
///
/// ```
/// use x32_lib::command::auxin;
///
/// let (address, args) = auxin::set_color(1, 2);
/// assert_eq!(address, "/auxin/01/config/color");
/// assert_eq!(args, vec![osc_lib::OscArg::Int(2)]);
/// ```
pub fn set_color(channel_num: u8, color: i32) -> (String, Vec<OscArg>) {
    (self::color(channel_num), vec![OscArg::Int(color)])
}

/// Creates an OSC message to set the eq band type of an auxin channel.
///
/// # Arguments
///
/// * `channel_num` - The auxin channel number (1-8).
/// * `band` - The eq band (1-4).
/// * `eq_type` - The new eq type for the channel (0-5).
///
/// ```
/// use x32_lib::command::auxin;
///
/// let (address, args) = auxin::set_eq_band_type(1, 1, 2);
/// assert_eq!(address, "/auxin/01/eq/1/type");
/// assert_eq!(args, vec![osc_lib::OscArg::Int(2)]);
/// ```
pub fn set_eq_band_type(channel_num: u8, band: u8, eq_type: i32) -> (String, Vec<OscArg>) {
    (
        self::eq_band_type(channel_num, band),
        vec![OscArg::Int(eq_type)],
    )
}

/// Creates an OSC message to set the fader level of an auxin channel.
///
/// # Arguments
///
/// * `channel_num` - The auxin channel number (1-8).
/// * `level` - The new fader level (0.0-1.0).
///
/// ```
/// use x32_lib::command::auxin;
///
/// let (address, args) = auxin::set_fader(1, 0.75);
/// assert_eq!(address, "/auxin/01/mix/fader");
/// assert_eq!(args, vec![osc_lib::OscArg::Float(0.75)]);
/// ```
pub fn set_fader(channel_num: u8, level: f32) -> (String, Vec<OscArg>) {
    (fader_level(channel_num), vec![OscArg::Float(level)])
}

/// Creates an OSC message to set the on state of an auxin channel.
///
/// # Arguments
///
/// * `channel_num` - The auxin channel number (1-8).
/// * `on` - The new on state for the channel (0 for Off, 1 for On).
///
/// ```
/// use x32_lib::command::auxin;
///
/// let (address, args) = auxin::set_on(1, 1);
/// assert_eq!(address, "/auxin/01/mix/on");
/// assert_eq!(args, vec![osc_lib::OscArg::Int(1)]);
/// ```
pub fn set_on(channel_num: u8, on: i32) -> (String, Vec<OscArg>) {
    (self::on(channel_num), vec![OscArg::Int(on)])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_auxin_commands_command_count() {
        assert_eq!(XAUXIN_COMMANDS.len(), 8);
    }

    #[test]
    fn test_get_auxin_commands_valid_channel() {
        let commands = &XAUXIN_COMMANDS;
        assert_eq!(commands[0].path, "/auxin/01/mix/fader");
        assert_eq!(commands.last().unwrap().path, "/auxin/04/mix/on");
    }

    #[test]
    #[should_panic]
    fn test_get_auxin_commands_invalid_channel() {
        let _ = &XAUXIN_COMMANDS[8];
    }

    #[test]
    fn test_set_fader() {
        let (address, args) = set_fader(1, 0.5);
        assert_eq!(address, "/auxin/01/mix/fader");
        assert_eq!(args, vec![OscArg::Float(0.5)]);
    }

    #[test]
    fn test_set_on() {
        let (address, args) = set_on(2, 1);
        assert_eq!(address, "/auxin/02/mix/on");
        assert_eq!(args, vec![OscArg::Int(1)]);
    }
}
