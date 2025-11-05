use clap::Parser;
use std::path::PathBuf;
use std::net::UdpSocket;
use std::fs::File;
use std::io::{self, BufRead, Write};
use x32_lib::{create_socket, X32Error};
use osc_lib::{OscMessage, OscArg};

mod nodes;

#[derive(Parser, Debug)]
#[command(author, version, about = "A Rust implementation of the X32DeskSave tool.", long_about = None)]
struct Args {
    /// X32 console IP address
    #[arg(short, long, default_value = "192.168.1.64")]
    ip: String,

    /// File path to pattern input file
    #[arg(short, long)]
    pattern_file: Option<PathBuf>,

    /// DeskSave file
    #[arg(short, long, group = "file_type")]
    desk_save: bool,

    /// Scene file
    #[arg(short, long, group = "file_type")]
    scene: bool,

    /// Routing file
    #[arg(short, long, group = "file_type")]
    routing: bool,

    /// Destination file name/path
    #[arg(required = true)]
    destination_file: PathBuf,
}

fn get_desk_data(socket: &UdpSocket, commands: &[String]) -> Result<Vec<String>, X32Error> {
    let mut results = Vec::new();
    let mut buf = [0; 512];

    for cmd in commands {
        let msg = OscMessage::new("/node".to_string(), vec![OscArg::String(",s".to_string()), OscArg::String(cmd.to_string())]);
        socket.send(&msg.to_bytes()?)?;
        let len = socket.recv(&mut buf)?;
        let response = OscMessage::from_bytes(&buf[..len])?;
        results.push(response.to_string());
    }

    Ok(results)
}

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

    let socket = create_socket(&args.ip, 2000)?;
    println!("Successfully connected to X32 at {}", args.ip);

    let data = get_desk_data(&socket, &commands)?;

    let mut file = File::create(&args.destination_file)?;
    for line in data {
        writeln!(file, "{}", line)?;
    }

    println!("Successfully saved data to {}", args.destination_file.display());

    Ok(())
}
