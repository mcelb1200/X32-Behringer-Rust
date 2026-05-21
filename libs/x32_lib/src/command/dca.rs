//! Provides functions for generating OSC commands to control X32/M32 DCAs (Digitally Controlled Amplifiers).
//!
//! DCAs are used to control the level of multiple channels simultaneously with a single fader.
//! This module provides functions for controlling the fader level, on/off state, and configuration
//! of the 8 DCAs.
use super::{Command, CommandFlags, CommandType, OFF_ON, XCOLORS};
use lazy_static::lazy_static;
use osc_lib::OscArg;

/// Programmatically generates a vector of all available OSC commands for a single DCA.
pub fn get_dca_commands(dca_num: u8) -> Vec<Command<'static>> {
    assert!((1..=8).contains(&dca_num));
    let dca = format!("{:01}", dca_num);
    let mut commands = vec![];
    let flags = CommandFlags::GET.union(CommandFlags::SET);

    let mut add =
        |path: String, command_type: CommandType, nodes: Option<&'static [&'static str]>| {
            commands.push(Command {
                path: Box::leak(path.into_boxed_str()),
                command_type,
                flags,
                nodes,
            });
        };

    add(format!("/dca/{}/fader", dca), CommandType::Float, None);
    add(format!("/dca/{}/on", dca), CommandType::Enum, Some(OFF_ON));
    add(
        format!("/dca/{}/config/name", dca),
        CommandType::String,
        None,
    );
    add(format!("/dca/{}/config/icon", dca), CommandType::Int, None);
    add(
        format!("/dca/{}/config/color", dca),
        CommandType::Enum,
        Some(XCOLORS),
    );
    commands
}

lazy_static! {
    /// A pre-computed, static vector containing the command lists for all 8 DCAs.
    pub static ref XDCA_COMMANDS: Vec<Vec<Command<'static>>> =
        (1..=8).map(get_dca_commands).collect();
}

// --- OSC Message Setters ---

/// Creates an OSC message to set the fader level of a DCA.
pub fn set_fader(dca_num: u8, level: f32) -> (String, Vec<OscArg>) {
    (
        format!("/dca/{}/fader", dca_num),
        vec![OscArg::Float(level)],
    )
}

/// Creates an OSC message to set the on state of a DCA.
pub fn set_on(dca_num: u8, on: i32) -> (String, Vec<OscArg>) {
    (format!("/dca/{}/on", dca_num), vec![OscArg::Int(on)])
}

/// Creates an OSC message to set the name of a DCA.
pub fn set_name(dca_num: u8, name: &str) -> (String, Vec<OscArg>) {
    (
        format!("/dca/{}/config/name", dca_num),
        vec![OscArg::String(name.to_string())],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dca_commands_array() {
        assert_eq!(XDCA_COMMANDS.len(), 8);
        for i in 0..8 {
            assert_eq!(XDCA_COMMANDS[i].len(), 5);
        }
    }
}
