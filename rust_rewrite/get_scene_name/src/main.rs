
use clap::Parser;
use std::time::{Duration, Instant};
use x32_lib::{create_socket, Result};
use osc_lib::{OscMessage, OscArg};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The IP address of the X32 console.
    #[arg(short, long, default_value = "192.168.0.64")]
    ip: String,

    /// Verbose mode.
    #[arg(short, long, default_value_t = true)]
    verbose: bool,

    /// Exit after the first scene change.
    #[arg(short, long, default_value_t = true)]
    onetime: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let socket = create_socket(&args.ip, 1)?;

    if args.verbose {
        println!("GetSceneName - v0.2 - (c)2018 Patrick-Gilles Maillot\n");
        println!("Connecting to X32 at {}...", args.ip);
    }

    // Connect to the X32
    let msg = OscMessage::new("/info".to_string(), vec![]);
    socket.send(&msg.to_bytes()?)?;
    let mut buf = [0; 512];
    socket.recv(&mut buf)?;

    if args.verbose {
        println!("Connected!");
    }

    // Subscribe to scene changes
    let subscribe_msg = OscMessage::new("/-prefs/show_control".to_string(), vec![OscArg::Int(1)]);
    socket.send(&subscribe_msg.to_bytes()?)?;

    let mut last_xremote = Instant::now();

    loop {
        // Send /xremote every 9 seconds to keep the connection alive
        if last_xremote.elapsed() > Duration::from_secs(9) {
            let xremote_msg = OscMessage::new("/xremote".to_string(), vec![]);
            socket.send(&xremote_msg.to_bytes()?)?;
            socket.send(&subscribe_msg.to_bytes()?)?;
            last_xremote = Instant::now();
        }

        if let Ok(len) = socket.recv(&mut buf) {
            let response = OscMessage::from_bytes(&buf[..len])?;

            if response.path == "/-show/prepos/current" {
                if let Some(OscArg::Int(scene_index)) = response.args.get(0) {
                    let get_name_msg = OscMessage::new(format!("/-show/showfile/scene/{:03}", scene_index), vec![]);
                    socket.send(&get_name_msg.to_bytes()?)?;
                }
            } else if response.path.starts_with("/-show/showfile/scene") {
                if let Some(OscArg::String(scene_name)) = response.args.get(0) {
                    println!("{}", scene_name);
                    if args.onetime {
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}
