
use anyhow::{anyhow, Result};
use clap::Parser;
use std::io::{self, BufRead};
use std::net::{SocketAddr, UdpSocket};
use std::thread;
use std::time::Duration;
use x32_lib::cparse;

/// A command-line tool for setting a scene on an X32 mixer from a scene file.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The IP address of the X32 console.
    #[arg(short, long, default_value = "192.168.0.64")]
    ip: String,

    /// Delay between commands in milliseconds.
    #[arg(short, long, default_value_t = 1)]
    delay: u64,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let socket = UdpSocket::bind("0.0.0.0:0")?;
    let x32_addr: SocketAddr = format!("{}:10023", args.ip).parse()?;
    socket.connect(x32_addr)?;

    println!("# X32SetScene ver. 1.10 (c)2017 Patrick-Gilles Maillot\n");

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.starts_with('/') {
            let command = cparse::xcparse(&line).map_err(|e| anyhow!(e))?;
            socket.send(&command)?;
            if args.delay > 0 {
                thread::sleep(Duration::from_millis(args.delay));
            }
        } else if line.starts_with('#') {
            if line.contains("#4.0#") {
                // Ignore header
            } else {
                eprintln!("Only ver. 4.0 files are accepted at this time");
                break;
            }
        } else if line == "exit" {
            break;
        }
    }

    Ok(())
}
