use crate::common::{CommandFlags, CommandFormat, CommandValue, X32Command};
use crate::{Result, UdpSocket};
use osc_lib::{OscArg, OscMessage};


pub const DCA_COMMANDS: &[X32Command] = &[
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
        assert_eq!(DCA_COMMANDS[0].command, "/dca");
        assert_eq!(DCA_COMMANDS[1].command, "/dca/1");
        assert_eq!(DCA_COMMANDS[8].command, "/dca/2");
        assert_eq!(DCA_COMMANDS[56].command, "/dca/8/config/color");
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
