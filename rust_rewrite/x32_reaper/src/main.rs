
use clap::Parser;
use std::fs;
use std::time::{Duration, Instant};
use std::thread;
use x32_lib::create_socket;

mod config;
use config::Config;
mod reaper_handler;
mod x32_handler;
mod state;
use state::AppState;

#[cfg(test)]
mod tests;

/// A bridge between the Behringer X32/Midas M32 and Reaper.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the configuration file
    #[arg(short, long, default_value = "config.json")]
    config: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let config = if let Ok(config_data) = fs::read_to_string(&args.config) {
        serde_json::from_str(&config_data)?
    } else {
        let config = Config::default();
        fs::write(&args.config, serde_json::to_string_pretty(&config)?)?;
        config
    };

    println!("Configuration loaded from {}", args.config);

    let mut app_state = AppState::new(&config);

    // X32 Socket
    let x32_socket = create_socket(&config.x32_ip, 100)?;
    x32_socket.set_nonblocking(true)?;


    // Reaper Socket
    let reaper_addr = format!("{}:{}", config.reaper_ip, config.reaper_send_port);
    let reaper_socket = create_socket(&reaper_addr, 100)?;
    reaper_socket.set_nonblocking(true)?;

    println!("Connected to X32 at {}", config.x32_ip);
    println!("Connected to Reaper at {}:{}", config.reaper_ip, config.reaper_send_port);

    let mut last_xremote = Instant::now();
    let mut x32_buf = [0; 8192];
    let mut reaper_buf = [0; 8192];

    loop {
        // Keep-alive for X32
        if last_xremote.elapsed() > Duration::from_secs(9) {
            let msg = osc_lib::OscMessage::new("/xremote".to_string(), vec![]);
            x32_socket.send(&msg.to_bytes()?)?;
            last_xremote = Instant::now();
        }

        // Check for messages from X32
        if let Ok(len) = x32_socket.recv(&mut x32_buf) {
            let messages = x32_handler::handle_x32_message(&x32_buf[..len], &config, &mut app_state)?;
            for msg in messages {
                reaper_socket.send(&msg.to_bytes()?)?;
            }
        }

        // Check for messages from Reaper
        if let Ok(len) = reaper_socket.recv(&mut reaper_buf) {
            let messages = reaper_handler::handle_reaper_message(&reaper_buf[..len], &config, &mut app_state)?;
            for msg in messages {
                x32_socket.send(&msg.to_bytes()?)?;
            }
        }

        thread::sleep(Duration::from_millis(1));
    }
}
