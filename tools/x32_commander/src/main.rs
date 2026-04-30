//! `x32_commander` is a bridge application that listens for specific OSC commands from
//! an X32 mixer and triggers other OSC commands in response.
//!
//! It reads a configuration file (default: `X32Commander.txt`) which maps incoming
//! OSC paths to outgoing OSC commands. This allows for complex automation and logic
//! that is not natively supported by the mixer, such as controlling one channel based
//! on the state of another, or creating macro-like functionality.
//!
//! Currently, only OSC-to-OSC mapping is fully implemented.
//!
//! # Credits
//!
//! *   **Original concept and work on the C library:** Patrick-Gilles Maillot
//! *   **Additional concepts by:** mcelb1200
//! *   **Rust implementation by:** mcelb1200

use clap::Parser;
use std::fs::File;
use std::io::{self, BufRead, Read};
use std::str::FromStr;
use std::time::Duration;
use tokio::net::UdpSocket;

use midir::MidiOutput;
use osc_lib::OscMessage;
use x32_lib::MixerClient;

/// A Rust implementation of the X32Commander utility.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the command file.
    #[arg(short, long, default_value = "X32Commander.txt")]
    file: String,

    /// IP address of the X32 mixer.
    #[arg(short, long)]
    ip: String,

    /// OSC output address and port (e.g., 127.0.0.1:9000).
    #[arg(short, long)]
    output: Option<String>,

    /// MIDI output port name.
    #[arg(short = 'm', long)]
    midi_out: Option<String>,
}

/// Represents the type of command to trigger.
#[derive(Debug, Clone, PartialEq)]
enum CommandType {
    /// An Open Sound Control (OSC) command.
    Osc,
    /// A MIDI command.
    Midi,
}

/// Represents a parsed command mapping from the configuration file.
#[derive(Debug, Clone, PartialEq)]
struct Command {
    /// The type of the outgoing command (OSC or MIDI).
    command_type: CommandType,
    /// The OSC path to listen for on the input.
    incoming_address: String,
    /// The command string to execute when the trigger is received.
    outgoing_command: String,
}

/// Parses the command configuration file.
fn parse_command_file(path: &str) -> io::Result<Vec<Command>> {
    let file = File::open(path)?;

    if file.metadata()?.len() > 1024 * 1024 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Configuration file too large (max 1MB)",
        ));
    }

    let mut file = file.take(1024 * 1024);
    let mut reader = io::BufReader::new(&mut file);
    let mut commands = Vec::new();

    loop {
        let mut line = String::new();
        let len = reader.by_ref().take(4096).read_line(&mut line)?;
        if len == 0 {
            break;
        }

        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((trigger, action)) = line.split_once('|') {
            let trigger = trigger.trim();
            let action = action.trim();

            if trigger.len() < 5 || &trigger[1..4] != "~~~" {
                continue;
            }

            let command_type = match trigger.chars().next() {
                Some('O') => CommandType::Osc,
                Some('M') => CommandType::Midi,
                _ => continue,
            };

            let incoming_address = trigger[4..].trim().to_string();

            commands.push(Command {
                command_type,
                incoming_address,
                outgoing_command: action.to_string(),
            });
        }
    }
    Ok(commands)
}

fn parse_midi_hex(hex_str: &str) -> std::result::Result<Vec<u8>, anyhow::Error> {
    hex_str
        .split_whitespace()
        .map(|s| {
            u8::from_str_radix(s, 16)
                .map_err(|e| anyhow::anyhow!("Invalid hex byte '{}': {}", s, e))
        })
        .collect()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let commands = parse_command_file(&args.file)
        .map_err(|e| anyhow::anyhow!("Failed to parse command file: {}", e))?;
    println!("Successfully parsed {} commands.", commands.len());

    println!("Connecting to X32 at {}...", args.ip);
    let client = MixerClient::connect(&args.ip, true).await?;

    let out_socket = if let Some(ref output_addr) = args.output {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.connect(output_addr).await?;
        println!("OSC output enabled to: {}", output_addr);
        Some(socket)
    } else {
        None
    };

    let mut midi_conn = None;
    if let Some(ref midi_port_name) = args.midi_out {
        let midi_out = MidiOutput::new("x32_commander")?;
        let ports = midi_out.ports();
        let mut found_port = None;
        for port in &ports {
            if let Ok(name) = midi_out.port_name(port) {
                if name.to_lowercase().contains(&midi_port_name.to_lowercase()) {
                    found_port = Some(port.clone());
                    break;
                }
            }
        }

        if let Some(port) = found_port {
            println!(
                "Connecting to MIDI output: {}",
                midi_out.port_name(&port).unwrap_or_default()
            );
            let conn = midi_out
                .connect(&port, "x32_commander_out")
                .map_err(|e| anyhow::anyhow!("Failed to connect to MIDI port: {}", e))?;
            midi_conn = Some(conn);
        }
    }

    let mut rx = client.subscribe();
    println!("Listening for OSC messages...");

    while let Ok(incoming_msg) = rx.recv().await {
        for command in &commands {
            if command.incoming_address == incoming_msg.path {
                match command.command_type {
                    CommandType::Osc => {
                        println!(
                            "Match found for: {}. Triggering: {}",
                            incoming_msg.path, command.outgoing_command
                        );
                        if let Ok(outgoing_msg) = OscMessage::from_str(&command.outgoing_command) {
                            if let Some(ref target_socket) = out_socket {
                                if let Ok(bytes) = outgoing_msg.to_bytes() {
                                    let _ = target_socket.send(&bytes).await;
                                }
                            } else {
                                let _ = client
                                    .send_message(&outgoing_msg.path, outgoing_msg.args)
                                    .await;
                            }
                        }
                    }
                    CommandType::Midi => {
                        if let Ok(bytes) = parse_midi_hex(&command.outgoing_command) {
                            if let Some(ref mut conn) = midi_conn {
                                let _ = conn.send(&bytes);
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn create_test_file(content: &str) -> anyhow::Result<(tempfile::NamedTempFile, String)> {
        let mut file = tempfile::NamedTempFile::new()?;
        writeln!(file, "{}", content)?;
        let path = file
            .path()
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Failed to convert temp file path to string"))?
            .to_string();
        Ok((file, path))
    }

    #[test]
    fn test_parse_valid_osc_command() -> anyhow::Result<()> {
        let content = "O~~~/ch/01/mix/fader|/ch/02/mix/fader ,f 0.5";
        let (_file, path) = create_test_file(content)?;
        let commands = parse_command_file(&path)?;
        assert_eq!(commands.len(), 1);
        assert_eq!(
            commands[0],
            Command {
                command_type: CommandType::Osc,
                incoming_address: "/ch/01/mix/fader".to_string(),
                outgoing_command: "/ch/02/mix/fader ,f 0.5".to_string(),
            }
        );
        Ok(())
    }

    #[test]
    fn test_parse_valid_midi_command() -> anyhow::Result<()> {
        let content = "M~~~/ch/01/mix/fader|F0 00 20 32 32 2F 63 68 2F 30 31 2F 6D 69 78 2F 66 61 64 65 72 20 2C 66 20 30 2E 35 F7";
        let (_file, path) = create_test_file(content)?;
        let commands = parse_command_file(&path)?;
        assert_eq!(commands.len(), 1);
        assert_eq!(
            commands[0],
            Command {
                command_type: CommandType::Midi,
                incoming_address: "/ch/01/mix/fader".to_string(),
                outgoing_command: "F0 00 20 32 32 2F 63 68 2F 30 31 2F 6D 69 78 2F 66 61 64 65 72 20 2C 66 20 30 2E 35 F7".to_string(),
            }
        );
        Ok(())
    }

    #[test]
    fn test_parse_empty_lines_and_comments() -> anyhow::Result<()> {
        let content = "
# This is a comment
O~~~/ch/01/mix/fader|/ch/02/mix/fader ,f 0.5

M~~~/ch/02/mix/fader|/ch/03/mix/fader ,f 0.75
";
        let (_file, path) = create_test_file(content)?;
        let commands = parse_command_file(&path)?;
        assert_eq!(commands.len(), 2);
        Ok(())
    }

    #[test]
    fn test_parse_invalid_format() -> anyhow::Result<()> {
        let content = "invalid line";
        let (_file, path) = create_test_file(content)?;
        let commands = parse_command_file(&path)?;
        assert_eq!(commands.len(), 0);
        Ok(())
    }

    #[test]
    fn test_parse_midi_hex() -> anyhow::Result<()> {
        let hex_str = "F0 00 20 32 32 2F 63 68 2F 30 31 2F 6D 69 78 2F 66 61 64 65 72 20 2C 66 20 30 2E 35 F7";
        let parsed = parse_midi_hex(hex_str)?;
        assert_eq!(
            parsed,
            vec![
                0xF0, 0x00, 0x20, 0x32, 0x32, 0x2F, 0x63, 0x68, 0x2F, 0x30, 0x31, 0x2F, 0x6D, 0x69,
                0x78, 0x2F, 0x66, 0x61, 0x64, 0x65, 0x72, 0x20, 0x2C, 0x66, 0x20, 0x30, 0x2E, 0x35,
                0xF7
            ]
        );

        let invalid = "F0 00 20 32 32 XX F7";
        assert!(parse_midi_hex(invalid).is_err());
        Ok(())
    }
}
