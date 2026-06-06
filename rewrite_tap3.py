import re

with open("tools/x32_tap/src/main.rs", "r") as f:
    text = f.read()

out = """//! `x32_tap` is a command-line tool for setting the tempo of X32 delay effects by tapping.
//!
//! It connects to the mixer and listens for user input (pressing Enter). By measuring the
//! time interval between taps, it calculates the tempo in milliseconds and updates the
//! corresponding parameter of the delay effect in the specified FX slot.

use anyhow::{Context, Result, anyhow};
use clap::Parser;
use osc_lib::{OscArg, OscMessage};
use std::io::{self, Write};
use std::sync::Arc;
use std::time::Instant;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::time::{interval, Duration};
use x32_lib::{MixerClient, get_parameter_async};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "192.168.0.64")]
    ip: String,

    #[arg(long, default_value = "auto")]
    transport: String,

    #[arg(long, default_value = "")]
    usb_port: String,

    #[arg(long, default_value = "")]
    aes50_ip: String,

    #[arg(short, long, default_value_t = 1)]
    slot: u8,

    #[arg(short, long, default_value_t = 1)]
    parameter: u8,

    #[arg(short, long, default_value_t = false)]
    auto: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    if args.slot < 1 || args.slot > 4 {
        return Err(anyhow!("FX slot must be between 1 and 4."));
    }

    let (client, _) = MixerClient::connect_with_transport(
        &args.ip,
        &args.aes50_ip,
        &args.usb_port,
        &args.transport,
        false,
    ).await?;
    let client = Arc::new(client);

    let mut rx = client.subscribe();
    client.send_message("/info", vec![]).await.context("Failed to send /info")?;

    match tokio::time::timeout(Duration::from_secs(2), rx.recv()).await {
        Ok(Ok(_)) => println!("Connected."),
        _ => eprintln!("Warning: Did not receive response to /info."),
    }

    handle_client(client, args).await
}

async fn handle_client(
    client: Arc<MixerClient>,
    args: Args,
) -> Result<()> {
    let mut last_tap: Option<Instant> = None;
    let mut t2_ms = 0.0;
    let mut last_t2 = 0.0;

    let fx_addr = format!("/fx/{}/par/{:02}", args.slot, args.parameter);

    // Test initial query
    if let Err(e) = get_parameter_async(&client, &fx_addr).await {
        eprintln!("Warning: Failed to query initial parameter state: {}", e);
    }

    let client_clone = client.clone();
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_millis(if args.auto { 50 } else { 100 }));
        loop {
            interval.tick().await;
            if client_clone.send_message("/info", vec![]).await.is_err() {
                eprintln!("Failed to send ping.");
            }
        }
    });

    println!("Ready to tap. Press Enter to tap, 'q' to quit.");
    if args.auto {
        println!("Auto mode enabled. Waiting for input stream...");
    }

    let stdin = io::stdin();
    let mut reader = BufReader::new(tokio::io::stdin());
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => break, // EOF
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed == "q" || trimmed == "quit" {
                    break;
                }

                let now = Instant::now();
                if let Some(last) = last_tap {
                    let elapsed = now.duration_since(last);
                    let elapsed_ms = elapsed.as_millis() as f32;

                    if elapsed_ms > 2000.0 {
                        // Timeout, reset
                        println!("Timeout (> 2s), starting new sequence.");
                        last_tap = Some(now);
                        t2_ms = 0.0;
                        continue;
                    } else if elapsed_ms < 10.0 {
                        // Ignore bounce
                        continue;
                    }

                    if t2_ms == 0.0 {
                        t2_ms = elapsed_ms;
                    } else {
                        // Moving average
                        t2_ms = (t2_ms + elapsed_ms) / 2.0;
                    }

                    if (t2_ms - last_t2).abs() > 1.0 {
                        let ms = t2_ms;
                        let value = if ms > 3000.0 {
                            1.0
                        } else if ms < 0.0 {
                            0.0
                        } else {
                            ms / 3000.0
                        };

                        if args.auto {
                            if let Err(e) = run_tap(&client, args.slot, args.parameter, value).await {
                                eprintln!("Error updating tap: {}", e);
                            }
                        } else {
                            print!("\\rTempo: {:.1} ms ({:.2})  ", ms, value);
                            io::stdout().flush().unwrap();
                            if let Err(e) = run_tap(&client, args.slot, args.parameter, value).await {
                                eprintln!("\\nError updating tap: {}", e);
                            }
                        }

                        last_t2 = t2_ms;
                    }
                }
                last_tap = Some(now);
            }
            Err(e) => {
                eprintln!("Error reading stdin: {}", e);
                break;
            }
        }
    }

    Ok(())
}

pub async fn run_tap(client: &MixerClient, fx_slot: u8, parameter: u8, value: f32) -> Result<()> {
    let addr = format!("/fx/{}/par/{:02}", fx_slot, parameter);
    x32_lib::set_parameter_async(client, &addr, value).await?;
    Ok(())
}
"""

with open("tools/x32_tap/src/main.rs", "w") as f:
    f.write(out)
