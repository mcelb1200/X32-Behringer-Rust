//! `x32_desk_restore` is a command-line utility for restoring preferences, scenes, and routing data
//! to a Behringer X32 digital mixer from a file. It is a Rust implementation of the original
//! `X32DeskRestore.c` tool by Patrick-Gilles Maillot.

use clap::Parser;
use osc_lib::OscMessage;
use std::fs::File;
use std::io::{self, BufRead};
use std::net::UdpSocket;
use std::path::PathBuf;
use std::str::FromStr;
use x32_lib::{
    create_socket,
    error::{Result, X32Error},
};

/// A Rust implementation of the X32DeskRestore tool.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// X32 console IP address
    #[arg(short, long, default_value = "192.168.1.64")]
    ip: String,

    /// The source file path to restore data from.
    #[arg(index = 1)]
    file: PathBuf,
}

/// Sends a list of OSC commands to the X32.
///
/// # Arguments
///
/// * `socket` - A `UdpSocket` connected to the X32 console.
/// * `commands` - A slice of strings, where each string is an OSC command to be sent.
///
/// # Returns
///
/// A `Result` indicating success or an `X32Error` if an error occurs.
fn send_commands(socket: &UdpSocket, commands: &[String]) -> Result<()> {
    let mut buf = [0; 512];
    for cmd_str in commands {
        // Prepend "/" to the command string if it doesn't start with it
        let command = if cmd_str.starts_with('/') {
            cmd_str.to_string()
        } else {
            format!("/{}", cmd_str)
        };

        // Attempt to parse with OscMessage::from_str first
        let msg = match OscMessage::from_str(&command) {
            Ok(msg) => msg,
            Err(_) => {
                // If parsing fails, treat the line as a simple command with no arguments
                OscMessage::new(command, vec![])
            }
        };

        let bytes = msg.to_bytes()?;
        socket.send(&bytes)?;

        match socket.recv(&mut buf) {
            Ok(_) => {
                // We don't care about the response for this tool
            }
            Err(e) => {
                if e.kind() != io::ErrorKind::WouldBlock {
                    return Err(X32Error::Io(e));
                }
            }
        }
    }
    Ok(())
}

/// The main entry point for the `x32_desk_restore` utility.
///
/// This function parses command-line arguments, reads OSC commands from a file,
/// connects to the X32, and sends the commands to restore the mixer's state.
fn main() -> Result<()> {
    let args = Args::parse();

    let file = File::open(&args.file)?;
    let commands: Vec<String> = io::BufReader::new(file)
        .lines()
        .filter_map(std::result::Result::ok)
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .collect();

    if commands.is_empty() {
        eprintln!("No commands to send from file: {}", args.file.display());
        return Ok(());
    }

    let socket = create_socket(&args.ip, 2000)?;
    println!("Successfully connected to X32 at {}", args.ip);

    send_commands(&socket, &commands)?;

    println!("Successfully restored data from {}", args.file.display());

    Ok(())
}
