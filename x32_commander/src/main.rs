use clap::Parser;
use std::fs::File;
use std::io::{self, BufRead};
use std::net::UdpSocket;
use std::str::FromStr;
use std::time::{Duration, Instant};

use osc_lib::OscMessage;
use x32_lib::{create_socket, error::X32Error};

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
}

#[derive(Debug, Clone, PartialEq)]
enum CommandType {
    Osc,
    Midi,
}

#[derive(Debug, Clone, PartialEq)]
struct Command {
    command_type: CommandType,
    incoming_address: String,
    outgoing_command: String,
}

fn parse_command_file(path: &str) -> io::Result<Vec<Command>> {
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);
    let mut commands = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((trigger, action)) = line.split_once('|') {
            let trigger = trigger.trim();
            let action = action.trim();

            if trigger.len() < 5 || &trigger[1..4] != "~~~" {
                eprintln!("Skipping invalid trigger format: {}", trigger);
                continue;
            }

            let command_type = match trigger.chars().next() {
                Some('O') => CommandType::Osc,
                Some('M') => CommandType::Midi,
                _ => {
                    eprintln!("Skipping invalid command type: {}", trigger);
                    continue;
                }
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

fn run(args: Args) -> Result<(), X32Error> {
    // This application is a partial rewrite of the original X32Commander.c utility.
    // Currently, only OSC commands are supported. MIDI functionality is not yet implemented.
    let commands = parse_command_file(&args.file)
        .map_err(|e| X32Error::Custom(format!("Failed to parse command file: {}", e)))?;
    println!("Successfully parsed {} commands.", commands.len());

    println!("Connecting to X32 at {}...", args.ip);
    let x32_socket = create_socket(&args.ip, 100)?;

    let out_socket = if let Some(ref output_addr) = args.output {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.connect(output_addr)?;
        println!("OSC output enabled to: {}", output_addr);
        Some(socket)
    } else {
        None
    };

    let mut last_xremote = Instant::now();
    let mut buf = [0u8; 1024];

    println!("Listening for OSC messages...");

    loop {
        if last_xremote.elapsed() > Duration::from_secs(9) {
            let xremote_msg = OscMessage::new("/xremote".to_string(), vec![]);
            x32_socket.send(&xremote_msg.to_bytes()?)?;
            last_xremote = Instant::now();
        }

        match x32_socket.recv(&mut buf) {
            Ok(len) => {
                if len > 0 {
                    if let Ok(incoming_msg) = OscMessage::from_bytes(&buf[..len]) {
                        for command in &commands {
                            if command.incoming_address == incoming_msg.path {
                                match command.command_type {
                                    CommandType::Osc => {
                                        println!(
                                            "Match found for: {}. Triggering: {}",
                                            incoming_msg.path, command.outgoing_command
                                        );
                                        match OscMessage::from_str(&command.outgoing_command) {
                                            Ok(outgoing_msg) => {
                                                let target_socket =
                                                    out_socket.as_ref().unwrap_or(&x32_socket);
                                                match outgoing_msg.to_bytes() {
                                                    Ok(bytes) => {
                                                        if let Err(e) = target_socket.send(&bytes) {
                                                            eprintln!(
                                                                "Failed to send OSC message: {}",
                                                                e
                                                            );
                                                        }
                                                    }
                                                    Err(e) => eprintln!(
                                                        "Failed to serialize outgoing OSC message: {}",
                                                        e
                                                    ),
                                                }
                                            }
                                            Err(e) => eprintln!(
                                                "Failed to parse outgoing command '{}': {}",
                                                command.outgoing_command, e
                                            ),
                                        }
                                    }
                                    CommandType::Midi => {
                                        println!(
                                            "Match found for: {}. This is a MIDI command, which is not yet supported in this version.",
                                            incoming_msg.path
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                if e.kind() != io::ErrorKind::WouldBlock && e.kind() != io::ErrorKind::TimedOut {
                    return Err(X32Error::Io(e));
                }
            }
        }
    }
}

fn main() {
    let args = Args::parse();
    if let Err(e) = run(args) {
        eprintln!("Application error: {}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn create_test_file(content: &str) -> (tempfile::NamedTempFile, String) {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        writeln!(file, "{}", content).unwrap();
        let path = file.path().to_str().unwrap().to_string();
        (file, path)
    }

    #[test]
    fn test_parse_valid_osc_command() {
        let content = "O~~~/ch/01/mix/fader|/ch/02/mix/fader ,f 0.5";
        let (_file, path) = create_test_file(content);
        let commands = parse_command_file(&path).unwrap();
        assert_eq!(commands.len(), 1);
        assert_eq!(
            commands[0],
            Command {
                command_type: CommandType::Osc,
                incoming_address: "/ch/01/mix/fader".to_string(),
                outgoing_command: "/ch/02/mix/fader ,f 0.5".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_valid_midi_command() {
        let content = "M~~~/ch/01/mix/fader|F0 00 20 32 32 2F 63 68 2F 30 31 2F 6D 69 78 2F 66 61 64 65 72 20 2C 66 20 30 2E 35 F7";
        let (_file, path) = create_test_file(content);
        let commands = parse_command_file(&path).unwrap();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0], Command {
            command_type: CommandType::Midi,
            incoming_address: "/ch/01/mix/fader".to_string(),
            outgoing_command: "F0 00 20 32 32 2F 63 68 2F 30 31 2F 6D 69 78 2F 66 61 64 65 72 20 2C 66 20 30 2E 35 F7".to_string(),
        });
    }

    #[test]
    fn test_parse_empty_lines_and_comments() {
        let content = "
# This is a comment
O~~~/ch/01/mix/fader|/ch/02/mix/fader ,f 0.5

M~~~/ch/02/mix/fader|/ch/03/mix/fader ,f 0.75
";
        let (_file, path) = create_test_file(content);
        let commands = parse_command_file(&path).unwrap();
        assert_eq!(commands.len(), 2);
    }

    #[test]
    fn test_parse_invalid_format() {
        let content = "invalid line";
        let (_file, path) = create_test_file(content);
        let commands = parse_command_file(&path).unwrap();
        assert_eq!(commands.len(), 0);
    }
}
