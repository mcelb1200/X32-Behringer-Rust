//! `x32_desk_save` is a command-line utility for saving preferences, scenes, and routing data
//! from a Behringer X32 digital mixer to a file. It is a Rust implementation of the original
//! `X32DeskSave.c` tool by Patrick-Gilles Maillot.

use clap::Parser;
use osc_lib::{OscArg, OscMessage};
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::net::UdpSocket;
use std::path::PathBuf;
use x32_lib::{create_socket, error::X32Error};

mod nodes;

/// A Rust implementation of the X32DeskSave tool.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// X32 console IP address
    #[arg(short, long, default_value = "192.168.1.64")]
    ip: String,

    /// File path to a pattern file containing OSC commands to be retrieved from the X32.
    #[arg(short, long)]
    pattern_file: Option<PathBuf>,

    /// Save a DeskSave file, containing the mixer's preferences and status.
    #[arg(short, long, group = "file_type")]
    desk_save: bool,

    /// Save a Scene file, containing the mixer's channel and configuration settings.
    #[arg(short, long, group = "file_type")]
    scene: bool,

    /// Save a Routing file, containing the mixer's input/output routing configuration.
    #[arg(short, long, group = "file_type")]
    routing: bool,

    /// The destination file path to save the retrieved data.
    #[arg(required = true)]
    destination_file: PathBuf,
}

/// Sends a list of OSC commands to the X32 and returns the responses.
///
/// # Arguments
///
/// * `socket` - A `UdpSocket` connected to the X32 console.
/// * `commands` - A slice of strings, where each string is an OSC command to be sent.
///
/// # Returns
///
/// A `Result` containing a `Vec<String>` of the X32's responses, or an `X32Error` if an
/// error occurs.
fn get_desk_data(socket: &UdpSocket, commands: &[String]) -> Result<Vec<String>, X32Error> {
    let mut results = Vec::new();
    let mut buf = [0; 512];

    for cmd in commands {
        let msg = OscMessage::new("/node".to_string(), vec![OscArg::String(cmd.to_string())]);
        socket.send(&msg.to_bytes()?)?;
        let len = socket.recv(&mut buf)?;
        let response = OscMessage::from_bytes(&buf[..len])?;
        results.push(response.to_string());
    }

    Ok(results)
}

/// The main entry point for the `x32_desk_save` utility.
///
/// This function parses command-line arguments, determines which OSC commands to send,
/// connects to the X32, retrieves the data, and saves it to a file.
fn main() -> Result<(), X32Error> {
    let args = Args::parse();

    let commands = if args.desk_save {
        nodes::DS_NODE.iter().map(|s| s.to_string()).collect()
    } else if args.scene {
        nodes::SC_NODE.iter().map(|s| s.to_string()).collect()
    } else if args.routing {
        nodes::RO_NODE.iter().map(|s| s.to_string()).collect()
    } else if let Some(pattern_file) = args.pattern_file {
        let file = File::open(pattern_file)?;
        let mut commands = Vec::new();
        for line in io::BufReader::new(file).lines() {
            let line = line?;
            if !line.starts_with('#') {
                if let Some(command) = line.split_whitespace().next() {
                    commands.push(command.to_string());
                }
            }
        }
        commands
    } else {
        // Default to DeskSave if no file type is specified
        nodes::DS_NODE.iter().map(|s| s.to_string()).collect()
    };

    if commands.is_empty() {
        eprintln!("No commands to send. Please specify a file type or a pattern file.");
        return Ok(());
    }

    let socket = create_socket(&args.ip, 500)?;
    println!("Successfully connected to X32 at {}", &args.ip);

    let data = get_desk_data(&socket, &commands)?;

    let mut file = File::create(&args.destination_file)?;
    for line in data {
        writeln!(file, "{}", line)?;
    }

    println!(
        "Successfully saved data to {}",
        args.destination_file.display()
    );

    Ok(())
}
