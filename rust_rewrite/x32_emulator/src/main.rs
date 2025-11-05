use anyhow::Result;
use clap::Parser;
use std::net::{SocketAddr, UdpSocket};

use x32_core::Mixer;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// IP address to bind to
    #[arg(short, long, default_value_t = String::from("0.0.0.0"))]
    ip: String,

    /// Port number to bind to
    #[arg(short, long, default_value_t = 10023)]
    port: u16,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let addr: SocketAddr = format!("{}:{}", cli.ip, cli.port).parse()?;
    let socket = UdpSocket::bind(&addr)?;
    let mixer = Mixer::new();

    println!("X32 Emulator listening on {}", addr);

    let mut buf = [0; 8192];
    loop {
        let (len, remote_addr) = socket.recv_from(&mut buf)?;
        match mixer.dispatch(&buf[..len]) {
            Ok(Some(response)) => {
                socket.send_to(&response, remote_addr)?;
            }
            Ok(None) => {}
            Err(e) => {
                eprintln!("Error handling message: {}", e);
            }
        }
    }
}
