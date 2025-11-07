//! `x32_get_scene_name` is a command-line utility that connects to a Behringer X32 digital
//! mixer, listens for scene change events, and prints the name of the new scene to standard
//! output. It is a Rust implementation of the original `GetSceneName.c` tool by
//! Patrick-gilles Maillot.

use clap::Parser;
use std::time::{Duration, Instant};
use x32_lib::{create_socket};
use x32_lib::error::{Result, X32Error};
use osc_lib::{OscMessage, OscArg};

/// A command line utility to get scene names when a scene change takes place.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// X32 console IP address
    #[arg(short, long, default_value = "192.168.1.62")]
    ip: String,

    /// Prints welcome and connection status messages (0 or 1)
    #[arg(short, long, default_value_t = 1)]
    verbose: u8,

    /// Exits at first occurrence (0 or 1)
    #[arg(short, long, default_value_t = 1)]
    onetime: u8,
}

/// The main entry point for the `x32_get_scene_name` utility.
///
/// This function parses command-line arguments, connects to the X32, subscribes to scene
/// change events, and prints the name of the new scene to standard output.
fn main() -> Result<()> {
    let args = Args::parse();

    if args.verbose != 0 {
        println!("GetSceneName - v0.2 - (c)2018 Patrick-Gilles Maillot");
        println!("Connecting to X32 at {}...", &args.ip);
    }

    let socket = create_socket(&args.ip, 500)?;
    socket.set_read_timeout(Some(Duration::from_millis(10)))?;

    let info_msg = OscMessage::new("/info".to_string(), vec![]);
    socket.send(&info_msg.to_bytes()?)?;

    let mut buf = [0; 512];
    loop {
        match socket.recv(&mut buf) {
            Ok(len) => {
                if let Ok(msg) = OscMessage::from_bytes(&buf[..len]) {
                    if msg.path == "/info" {
                        if args.verbose != 0 {
                            println!("Connected!");
                        }
                        break;
                    }
                }
            }
            Err(_) => {
                socket.send(&info_msg.to_bytes()?)?;
                if args.verbose != 0 {
                    print!(".");
                }
            }
        }
    }

    let xremote_msg = OscMessage::new("/xremote".to_string(), vec![]);
    let show_control_msg = OscMessage::new("/-prefs/show_control".to_string(), vec![OscArg::Int(1)]);

    let mut last_remote_sent = Instant::now();

    loop {
        if last_remote_sent.elapsed() >= Duration::from_secs(9) {
            socket.send(&xremote_msg.to_bytes()?)?;
            socket.send(&show_control_msg.to_bytes()?)?;
            last_remote_sent = Instant::now();
        }

        if let Ok(len) = socket.recv(&mut buf) {
            if let Ok(msg) = OscMessage::from_bytes(&buf[..len]) {
                if msg.path == "/-show/prepos/current" {
                    if let Some(OscArg::Int(scene_index)) = msg.args.get(0) {
                        let get_name_msg = OscMessage::new(format!("/-show/showfile/scene/{:03}", scene_index), vec![]);
                        socket.send(&get_name_msg.to_bytes()?)?;
                    }
                } else if msg.path.starts_with("/-show/showfile/scene") {
                    if let Some(OscArg::String(scene_name)) = msg.args.get(0) {
                         if let Some(OscArg::Int(scene_index)) = msg.args.get(1) {
                            println!("{:02} - {}", scene_index, scene_name);
                            if args.onetime != 0 {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
