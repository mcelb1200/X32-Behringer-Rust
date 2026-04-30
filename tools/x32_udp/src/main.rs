use anyhow::{Context, Result};
use clap::Parser;
use osc_lib::OscMessage;
use std::str::FromStr;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::timeout;

/// A simple UDP client for sending OSC messages to the X32 mixer and receiving responses.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The IP address of the X32 mixer.
    #[arg(short, long, default_value = "192.168.0.64")]
    ip: String,

    /// The UDP port to connect to.
    #[arg(short, long, default_value_t = 10023)]
    port: u16,

    /// The OSC command to send.
    #[arg(default_value = "/status")]
    command: String,

    /// Timeout in milliseconds for receiving a response.
    #[arg(short, long, default_value_t = 1000)]
    timeout: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    println!("Connection status: 1");

    let socket = UdpSocket::bind("0.0.0.0:0")
        .await
        .context("Failed to bind UDP socket")?;

    let addr = format!("{}:{}", args.ip, args.port);
    socket
        .connect(&addr)
        .await
        .context(format!("Failed to connect to {}", addr))?;

    let msg = match OscMessage::from_str(&args.command) {
        Ok(msg) => msg,
        Err(e) => {
            eprintln!("Failed to parse command: {}", e);
            return Ok(());
        }
    };

    let payload = msg.to_bytes().context("Failed to serialize OSC message")?;

    match socket.send(&payload).await {
        Ok(len) => {
            println!("Send status: {}", len);

            let mut buf = vec![0u8; 512];
            let timeout_duration = Duration::from_millis(args.timeout);

            match timeout(timeout_duration, socket.recv(&mut buf)).await {
                Ok(Ok(recv_len)) => {
                    println!("Recv status: {}", recv_len);
                    for &byte in &buf[..recv_len] {
                        if byte < b' ' {
                            print!("~");
                        } else {
                            print!("{}", byte as char);
                        }
                    }
                    println!();
                }
                Ok(Err(e)) => {
                    eprintln!("Error receiving data: {}", e);
                }
                Err(_) => {
                    println!("Recv status: 0");
                    println!("Receive timeout.");
                }
            }
        }
        Err(e) => {
            eprintln!("Error sending data: {}", e);
        }
    }

    Ok(())
}
