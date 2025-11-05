
use clap::Parser;
use x32_lib::{create_socket, Result};
use osc_lib::OscMessage;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The IP address of the X32 console.
    #[arg(short, long, default_value = "192.168.0.64")]
    ip: String,

    /// The OSC command string to send.
    #[arg(index = 1)]
    command: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let socket = create_socket(&args.ip, 100)?;

    let msg = OscMessage::from_str(&args.command)?;
    socket.send(&msg.to_bytes()?)?;

    let mut buf = [0; 512];
    if let Ok(len) = socket.recv(&mut buf) {
        let response = OscMessage::from_bytes(&buf[..len])?;
        println!("{}", response.to_string());
    }

    Ok(())
}
