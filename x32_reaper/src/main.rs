use anyhow::{Context, Result};
use clap::Parser;
use std::sync::{Arc, Mutex};
use tokio::net::UdpSocket;
use tokio::time::{self, Duration};

mod config;
mod state;
mod x32_handler;
mod reaper_handler;

use config::{Config, load_config};
use state::AppState;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to configuration file
    #[arg(short, long, default_value = "config.json")]
    config: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Load config or use default
    let config = load_config(&args.config).unwrap_or_else(|_| {
        eprintln!("Could not load config file '{}', using defaults.", args.config);
        Config::default()
    });

    let x32_ip = format!("{}:10023", config.x32_ip);
    let reaper_dest = format!("{}:{}", config.reaper_ip, config.reaper_send_port);

    // X32 Socket: Bind to dynamic, connect to X32
    // Actually, standard practice for X32 client: bind 0.0.0.0:0 (or specific if needed), send to X32:10023
    let x32_socket = UdpSocket::bind("0.0.0.0:0").await.context("Failed to bind X32 socket")?;
    x32_socket.connect(&x32_ip).await.context("Failed to connect to X32")?;
    let x32_socket = Arc::new(x32_socket);

    // Reaper Socket: Bind to recv port
    let reaper_bind_addr = format!("0.0.0.0:{}", config.reaper_recv_port);
    let reaper_socket = UdpSocket::bind(&reaper_bind_addr).await.context("Failed to bind Reaper socket")?;
    let reaper_socket = Arc::new(reaper_socket);

    println!("X32 Reaper Bridge started.");
    println!("X32: {}", x32_ip);
    println!("Reaper: Listening on {}, Sending to {}", reaper_bind_addr, reaper_dest);

    let state = Arc::new(Mutex::new(AppState::new(config.clone())));

    // Keep-alive loop (/xremote)
    let ka_socket = x32_socket.clone();
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(9));
        loop {
            interval.tick().await;
            let msg = osc_lib::OscMessage::new("/xremote".to_string(), vec![]);
            if let Ok(bytes) = msg.to_bytes() {
                let _ = ka_socket.send(&bytes).await;
            }
        }
    });

    // Subscribe initially
    let msg = osc_lib::OscMessage::new("/xremote".to_string(), vec![]);
    x32_socket.send(&msg.to_bytes()?).await?;
    let msg = osc_lib::OscMessage::new("/info".to_string(), vec![]);
    x32_socket.send(&msg.to_bytes()?).await?;

    let mut x32_buf = [0u8; 2048];
    let mut reaper_buf = [0u8; 4096]; // Bundles can be large

    loop {
        tokio::select! {
            res = x32_socket.recv(&mut x32_buf) => {
                match res {
                    Ok(len) => {
                        if let Err(e) = x32_handler::handle_x32_message(&x32_buf, len, &state, &reaper_socket, &reaper_dest).await {
                            if config.verbose { eprintln!("Error handling X32 message: {}", e); }
                        }
                    }
                    Err(e) => eprintln!("X32 recv error: {}", e),
                }
            }
            res = reaper_socket.recv_from(&mut reaper_buf) => {
                match res {
                    Ok((len, _src)) => {
                        // We trust source for now, or could verify it matches reaper_ip
                        if let Err(e) = reaper_handler::handle_reaper_message(&reaper_buf, len, &state, &x32_socket, &x32_ip).await {
                            if config.verbose { eprintln!("Error handling Reaper message: {}", e); }
                        }
                    }
                    Err(e) => eprintln!("Reaper recv error: {}", e),
                }
            }
        }
    }
}
