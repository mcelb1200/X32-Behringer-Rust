
//! # Auxin Module
//!
//! Controls the 8 auxiliary inputs on the X32/M32.
//!
//! This module provides functions for controlling various aspects of the auxiliary inputs,
//! including configuration, EQ, and mix settings.

use crate::common::{CommandFlags, CommandFormat, CommandValue, X32Command};
use osc_lib::OscArg;
use lazy_static::lazy_static;

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

lazy_static! {
    pub static ref AUXIN_COMMANDS: Vec<X32Command> = {
        let mut commands = Vec::new();
        commands.push(X32Command { command: "/auxin".to_string(), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None });
        for i in 1..=8 {
            let auxin_prefix = format!("/auxin/{:02}", i);
            commands.extend(vec![
                X32Command { command: auxin_prefix.clone(), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
                X32Command { command: format!("{}/config", auxin_prefix), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
                X32Command { command: format!("{}/config/name", auxin_prefix), format: CommandFormat::String, flags: CommandFlags::F_XET, value: CommandValue::None },
                X32Command { command: format!("{}/config/icon", auxin_prefix), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
                X32Command { command: format!("{}/config/color", auxin_prefix), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
                X32Command { command: format!("{}/config/source", auxin_prefix), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
                X32Command { command: format!("{}/preamp", auxin_prefix), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
                X32Command { command: format!("{}/preamp/trim", auxin_prefix), format: CommandFormat::Float, flags: CommandFlags::F_XET, value: CommandValue::None },
                X32Command { command: format!("{}/preamp/invert", auxin_prefix), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
                X32Command { command: format!("{}/eq", auxin_prefix), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
                X32Command { command: format!("{}/eq/on", auxin_prefix), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
            ]);
            for j in 1..=4 {
                commands.extend(vec![
                    X32Command { command: format!("{}/eq/{}", auxin_prefix, j), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
                    X32Command { command: format!("{}/eq/{}/type", auxin_prefix, j), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
                    X32Command { command: format!("{}/eq/{}/f", auxin_prefix, j), format: CommandFormat::Float, flags: CommandFlags::F_XET, value: CommandValue::None },
                    X32Command { command: format!("{}/eq/{}/g", auxin_prefix, j), format: CommandFormat::Float, flags: CommandFlags::F_XET, value: CommandValue::None },
                    X32Command { command: format!("{}/eq/{}/q", auxin_prefix, j), format: CommandFormat::Float, flags: CommandFlags::F_XET, value: CommandValue::None },
                ]);
            }
            commands.extend(vec![
                X32Command { command: format!("{}/mix", auxin_prefix), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
                X32Command { command: format!("{}/mix/on", auxin_prefix), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
                X32Command { command: format!("{}/mix/fader", auxin_prefix), format: CommandFormat::Float, flags: CommandFlags::F_XET, value: CommandValue::None },
                X32Command { command: format!("{}/mix/st", auxin_prefix), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
                X32Command { command: format!("{}/mix/pan", auxin_prefix), format: CommandFormat::Float, flags: CommandFlags::F_XET, value: CommandValue::None },
                X32Command { command: format!("{}/mix/mono", auxin_prefix), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
                X32Command { command: format!("{}/mix/mlevel", auxin_prefix), format: CommandFormat::Float, flags: CommandFlags::F_XET, value: CommandValue::None },
            ]);
            for j in 1..=16 {
                 commands.extend(vec![
                    X32Command { command: format!("{}/mix/{:02}", auxin_prefix, j), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
                    X32Command { command: format!("{}/mix/{:02}/on", auxin_prefix, j), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
                    X32Command { command: format!("{}/mix/{:02}/level", auxin_prefix, j), format: CommandFormat::Float, flags: CommandFlags::F_XET, value: CommandValue::None },
                ]);
                if j % 2 != 0 {
                    commands.extend(vec![
                        X32Command { command: format!("{}/mix/{:02}/pan", auxin_prefix, j), format: CommandFormat::Float, flags: CommandFlags::F_XET, value: CommandValue::None },
                        X32Command { command: format!("{}/mix/{:02}/type", auxin_prefix, j), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
                        X32Command { command: format!("{}/mix/{:02}/panFollow", auxin_prefix, j), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
                    ]);
                }
            }
             commands.extend(vec![
                X32Command { command: format!("{}/grp", auxin_prefix), format: CommandFormat::StringList(&[]), flags: CommandFlags::F_FND, value: CommandValue::None },
                X32Command { command: format!("{}/grp/dca", auxin_prefix), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
                X32Command { command: format!("{}/grp/mute", auxin_prefix), format: CommandFormat::Int, flags: CommandFlags::F_XET, value: CommandValue::None },
            ]);
        }
        commands
    };
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

    #[test]
    fn test_get_auxin_commands_valid_channel() {
        assert!(AUXIN_COMMANDS.iter().any(|c| c.command == "/auxin/01/config/name"));
        assert!(AUXIN_COMMANDS.iter().any(|c| c.command == "/auxin/01/mix/01/pan"));
        assert!(!AUXIN_COMMANDS.iter().any(|c| c.command == "/auxin/01/mix/02/pan"));
    }

    #[test]
    fn test_get_auxin_commands_command_count() {
        assert_eq!(AUXIN_COMMANDS.len(), 1 + 8 * (1 + 5 + 2 + 5 + 7 + 16 * 3 + 8 * 3 + 3));
    }
}
