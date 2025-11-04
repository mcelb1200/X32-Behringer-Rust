
use anyhow::{anyhow, Result};
use clap::Parser;
use std::io::{self, Write};
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};
use osc_lib::{OscMessage, OscArg};

/// A command-line tool for getting the name of the currently loaded scene on an X32 mixer.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The IP address of the X32 console.
    #[arg(short, long, default_value = "192.168.1.62")]
    ip: String,

    /// Exit after the first scene change.
    #[arg(short, long, default_value_t = true)]
    one_time: bool,

    /// Enable verbose output.
    #[arg(short, long, default_value_t = true)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_read_timeout(Some(Duration::from_millis(500)))?;

    let x32_addr: SocketAddr = format!("{}:10023", args.ip).parse()?;
    socket.connect(x32_addr)?;

    if args.verbose {
        println!("GetSceneName - v0.2 - (c)2018 Patrick-Gilles Maillot");
        println!("Connecting to X32...");
    }

    let info_cmd = OscMessage::new("/info".to_string(), vec![]).to_bytes().map_err(|e: String| anyhow!(e))?;
    loop {
        socket.send(&info_cmd)?;
        let mut buf = [0; 512];
        if let Ok(len) = socket.recv(&mut buf) {
            let msg = OscMessage::from_bytes(&buf[..len]).map_err(|e: String| anyhow!(e))?;
            if msg.path == "/info" {
                break;
            }
        }
        if args.verbose {
            print!(".");
            io::stdout().flush()?;
        }
    }

    if args.verbose {
        println!(" Done!");
    }

    socket.set_read_timeout(Some(Duration::from_millis(1)))?;

    let mut last_xremote_time = Instant::now();
    let mut scene_index = 0;

    loop {
        if last_xremote_time.elapsed() > Duration::from_secs(9) {
            let xremote_cmd = OscMessage::new("/xremote".to_string(), vec![]).to_bytes().map_err(|e: String| anyhow!(e))?;
            socket.send(&xremote_cmd)?;
            let show_control_cmd = OscMessage::new("/-prefs/show_control".to_string(), vec![OscArg::Int(1)]).to_bytes().map_err(|e: String| anyhow!(e))?;
            socket.send(&show_control_cmd)?;
            last_xremote_time = Instant::now();
        }

        let mut buf = [0; 512];
        if let Ok(len) = socket.recv(&mut buf) {
            let msg = OscMessage::from_bytes(&buf[..len]).map_err(|e: String| anyhow!(e))?;
            if msg.path == "/-show/prepos/current" {
                if let Some(OscArg::Int(index)) = msg.args.get(0) {
                    scene_index = *index;
                    let scene_cmd = OscMessage::new(format!("/-show/showfile/scene/{:03}", scene_index), vec![]).to_bytes().map_err(|e: String| anyhow!(e))?;
                    socket.send(&scene_cmd)?;
                }
            } else if msg.path.starts_with("/-show/showfile/scene") {
                if let Some(OscArg::String(name)) = msg.args.get(0) {
                    println!("{:02} - {}", scene_index, name);
                    if args.one_time {
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}
