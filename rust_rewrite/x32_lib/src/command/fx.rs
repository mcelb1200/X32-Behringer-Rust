//! This module provides the command definitions for the X32 effects processors.
use super::{Command, CommandFlags, CommandType};
use lazy_static::lazy_static;
use osc_lib::{OscArg, OscMessage};
use crate::error::Result;
use std::net::UdpSocket;

pub fn get_fx_commands(fx_num: u8) -> Vec<Command<'static>> {
    assert!((1..=8).contains(&fx_num));
    let fx = format!("{:01}", fx_num);
    let mut commands = vec![];
    let flags = CommandFlags::GET.union(CommandFlags::SET);

    let mut add = |path: String, command_type: CommandType| {
        commands.push(Command {
            path: Box::leak(path.into_boxed_str()),
            command_type,
            flags,
            nodes: None,
        });
    };

    add(format!("/fx/{}/type", fx), CommandType::Int);
    add(format!("/fx/{}/source/l", fx), CommandType::Int);
    add(format!("/fx/{}/source/r", fx), CommandType::Int);
    for i in 1..=64 {
        add(format!("/fx/{}/par/{:02}", fx, i), CommandType::Float);
    }
    commands
}

pub fn set_fx_param(socket: &UdpSocket, fx_num: u8, param_num: u8, value: f32) -> Result<(String, Vec<OscArg>)> {
    let address = format!("/fx/{}/par/{:02}", fx_num, param_num);
    let msg = OscMessage::new(address.clone(), vec![OscArg::Float(value)]);
    socket.send(&msg.to_bytes()?)?;
    Ok((address, msg.args))
}

lazy_static! {
    pub static ref XFX_COMMANDS: Vec<Vec<Command<'static>>> =
        (1..=8).map(get_fx_commands).collect();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_fx_commands() {
        let commands = get_fx_commands(1);
        assert_eq!(commands.len(), 67);
    }
}
