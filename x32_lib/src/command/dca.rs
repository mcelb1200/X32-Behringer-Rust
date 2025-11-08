//! This module provides the command definitions for the X32 dca faders.
use super::{Command, CommandFlags, CommandType, OFF_ON, XCOLORS};
use lazy_static::lazy_static;

pub fn get_dca_commands(dca_num: u8) -> Vec<Command<'static>> {
    assert!((1..=8).contains(&dca_num));
    let dca = format!("{:01}", dca_num);
    let mut commands = vec![];
    let flags = CommandFlags::GET.union(CommandFlags::SET);

    let mut add = |path: String, command_type: CommandType, nodes: Option<&'static [&'static str]>| {
        commands.push(Command {
            path: Box::leak(path.into_boxed_str()),
            command_type,
            flags,
            nodes,
        });
    };

    add(format!("/dca/{}/fader", dca), CommandType::Float, None);
    add(format!("/dca/{}/on", dca), CommandType::Enum, Some(OFF_ON));
    add(format!("/dca/{}/config/name", dca), CommandType::String, None);
    add(format!("/dca/{}/config/icon", dca), CommandType::Int, None);
    add(format!("/dca/{}/config/color", dca), CommandType::Enum, Some(XCOLORS));
    commands
}

lazy_static! {
    pub static ref XDCA_COMMANDS: Vec<Vec<Command<'static>>> =
        (1..=8).map(get_dca_commands).collect();
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
