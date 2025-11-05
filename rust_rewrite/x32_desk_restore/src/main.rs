use clap::Parser;
use std::fs::File;
use std::io::{self, BufRead};
use std::net::UdpSocket;
use x32_lib::{create_socket, Result, X32Error};
use osc_lib::OscMessage;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// IP address of the X32 mixer
    #[arg(short, long, default_value = "192.168.1.64")]
    ip: String,

    /// File to restore from
    #[arg(index = 1)]
    file: String,
}

fn main() {
    let args = Args::parse();
    if let Err(e) = run(&args.ip, &args.file) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run(ip: &str, file_path: &str) -> Result<()> {
    let socket = create_socket(ip, 2000)?;
    println!("Connected to X32 at {}", ip);

    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        if !line.starts_with('#') && !line.trim().is_empty() {
            // Prepend "/" to the command string
            let command = if line.starts_with('/') {
                line.to_string()
            } else {
                format!("/{}", line)
            };

            // Attempt to parse with OscMessage::from_str first
            let msg = match OscMessage::from_str(&command) {
                Ok(msg) => msg,
                Err(_) => {
                    // If parsing fails, treat the line as a simple command with no arguments
                    OscMessage::new(command, vec![])
                }
            };

            send_and_receive(&socket, &msg)?;
        }
    }

    Ok(())
}

fn send_and_receive(socket: &UdpSocket, msg: &OscMessage) -> Result<()> {
    let bytes = msg.to_bytes()?;
    socket.send(&bytes)?;

    let mut buf = [0; 512];
    match socket.recv(&mut buf) {
        Ok(_) => {
            // We don't care about the response for this tool
        },
        Err(e) => {
            if e.kind() != io::ErrorKind::WouldBlock {
                return Err(X32Error::Io(e));
            }
        }
    }
    Ok(())
}
