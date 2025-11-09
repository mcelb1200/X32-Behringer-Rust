//! `x32_tcp` is a command-line utility that acts as a TCP to UDP bridge for the Behringer X32 and Midas M32 digital mixers.
//! It allows you to send OSC commands to the mixer using a simple text-based TCP protocol.

use anyhow::Result;
use clap::Parser;
use osc_lib::OscMessage;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::str::FromStr;
use std::thread;
use x32_lib::create_socket;

/// A TCP to UDP bridge for the Behringer X32 digital mixer.
#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Server max connections backlog
    #[clap(short, long, default_value_t = 10)]
    backlog: i32,

    /// X32 console IP address
    #[clap(short, long, default_value = "127.0.0.1")]
    ip: String,

    /// Server port
    #[clap(short, long, default_value_t = 10041)]
    port: u16,

    /// Debug mode
    #[clap(short, long)]
    debug: bool,

    /// Verbose mode
    #[clap(short, long)]
    verbose: bool,
}

/// The main entry point for the `x32_tcp` application.
///
/// This function parses command-line arguments, starts the TCP server, and listens for incoming client connections.
/// Each client connection is handled in a separate thread.
fn main() -> Result<()> {
    let args = Args::parse();

    if args.verbose {
        println!("Starting X32TCP server...");
        println!("Listening on port: {}", args.port);
        println!("Connecting to X32 at: {}", args.ip);
        println!("Backlog set to: {}", args.backlog);
    }

    if args.debug {
        println!("Debug mode enabled.");
        println!("Arguments: {:?}", args);
    }

    let listener = TcpListener::bind(format!("0.0.0.0:{}", args.port))?;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if args.verbose {
                    println!("New client connected: {}", stream.peer_addr()?);
                }
                let args_clone = args.clone();
                thread::spawn(move || {
                    if let Err(e) = handle_client(stream, args_clone) {
                        eprintln!("Error handling client: {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }

    Ok(())
}

/// Handles a single TCP client connection.
///
/// This function reads OSC command strings from the client, sends them to the X32 mixer via UDP,
/// and relays the mixer's response back to the client.
///
/// # Arguments
///
/// * `stream` - The TCP stream for the client connection.
/// * `args` - The command-line arguments.
fn handle_client(mut stream: TcpStream, args: Args) -> Result<()> {
    let x32_socket = create_socket(&args.ip, 10)?;

    let mut reader = BufReader::new(stream.try_clone()?);

    loop {
        let mut line = String::new();
        let len = reader.read_line(&mut line)?;
        if len == 0 {
            break; // Connection closed
        }

        let trimmed_line = line.trim();
        if trimmed_line.is_empty() {
            continue;
        }

        if args.verbose {
            println!("Received from client: {}", trimmed_line);
        }

        if trimmed_line == "exit" {
            break;
        }

        match OscMessage::from_str(trimmed_line) {
            Ok(osc_msg) => {
                let msg_bytes = osc_msg.to_bytes()?;
                x32_socket.send(&msg_bytes)?;

                let mut buf = [0; 1024];
                match x32_socket.recv(&mut buf) {
                    Ok(len) => {
                        let response_msg = OscMessage::from_bytes(&buf[..len])?;
                        let response_str = response_msg.to_string();
                        if args.verbose {
                            println!("Sending to client: {}", response_str);
                        }
                        stream.write_all(response_str.as_bytes())?;
                        stream.write_all(b"\n")?;
                    }
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::WouldBlock {
                            if args.verbose {
                                println!("No response from X32.");
                            }
                            stream.write_all(b"no data\n")?;
                        } else {
                            return Err(e.into());
                        }
                    }
                }
            }
            Err(e) => {
                let error_msg = format!("Error parsing OSC message: {}", e);
                eprintln!("{}", error_msg);
                stream.write_all(error_msg.as_bytes())?;
                stream.write_all(b"\n")?;
            }
        }
    }

    if args.verbose {
        println!("Client disconnected: {}", stream.peer_addr()?);
    }

    Ok(())
}
