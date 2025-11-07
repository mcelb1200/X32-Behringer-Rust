//! # DCA Module
//!
//! Controls the 8 DCAs (Digitally Controlled Amplifiers) on the X32/M32.
//!
//! DCAs are used to control the level of multiple channels simultaneously.

use crate::common::{Color, On, CommandFlags, CommandFormat, CommandValue, X32Command};
use osc_lib::OscArg;
use lazy_static::lazy_static;

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

lazy_static! {
    pub static ref DCA_COMMANDS: Vec<X32Command> = vec![
        X32Command { command: "/dca".to_string(), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
        X32Command { command: "/dca/1".to_string(), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
        X32Command { command: "/dca/1/on".to_string(), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
        X32Command { command: "/dca/1/fader".to_string(), format: CommandFormat::Float, flags: CommandFlags::F_XET, value: CommandValue::None },
        X32Command { command: "/dca/1/config".to_string(), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
        X32Command { command: "/dca/1/config/name".to_string(), format: CommandFormat::String, flags: CommandFlags::F_XET, value: CommandValue::None },
        X32Command { command: "/dca/1/config/icon".to_string(), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
        X32Command { command: "/dca/1/config/color".to_string(), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
        X32Command { command: "/dca/2".to_string(), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
        X32Command { command: "/dca/2/on".to_string(), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
        X32Command { command: "/dca/2/fader".to_string(), format: CommandFormat::Float, flags: CommandFlags::F_XET, value: CommandValue::None },
        X32Command { command: "/dca/2/config".to_string(), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
        X32Command { command: "/dca/2/config/name".to_string(), format: CommandFormat::String, flags: CommandFlags::F_XET, value: CommandValue::None },
        X32Command { command: "/dca/2/config/icon".to_string(), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
        X32Command { command: "/dca/2/config/color".to_string(), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
        X32Command { command: "/dca/3".to_string(), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
        X32Command { command: "/dca/3/on".to_string(), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
        X32Command { command: "/dca/3/fader".to_string(), format: CommandFormat::Float, flags: CommandFlags::F_XET, value: CommandValue::None },
        X32Command { command: "/dca/3/config".to_string(), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
        X32Command { command: "/dca/3/config/name".to_string(), format: CommandFormat::String, flags: CommandFlags::F_XET, value: CommandValue::None },
        X32Command { command: "/dca/3/config/icon".to_string(), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
        X32Command { command: "/dca/3/config/color".to_string(), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
        X32Command { command: "/dca/4".to_string(), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
        X32Command { command: "/dca/4/on".to_string(), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
        X32Command { command: "/dca/4/fader".to_string(), format: CommandFormat::Float, flags: CommandFlags::F_XET, value: CommandValue::None },
        X32Command { command: "/dca/4/config".to_string(), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
        X32Command { command: "/dca/4/config/name".to_string(), format: CommandFormat::String, flags: CommandFlags::F_XET, value: CommandValue::None },
        X32Command { command: "/dca/4/config/icon".to_string(), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
        X32Command { command: "/dca/4/config/color".to_string(), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
        X32Command { command: "/dca/5".to_string(), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
        X32Command { command: "/dca/5/on".to_string(), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
        X32Command { command: "/dca/5/fader".to_string(), format: CommandFormat::Float, flags: CommandFlags::F_XET, value: CommandValue::None },
        X32Command { command: "/dca/5/config".to_string(), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
        X32Command { command: "/dca/5/config/name".to_string(), format: CommandFormat::String, flags: CommandFlags::F_XET, value: CommandValue::None },
        X32Command { command: "/dca/5/config/icon".to_string(), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
        X32Command { command: "/dca/5/config/color".to_string(), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
        X32Command { command: "/dca/6".to_string(), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
        X32Command { command: "/dca/6/on".to_string(), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
        X32Command { command: "/dca/6/fader".to_string(), format: CommandFormat::Float, flags: CommandFlags::F_XET, value: CommandValue::None },
        X32Command { command: "/dca/6/config".to_string(), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
        X32Command { command: "/dca/6/config/name".to_string(), format: CommandFormat::String, flags: CommandFlags::F_XET, value: CommandValue::None },
        X32Command { command: "/dca/6/config/icon".to_string(), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
        X32Command { command: "/dca/6/config/color".to_string(), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
        X32Command { command: "/dca/7".to_string(), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
        X32Command { command: "/dca/7/on".to_string(), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
        X32Command { command: "/dca/7/fader".to_string(), format: CommandFormat::Float, flags: CommandFlags::F_XET, value: CommandValue::None },
        X32Command { command: "/dca/7/config".to_string(), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
        X32Command { command: "/dca/7/config/name".to_string(), format: CommandFormat::String, flags: CommandFlags::F_XET, value: CommandValue::None },
        X32Command { command: "/dca/7/config/icon".to_string(), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
        X32Command { command: "/dca/7/config/color".to_string(), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
        X32Command { command: "/dca/8".to_string(), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
        X32Command { command: "/dca/8/on".to_string(), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
        X32Command { command: "/dca/8/fader".to_string(), format: CommandFormat::Float, flags: CommandFlags::F_XET, value: CommandValue::None },
        X32Command { command: "/dca/8/config".to_string(), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
        X32Command { command: "/dca/8/config/name".to_string(), format: CommandFormat::String, flags: CommandFlags::F_XET, value: CommandValue::None },
        X32Command { command: "/dca/8/config/icon".to_string(), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
        X32Command { command: "/dca/8/config/color".to_string(), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
    ];
}

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
