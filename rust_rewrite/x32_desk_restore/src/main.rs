//! `x32_desk_restore` is a command-line utility for restoring preferences, scenes, and routing data
//! to a Behringer X32 digital mixer from a file. It is a Rust implementation of the original
//! `X32DeskRestore.c` tool by Patrick-Gilles Maillot.

use clap::Parser;
use std::path::PathBuf;
use x32_lib::{create_socket, X32Error};
use osc_lib::{OscMessage};
use std::fs::File;
use std::io::{self, BufRead};
use std::net::UdpSocket;

/// A Rust implementation of the X32DeskRestore tool.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// X32 console IP address
    #[arg(short, long, default_value = "192.168.1.64")]
    ip: String,

    /// The source file path to restore data from.
    #[arg(required = true)]
    source_file: PathBuf,
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
fn send_commands(socket: &UdpSocket, commands: &[String]) -> Result<(), X32Error> {
    let mut buf = [0; 512];
    for cmd_str in commands {
        let msg = OscMessage::from_str(cmd_str)?;
        socket.send(&msg.to_bytes()?)?;
        // We expect a response, but we'll ignore it for now.
        let _ = socket.recv(&mut buf);
    }
    Ok(())
}

/// The main entry point for the `x32_desk_restore` utility.
///
/// This function parses command-line arguments, reads OSC commands from a file,
/// connects to the X32, and sends the commands to restore the mixer's state.
fn main() -> Result<(), X32Error> {
    let args = Args::parse();

    let file = File::open(&args.source_file)?;
    let commands: Vec<String> = io::BufReader::new(file)
        .lines()
        .filter_map(io::Result::ok)
        .filter(|line| !line.starts_with('#'))
        .collect();

    if commands.is_empty() {
        eprintln!("No commands to send from file: {}", args.source_file.display());
        return Ok(());
    }

    let socket = create_socket(&args.ip, 2000)?;
    println!("Successfully connected to X32 at {}", args.ip);

    send_commands(&socket, &commands)?;

    println!("Successfully restored data from {}", args.source_file.display());

    Ok(())
}
