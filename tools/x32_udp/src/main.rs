use anyhow::Context;
use clap::Parser;
use osc_lib::OscMessage;
use std::str::FromStr;
use std::time::Duration;
use x32_lib::MixerClient;

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
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    println!("Connection status: 1");

    let client = MixerClient::connect(&args.ip, true).await?;

    let msg = match OscMessage::from_str(&args.command) {
        Ok(msg) => msg,
        Err(e) => {
            eprintln!("Failed to parse command: {}", e);
            return Ok(());
        }
    };

    let _payload = msg.to_bytes().context("Failed to serialize OSC message")?;

    match client.send_message(&msg.path, msg.args).await {
        Ok(_len) => {
            println!("Send status: OK");

            let _buf = vec![0u8; 512];
            let timeout_duration = Duration::from_millis(args.timeout);

            match tokio::time::timeout(timeout_duration, client.query_value(&msg.path)).await {
                Ok(Ok(recv_len)) => {
                    println!("Recv status: {:?}", recv_len);

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
