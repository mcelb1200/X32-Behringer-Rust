//! Provides functions for generating OSC commands to control X32/M32 effects processors (FX 1-8).
//!
//! The X32 has 8 stereo effects processors that can be loaded with a variety of effects,
//! from reverbs and delays to EQs and compressors. This module provides the functions
//! needed to control the type, source, and parameters of these effects.
use super::{Command, CommandFlags, CommandType};
use crate::error::Result;
use lazy_static::lazy_static;
use osc_lib::{OscArg, OscMessage};
use std::net::UdpSocket;

/// Programmatically generates a vector of all available OSC commands for a single effects processor.
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

lazy_static! {
    /// A pre-computed, static vector containing the command lists for all 8 effects processors.
    pub static ref XFX_COMMANDS: Vec<Vec<Command<'static>>> =
        (1..=8).map(get_fx_commands).collect();
}

// --- OSC Message Setters ---

/// Creates an OSC message to set the type of an effects processor.
pub fn set_type(fx_num: u8, fx_type: i32) -> (String, Vec<OscArg>) {
    (format!("/fx/{}/type", fx_num), vec![OscArg::Int(fx_type)])
}

/// Creates an OSC message to set a parameter of an effects processor.
pub fn set_param(fx_num: u8, param_num: u8, value: f32) -> (String, Vec<OscArg>) {
    (
        format!("/fx/{}/par/{:02}", fx_num, param_num),
        vec![OscArg::Float(value)],
    )
}

/// Sends an OSC message to set a parameter of an effects processor.
pub fn set_fx_param(
    socket: &UdpSocket,
    fx_num: u8,
    param_num: u8,
    value: f32,
) -> Result<(String, Vec<OscArg>)> {
    let (address, args) = set_param(fx_num, param_num, value);
    let msg = OscMessage::new(address.clone(), args);
    socket.send(&msg.to_bytes()?)?;
    Ok((address, msg.args))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_fx_commands() {
        let commands = get_fx_commands(1);
        assert_eq!(commands.len(), 67);
    }

    #[test]
    fn test_set_fx_param() {
        // Create a dummy server socket
        let server = UdpSocket::bind("127.0.0.1:0").unwrap();
        let server_addr = server.local_addr().unwrap();

        // Create a client socket and connect it to the server
        let client = UdpSocket::bind("127.0.0.1:0").unwrap();
        client.connect(server_addr).unwrap();

        // Call the function
        let result = set_fx_param(&client, 1, 5, 0.5);

        // Assert the result is Ok
        assert!(result.is_ok());

        // Assert on the returned values
        let (path, args) = result.unwrap();
        assert_eq!(path, "/fx/1/par/05");
        assert_eq!(args.len(), 1);
        assert_eq!(args[0], OscArg::Float(0.5));
    }
}
