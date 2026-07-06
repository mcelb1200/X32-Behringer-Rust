//! `x32_ssavergw` is a command-line tool that acts as a screen saver for the Behringer X32/M32.
//!
//! It monitors the X32 console for activity via OSC. If no activity is detected for a
//! specified amount of time, it saves the current brightness settings for the main LCD
//! screen and LEDs, and then sets them to their lowest values (0.0). When activity
//! is detected again, it restores the original brightness settings.
//!
//! # Credits
//!
//! *   **Original concept and work on the C library:** Patrick-Gilles Maillot
//! *   **Rust implementation by:** mcelb1200

use anyhow::Result;
use clap::Parser;
use osc_lib::OscArg;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{self, Duration};
use x32_lib::MixerClient;

pub mod state;
use state::AppState;

/// Set Low Light mode for X32 after a period of inactivity.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The IP address of the X32 console.
    #[arg(short, long, default_value = "192.168.0.64")]
    pub ip: String,

    /// Delay in seconds before entering Low Light mode.
    #[arg(short, long, default_value_t = 5)]
    pub delay: u64,
}

pub async fn run(args: Args) -> Result<()> {
    let addr = format!("{}:10023", args.ip);
    let client = MixerClient::connect(&addr, true).await?;

    println!("X32Ssaver - Rust Rewrite");
    println!("Connecting to X32 at {}...", args.ip);

    if client.probe().await {
        println!(
            "Connected. Waiting {}s before activating screen saver...",
            args.delay
        );
    } else {
        return Err(anyhow::anyhow!(
            "No response from X32 at {}. Check IP.",
            args.ip
        ));
    }

    let mut rx = client.subscribe();
    let state = Arc::new(Mutex::new(AppState::new(args.delay)));

    let state_clone = state.clone();
    let addr_clone = addr.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            println!("Shutting down... Restoring brightness.");
            let st = state_clone.lock().await;
            if st.is_dimmed {
                if let Ok(client) = MixerClient::connect(&addr_clone, false).await {
                    let _ = restore_brightness(&client, &st).await;
                }
            }
            std::process::exit(0);
        }
    });

    let mut check_interval = time::interval(Duration::from_millis(500));

    loop {
        tokio::select! {
            res = rx.recv() => {
                match res {
                    Ok(msg) => {
                        if !msg.path.starts_with("/-") && msg.path != "/xremote" {
                            // Activity!
                            let mut st = state.lock().await;
                            st.mark_activity();
                            if st.is_dimmed {
                                restore_brightness(&client, &st).await?;
                                st.is_dimmed = false;
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(_) => break,
                }
            }
            _ = check_interval.tick() => {
                let mut st = state.lock().await;
                if st.should_dim() {
                    save_and_dim(&client, &mut st).await?;
                    st.is_dimmed = true;
                }
            }
        }
    }
    Ok(())
}

async fn save_and_dim(client: &MixerClient, state: &mut AppState) -> Result<()> {
    println!("Saving brightness and dimming...");

    // Ask for current values
    let paths = vec!["/-prefs/lcd/bright", "/-prefs/led/bright"];

    for path in &paths {
        if let Ok(OscArg::Float(val)) = client.query_value(path).await {
            match *path {
                "/-prefs/lcd/bright" => state.saved_lcd_bright = val,
                "/-prefs/led/bright" => state.saved_led_bright = val,
                _ => {}
            }
        }
    }

    // Now set them to 0.0
    for path in &paths {
        client.send_message(path, vec![OscArg::Float(0.0)]).await?;
    }

    Ok(())
}

async fn restore_brightness(client: &MixerClient, state: &AppState) -> Result<()> {
    println!("Activity detected. Restoring brightness...");

    let map = vec![
        ("/-prefs/lcd/bright", state.saved_lcd_bright),
        ("/-prefs/led/bright", state.saved_led_bright),
    ];

    for (path, val) in map {
        client.send_message(path, vec![OscArg::Float(val)]).await?;
    }

    Ok(())
}
