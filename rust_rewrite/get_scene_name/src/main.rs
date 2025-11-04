
use anyhow::{anyhow, Result};
use clap::Parser;
use std::io::{self, Write};
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};
use x32_lib::cparse;

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

    let info_cmd = cparse::xcparse("/info").map_err(|e| anyhow!(e))?;
    loop {
        socket.send(&info_cmd)?;
        let mut buf = [0; 512];
        if let Ok(len) = socket.recv(&mut buf) {
            if &buf[..len] == b"/info" {
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
            let xremote_cmd = cparse::xcparse("/xremote").map_err(|e| anyhow!(e))?;
            socket.send(&xremote_cmd)?;

            let show_control_cmd = cparse::xcparse("/-prefs/show_control,i,1").map_err(|e| anyhow!(e))?;
            socket.send(&show_control_cmd)?;

            last_xremote_time = Instant::now();
        }

        let mut buf = [0; 512];
        if let Ok(len) = socket.recv(&mut buf) {
            let response = &buf[..len];
            if response.starts_with(b"/-show/prepos/current") {
                if let Ok(index) = std::str::from_utf8(&response[32..36])?.parse::<i32>() {
                    scene_index = index;
                    let scene_cmd = cparse::xcparse(&format!("/-show/showfile/scene/{:03}", scene_index)).map_err(|e| anyhow!(e))?;
                    socket.send(&scene_cmd)?;
                }
            } else if response.starts_with(b"/-show/showfile/scene") {
                if let Some(name_start) = response.iter().position(|&b| b == 0) {
                    let name = String::from_utf8_lossy(&response[name_start + 1..]);
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
