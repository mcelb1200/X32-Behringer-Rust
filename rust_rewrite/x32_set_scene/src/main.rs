
use clap::Parser;
use std::io::{self, BufRead};
use std::thread;
use std::time::Duration;
use x32_lib::{create_socket, Result};
use osc_lib::OscMessage;

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
    let socket = create_socket(&args.ip, 100)?;

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.starts_with('/') {
            match OscMessage::from_str(&line) {
                Ok(msg) => {
                    socket.send(&msg.to_bytes()?)?;
                    if args.delay > 0 {
                        thread::sleep(Duration::from_millis(args.delay));
                    }
                }
                Err(e) => eprintln!("Error parsing line: {}", e),
            }
        }
    }

    Ok(())
}
