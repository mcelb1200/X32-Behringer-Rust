
use anyhow::Result;
use clap::Parser;
use std::net::{SocketAddr, UdpSocket};
use std::thread;

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

pub fn run(mut mixer: Mixer, ip: String, port: u16) -> Result<SocketAddr> {
    let addr: SocketAddr = format!("{}:{}", ip, port).parse()?;
    let socket = UdpSocket::bind(&addr)?;
    let local_addr = socket.local_addr()?;

    thread::spawn(move || {
        let mut buf = [0; 8192];
        loop {
            let (len, remote_addr) = socket.recv_from(&mut buf).unwrap();
            match mixer.dispatch(&buf[..len]) {
                Ok(Some(response)) => {
                    socket.send_to(&response, remote_addr).unwrap();
                }
                Ok(None) => {}
                Err(e) => {
                    eprintln!("Error handling message: {}", e);
                }
            }
        }
    });

    Ok(local_addr)
}
