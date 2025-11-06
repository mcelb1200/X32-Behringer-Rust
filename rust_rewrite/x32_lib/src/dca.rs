//! # DCA Module
//!
//! Controls the 8 DCAs (Digitally Controlled Amplifiers) on the X32/M32.
//!
//! DCAs are used to control the level of multiple channels simultaneously.

use crate::common::{Color, On, CommandFlags, CommandFormat, CommandValue, X32Command};
use osc_lib::OscArg;

/// Sets the fader level for a specific DCA.
///
/// # Arguments
///
/// * `dca_id` - The ID of the DCA (1-8).
/// * `level` - The fader level (0.0 to 1.0).
///
/// # Example
///
/// ```
/// use x32_lib::dca;
/// use osc_lib::OscArg;
///
/// let (address, args) = dca::set_fader(1, 0.75);
/// assert_eq!(address, "/dca/1/fader");
/// assert_eq!(args, vec![OscArg::Float(0.75)]);
/// ```
pub fn set_fader(dca_id: u8, level: f32) -> (String, Vec<OscArg>) {
    let address = format!("/dca/{}/fader", dca_id);
    let args = vec![OscArg::Float(level)];
    (address, args)
}

/// Sets the mute state for a specific DCA.
///
/// # Arguments
///
/// * `dca_id` - The ID of the DCA (1-8).
/// * `on` - The mute state (`On::On` for muted, `On::Off` for unmuted).
///
/// # Example
///
/// ```
/// use x32_lib::dca;
/// use x32_lib::common::On;
/// use osc_lib::OscArg;
///
/// let (address, args) = dca::set_on(1, On::On);
/// assert_eq!(address, "/dca/1/on");
/// assert_eq!(args, vec![OscArg::Int(1)]);
/// ```
pub fn set_on(dca_id: u8, on: On) -> (String, Vec<OscArg>) {
    let address = format!("/dca/{}/on", dca_id);
    let args = vec![OscArg::Int(on as i32)];
    (address, args)
}

/// Sets the name for a specific DCA.
///
/// # Arguments
///
/// * `dca_id` - The ID of the DCA (1-8).
/// * `name` - The name to set.
///
/// # Example
///
/// ```
/// use x32_lib::dca;
/// use osc_lib::OscArg;
///
/// let (address, args) = dca::set_name(1, "Drums");
/// assert_eq!(address, "/dca/1/config/name");
/// assert_eq!(args, vec![OscArg::String("Drums".to_string())]);
/// ```
pub fn set_name(dca_id: u8, name: &str) -> (String, Vec<OscArg>) {
    let address = format!("/dca/{}/config/name", dca_id);
    let args = vec![OscArg::String(name.to_string())];
    (address, args)
}

/// Sets the color for a specific DCA.
///
/// # Arguments
///
/// * `dca_id` - The ID of the DCA (1-8).
/// * `color` - The color to set.
///
/// # Example
///
/// ```
/// use x32_lib::dca;
/// use x32_lib::common::Color;
/// use osc_lib::OscArg;
///
/// let (address, args) = dca::set_color(1, Color::Red);
/// assert_eq!(address, "/dca/1/config/color");
/// assert_eq!(args, vec![OscArg::Int(1)]);
/// ```
pub fn set_color(dca_id: u8, color: Color) -> (String, Vec<OscArg>) {
    let address = format!("/dca/{}/config/color", dca_id);
    let args = vec![OscArg::Int(color as i32)];
    (address, args)
}

use std::borrow::Cow;
pub const DCA_COMMANDS: &[X32Command] = &[
    X32Command { command: Cow::Borrowed("/dca"), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/1"), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/1/on"), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/1/fader"), format: CommandFormat::Float, flags: CommandFlags::F_XET, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/1/config"), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/1/config/name"), format: CommandFormat::String, flags: CommandFlags::F_XET, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/1/config/icon"), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/1/config/color"), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/2"), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/2/on"), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/2/fader"), format: CommandFormat::Float, flags: CommandFlags::F_XET, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/2/config"), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/2/config/name"), format: CommandFormat::String, flags: CommandFlags::F_XET, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/2/config/icon"), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/2/config/color"), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/3"), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/3/on"), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/3/fader"), format: CommandFormat::Float, flags: CommandFlags::F_XET, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/3/config"), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/3/config/name"), format: CommandFormat::String, flags: CommandFlags::F_XET, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/3/config/icon"), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/3/config/color"), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/4"), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/4/on"), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/4/fader"), format: CommandFormat::Float, flags: CommandFlags::F_XET, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/4/config"), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/4/config/name"), format: CommandFormat::String, flags: CommandFlags::F_XET, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/4/config/icon"), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/4/config/color"), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/5"), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/5/on"), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/5/fader"), format: CommandFormat::Float, flags: CommandFlags::F_XET, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/5/config"), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/5/config/name"), format: CommandFormat::String, flags: CommandFlags::F_XET, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/5/config/icon"), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/5/config/color"), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/6"), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/6/on"), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/6/fader"), format: CommandFormat::Float, flags: CommandFlags::F_XET, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/6/config"), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/6/config/name"), format: CommandFormat::String, flags: CommandFlags::F_XET, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/6/config/icon"), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/6/config/color"), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/7"), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/7/on"), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/7/fader"), format: CommandFormat::Float, flags: CommandFlags::F_XET, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/7/config"), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/7/config/name"), format: CommandFormat::String, flags: CommandFlags::F_XET, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/7/config/icon"), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/7/config/color"), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/8"), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/8/on"), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/8/fader"), format: CommandFormat::Float, flags: CommandFlags::F_XET, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/8/config"), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/8/config/name"), format: CommandFormat::String, flags: CommandFlags::F_XET, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/8/config/icon"), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
    X32Command { command: Cow::Borrowed("/dca/8/config/color"), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_fader() {
        let (address, args) = set_fader(1, 0.75);
        assert_eq!(address, "/dca/1/fader");
        assert_eq!(args, vec![OscArg::Float(0.75)]);
    }

    #[test]
    fn test_dca_commands_array() {
        assert_eq!(DCA_COMMANDS.len(), 57);
        assert_eq!(DCA_COMMANDS[0].command, "/dca");
        assert_eq!(DCA_COMMANDS[1].command, "/dca/1");
        assert_eq!(DCA_COMMANDS[8].command, "/dca/2");
        assert_eq!(DCA_COMMANDS[56].command, "/dca/8/config/color");
    }

    #[test]
    fn test_set_on() {
        let (address, args) = set_on(1, On::On);
        assert_eq!(address, "/dca/1/on");
        assert_eq!(args, vec![OscArg::Int(1)]);
    }
}