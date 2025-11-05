
//! This module defines the OSC commands for controlling DCA groups on the X32/X-Air mixers.
//! It provides a static array of `DcaCommand` structs that represent the available DCA commands.

use crate::{Result, UdpSocket};
use osc_lib::{OscArg, OscMessage};

/// Represents the type of argument a DCA command expects.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DcaArgType {
    /// A command that acts as a header or grouping for other commands.
    Header,
    /// An on/off toggle. Corresponds to an integer argument (0 or 1).
    OnOff,
    /// A fader level. Corresponds to a float argument (0.0 to 1.0).
    Fader,
    /// A name for the DCA group. Corresponds to a string argument.
    Name,
    /// An icon for the DCA group. Corresponds to an integer argument.
    Icon,
    /// A color for the DCA group. Corresponds to an integer argument representing an enum.
    Color,
}

/// Represents a static definition of a DCA-related OSC command.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct DcaCommand {
    /// The OSC path for the command.
    pub path: &'static str,
    /// The type of argument the command expects.
    pub arg_type: DcaArgType,
}

/// A static array of all available DCA commands.
pub const DCA_COMMANDS: &[DcaCommand] = &[
    DcaCommand { path: "/dca", arg_type: DcaArgType::Header },
    DcaCommand { path: "/dca/1", arg_type: DcaArgType::Header },
    DcaCommand { path: "/dca/1/on", arg_type: DcaArgType::OnOff },
    DcaCommand { path: "/dca/1/fader", arg_type: DcaArgType::Fader },
    DcaCommand { path: "/dca/1/config", arg_type: DcaArgType::Header },
    DcaCommand { path: "/dca/1/config/name", arg_type: DcaArgType::Name },
    DcaCommand { path: "/dca/1/config/icon", arg_type: DcaArgType::Icon },
    DcaCommand { path: "/dca/1/config/color", arg_type: DcaArgType::Color },
    DcaCommand { path: "/dca/2", arg_type: DcaArgType::Header },
    DcaCommand { path: "/dca/2/on", arg_type: DcaArgType::OnOff },
    DcaCommand { path: "/dca/2/fader", arg_type: DcaArgType::Fader },
    DcaCommand { path: "/dca/2/config", arg_type: DcaArgType::Header },
    DcaCommand { path: "/dca/2/config/name", arg_type: DcaArgType::Name },
    DcaCommand { path: "/dca/2/config/icon", arg_type: DcaArgType::Icon },
    DcaCommand { path: "/dca/2/config/color", arg_type: DcaArgType::Color },
    DcaCommand { path: "/dca/3", arg_type: DcaArgType::Header },
    DcaCommand { path: "/dca/3/on", arg_type: DcaArgType::OnOff },
    DcaCommand { path: "/dca/3/fader", arg_type: DcaArgType::Fader },
    DcaCommand { path: "/dca/3/config", arg_type: DcaArgType::Header },
    DcaCommand { path: "/dca/3/config/name", arg_type: DcaArgType::Name },
    DcaCommand { path: "/dca/3/config/icon", arg_type: DcaArgType::Icon },
    DcaCommand { path: "/dca/3/config/color", arg_type: DcaArgType::Color },
    DcaCommand { path: "/dca/4", arg_type: DcaArgType::Header },
    DcaCommand { path: "/dca/4/on", arg_type: DcaArgType::OnOff },
    DcaCommand { path: "/dca/4/fader", arg_type: DcaArgType::Fader },
    DcaCommand { path: "/dca/4/config", arg_type: DcaArgType::Header },
    DcaCommand { path: "/dca/4/config/name", arg_type: DcaArgType::Name },
    DcaCommand { path: "/dca/4/config/icon", arg_type: DcaArgType::Icon },
    DcaCommand { path: "/dca/4/config/color", arg_type: DcaArgType::Color },
    DcaCommand { path: "/dca/5", arg_type: DcaArgType::Header },
    DcaCommand { path: "/dca/5/on", arg_type: DcaArgType::OnOff },
    DcaCommand { path: "/dca/5/fader", arg_type: DcaArgType::Fader },
    DcaCommand { path: "/dca/5/config", arg_type: DcaArgType::Header },
    DcaCommand { path: "/dca/5/config/name", arg_type: DcaArgType::Name },
    DcaCommand { path: "/dca/5/config/icon", arg_type: DcaArgType::Icon },
    DcaCommand { path: "/dca/5/config/color", arg_type: DcaArgType::Color },
    DcaCommand { path: "/dca/6", arg_type: DcaArgType::Header },
    DcaCommand { path: "/dca/6/on", arg_type: DcaArgType::OnOff },
    DcaCommand { path: "/dca/6/fader", arg_type: DcaArgType::Fader },
    DcaCommand { path: "/dca/6/config", arg_type: DcaArgType::Header },
    DcaCommand { path: "/dca/6/config/name", arg_type: DcaArgType::Name },
    DcaCommand { path: "/dca/6/config/icon", arg_type: DcaArgType::Icon },
    DcaCommand { path: "/dca/6/config/color", arg_type: DcaArgType::Color },
    DcaCommand { path: "/dca/7", arg_type: DcaArgType::Header },
    DcaCommand { path: "/dca/7/on", arg_type: DcaArgType::OnOff },
    DcaCommand { path: "/dca/7/fader", arg_type: DcaArgType::Fader },
    DcaCommand { path: "/dca/7/config", arg_type: DcaArgType::Header },
    DcaCommand { path: "/dca/7/config/name", arg_type: DcaArgType::Name },
    DcaCommand { path: "/dca/7/config/icon", arg_type: DcaArgType::Icon },
    DcaCommand { path: "/dca/7/config/color", arg_type: DcaArgType::Color },
    DcaCommand { path: "/dca/8", arg_type: DcaArgType::Header },
    DcaCommand { path: "/dca/8/on", arg_type: DcaArgType::OnOff },
    DcaCommand { path: "/dca/8/fader", arg_type: DcaArgType::Fader },
    DcaCommand { path: "/dca/8/config", arg_type: DcaArgType::Header },
    DcaCommand { path: "/dca/8/config/name", arg_type: DcaArgType::Name },
    DcaCommand { path: "/dca/8/config/icon", arg_type: DcaArgType::Icon },
    DcaCommand { path: "/dca/8/config/color", arg_type: DcaArgType::Color },
];

pub fn set_dca_fader(socket: &UdpSocket, dca_index: u8, level: f32) -> Result<()> {
    let path = format!("/dca/{}/fader", dca_index);
    let msg = OscMessage::new(path, vec![OscArg::Float(level)]);
    socket.send(&msg.to_bytes()?)?;
    Ok(())
}

pub fn set_dca_on(socket: &UdpSocket, dca_index: u8, on: bool) -> Result<()> {
    let path = format!("/dca/{}/on", dca_index);
    let msg = OscMessage::new(path, vec![OscArg::Int(if on { 1 } else { 0 })]);
    socket.send(&msg.to_bytes()?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_socket;
    use std::net::UdpSocket as StdUdpSocket;
    use std::thread;

    #[test]
    fn test_dca_commands_array() {
        assert_eq!(DCA_COMMANDS.len(), 57);
        assert_eq!(DCA_COMMANDS[0].path, "/dca");
        assert_eq!(DCA_COMMANDS[0].arg_type, DcaArgType::Header);
        assert_eq!(DCA_COMMANDS[1].path, "/dca/1");
        assert_eq!(DCA_COMMANDS[8].path, "/dca/2");
        assert_eq!(DCA_COMMANDS[56].path, "/dca/8/config/color");
        assert_eq!(DCA_COMMANDS[56].arg_type, DcaArgType::Color);
    }

    #[test]
    fn test_set_dca_fader() {
        let server = StdUdpSocket::bind("127.0.0.1:0").unwrap();
        let server_addr = server.local_addr().unwrap();

        let client_socket = create_socket(&server_addr.ip().to_string(), 200).unwrap();
        client_socket.connect(server_addr).unwrap();

        let handle = thread::spawn(move || {
            let mut buf = [0; 512];
            let (len, _) = server.recv_from(&mut buf).unwrap();
            let msg = OscMessage::from_bytes(&buf[..len]).unwrap();
            assert_eq!(msg.path, "/dca/1/fader");
            assert_eq!(msg.args.len(), 1);
            assert_eq!(msg.args[0], OscArg::Float(0.75));
        });

        set_dca_fader(&client_socket, 1, 0.75).unwrap();
        handle.join().unwrap();
    }

    #[test]
    fn test_set_dca_on() {
        let server = StdUdpSocket::bind("127.0.0.1:0").unwrap();
        let server_addr = server.local_addr().unwrap();

        let client_socket = create_socket(&server_addr.ip().to_string(), 200).unwrap();
        client_socket.connect(server_addr).unwrap();

        let handle = thread::spawn(move || {
            let mut buf = [0; 512];
            let (len, _) = server.recv_from(&mut buf).unwrap();
            let msg = OscMessage::from_bytes(&buf[..len]).unwrap();
            assert_eq!(msg.path, "/dca/2/on");
            assert_eq!(msg.args.len(), 1);
            assert_eq!(msg.args[0], OscArg::Int(1));
        });

        set_dca_on(&client_socket, 2, true).unwrap();
        handle.join().unwrap();
    }
}
