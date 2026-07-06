use anyhow::{Context, Result};
use clap::Parser;
use osc_lib::OscMessage;
use std::str::FromStr;
use std::time::Duration;
use x32_lib::MixerClient;
use tokio::time::timeout;

/// A simple UDP client for sending OSC messages to the X32 mixer and receiving responses.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The IP address of the X32 mixer.
    #[arg(short, long, default_value = "192.168.0.64")]
    pub ip: String,

    /// The UDP port to connect to.
    #[arg(short, long, default_value_t = 10023)]
    pub port: u16,

    /// The OSC command to send.
    #[arg(default_value = "/status")]
    pub command: String,

    /// Timeout in milliseconds for receiving a response.
    #[arg(short, long, default_value_t = 1000)]
    pub timeout: u64,
}

pub async fn run(args: Args) -> Result<()> {
    println!("Connection status: 1");

    let addr = format!("{}:{}", args.ip, args.port);
    let client = MixerClient::connect(&addr, false)
        .await
        .context(format!("Failed to connect to {}", addr))?;

    let mut rx = client.subscribe();

    let msg = match OscMessage::from_str(&args.command) {
        Ok(msg) => msg,
        Err(e) => {
            eprintln!("Failed to parse command: {}", e);
            return Ok(());
        }
    };

    let payload_len = msg.to_bytes().context("Failed to serialize OSC message")?.len();

    match client.send_message(&msg.path, msg.args).await {
        Ok(_) => {
            println!("Send status: {}", payload_len);

            let timeout_duration = Duration::from_millis(args.timeout);

            let start = tokio::time::Instant::now();
            let mut received = false;
            while start.elapsed() < timeout_duration {
                match timeout(timeout_duration.saturating_sub(start.elapsed()), rx.recv()).await {
                    Ok(Ok(recv_msg)) => {
                        let recv_buf = recv_msg.to_bytes().unwrap_or_default();
                        println!("Recv status: {}", recv_buf.len());
                        for &byte in &recv_buf {
                            if byte < b' ' {
                                print!("~");
                            } else {
                                print!("{}", byte as char);
                            }
                        }
                        println!();
                        received = true;
                        break;
                    }
                    Ok(Err(tokio::sync::broadcast::error::RecvError::Lagged(_))) => continue,
                    Ok(Err(e)) => {
                        eprintln!("Error receiving data: {}", e);
                        break;
                    }
                    Err(_) => break,
                }
            }
            if !received {
                println!("Recv status: 0");
                println!("Receive timeout.");
            }
        }
        Err(e) => {
            eprintln!("Error sending data: {}", e);
        }
    }

    Ok(())
}
