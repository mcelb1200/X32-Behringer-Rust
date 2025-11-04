
use anyhow::{anyhow, Result};
use clap::Parser;
use std::io::{self, BufRead, Write};
use std::net::{SocketAddr, UdpSocket};
use std::thread;
use std::time::Duration;
use osc_lib::{OscMessage, OscArg};

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
            let mut parts = line.splitn(2, ' ');
            let path = parts.next().unwrap_or("").to_string();
            let mut args_vec = Vec::new();
            if let Some(mut args_str) = parts.next() {
                while !args_str.is_empty() {
                    args_str = args_str.trim_start();
                    if args_str.starts_with('"') {
                        let end_quote = args_str[1..].find('"').ok_or_else(|| anyhow!("Unmatched quote"))?;
                        args_vec.push(OscArg::String(args_str[1..end_quote + 1].to_string()));
                        args_str = &args_str[end_quote + 2..];
                    } else {
                        let next_space = args_str.find(' ').unwrap_or(args_str.len());
                        let arg = &args_str[..next_space];
                        if let Ok(i) = arg.parse::<i32>() {
                            args_vec.push(OscArg::Int(i));
                        } else if let Ok(f) = arg.parse::<f32>() {
                            args_vec.push(OscArg::Float(f));
                        } else {
                            args_vec.push(OscArg::String(arg.to_string()));
                        }
                        args_str = &args_str[next_space..];
                    }
                }
            }
            let msg = OscMessage::new(path, args_vec);
            let command = msg.to_bytes().map_err(|e: String| anyhow!(e))?;
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
