//! `x32_auto_gain` is a tool that monitors peak levels on selected channels
//! and automatically sets the HA (Headamp) gain to an optimal target (e.g., -18dBFS),
//! minimizing clipping for new operators.
use anyhow::Result;
use clap::Parser;
use osc_lib::OscArg;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::interval;
use x32_lib::MixerClient;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about = "Auto-Gain / Smart Gain Staging for X32/M32", long_about = None)]
pub struct Args {
    /// IP address of the X32 console
    #[arg(short, long)]
    pub ip: String,

    /// Comma-separated list of channel numbers (1-32) to monitor and auto-gain (e.g. 1,2,3)
    #[arg(short, long)]
    pub channels: String,

    /// Target level in dBFS (e.g. -18.0)
    #[arg(short, long, default_value_t = -18.0)]
    pub target_dbfs: f32,

    /// Target Tolerance in dB (e.g. stop adjusting if within 2dB of target)
    #[arg(short = 'e', long, default_value_t = 2.0)]
    pub tolerance_db: f32,

    /// Max gain adjustment per step in dB (slow down the auto-gain)
    #[arg(short = 'm', long, default_value_t = 1.0)]
    pub max_step_db: f32,

    /// Polling rate in milliseconds
    #[arg(short, long, default_value_t = 100)]
    pub rate_ms: u64,
}

pub async fn run(args: Args) -> Result<()> {
    // Parse channels
    let mut channels: Vec<u8> = Vec::new();
    for part in args.channels.split(',') {
        if let Ok(ch) = part.trim().parse::<u8>() {
            if (1..=32).contains(&ch) {
                channels.push(ch);
            }
        }
    }

    if channels.is_empty() {
        println!("No valid channels provided. Expected format: --channels 1,2,3");
        return Ok(());
    }

    println!("Connecting to {}...", args.ip);
    let ip = if args.ip.contains(':') {
        args.ip.clone()
    } else {
        format!("{}:10023", args.ip)
    };
    let client = MixerClient::connect(&ip, true).await?;
    println!("Connected. Monitoring channels: {:?}", channels);

    let mut ticker = interval(Duration::from_millis(args.rate_ms));

    // We need a map to store current HA gains.
    // /headamp/01/gain etc
    let mut ha_gains: HashMap<u8, f32> = HashMap::new();

    // Initial fetch of gains
    for ch in &channels {
        let path = format!("/headamp/{:02}/gain", ch);
        let val = client.query_value(&path).await?;
        if let OscArg::Float(f) = val {
            ha_gains.insert(*ch, f);
        }
    }

    let mut rx = client.subscribe();

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                // Request meter update
                let _ = client.send_message("/meters", vec![OscArg::String("/meters/1".to_string())]).await;
            }
            msg = rx.recv() => {
                let msg = match msg {
                    Ok(m) => m,
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(_) => break,
                };

                // Track HA gain manual changes
                if msg.path.starts_with("/headamp/") && msg.path.ends_with("/gain") {
                    if let Some(OscArg::Float(f)) = msg.args.first() {
                        // ⚡ Bolt: Use .nth(2) instead of .collect::<Vec<&str>>() to avoid heap allocation
                        // in the hot network loop when parsing OSC messages.
                        if let Some(ch_str) = msg.path.split('/').nth(2) {
                            if let Ok(ch) = ch_str.parse::<u8>() {
                                ha_gains.insert(ch, *f);
                            }
                        }
                    }
                }

                // Track phantom power changes to warn user
                if msg.path.starts_with("/headamp/") && msg.path.ends_with("/+48V") {
                    // ⚡ Bolt: Use .nth(2) instead of .collect::<Vec<&str>>() to avoid heap allocation
                    // in the hot network loop when parsing OSC messages.
                    if let Some(ch_str) = msg.path.split('/').nth(2) {
                        if let Ok(ch) = ch_str.parse::<u8>() {
                            if channels.contains(&ch) {
                                println!("WARNING: Phantom power (+48V) changed on Ch {:02} during Auto-Gain. This can cause severe audio transients!", ch);
                            }
                        }
                    }
                }
                // Process meter updates
                if msg.path == "/meters/1" {
                    if let Some(OscArg::Blob(data)) = msg.args.first() {
                        // skip first 4 bytes (length)
                        if data.len() < 4 + 32 * 4 {
                            continue;
                        }

                        for ch in &channels {
                            let idx = *ch as usize - 1;
                            let start = 4 + idx * 4;
                            let bytes: [u8; 4] = data[start..start+4].try_into().unwrap_or([0; 4]);
                            let val = f32::from_le_bytes(bytes);

                            if val > 0.00001 {
                                let current_db = 20.0 * val.log10();

                                if (current_db - args.target_dbfs).abs() > args.tolerance_db {
                                    // Need to adjust
                                    let mut delta_db = args.target_dbfs - current_db;

                                    // Clamp max step
                                    if delta_db > args.max_step_db { delta_db = args.max_step_db; }
                                    if delta_db < -args.max_step_db { delta_db = -args.max_step_db; }

                                    // HA gain on X32 goes from -12dB to +60dB in 0.5dB steps.
                                    // The OSC value is 0.0 to 1.0 mapping linearly to 72dB range (-12 to +60).
                                    // So 1.0 = 72dB -> 1 unit = 72dB -> 1 dB = 1/72 units.
                                    let delta_osc = delta_db / 72.0;

                                    if let Some(current_osc) = ha_gains.get(ch) {
                                        let new_osc = (current_osc + delta_osc).clamp(0.0, 1.0);
                                        let mut new_osc = current_osc + delta_osc;
                                        if new_osc < 0.0 { new_osc = 0.0; }
                                        if new_osc > 1.0 { new_osc = 1.0; }

                                        // Update only if changed significantly
                                        if (new_osc - current_osc).abs() > 0.005 {
                                            println!("Ch {:02} level {:.1}dBFS. Adjusting gain by {:.1}dB", ch, current_db, delta_db);
                                            let path = format!("/headamp/{:02}/gain", ch);
                                            let _ = client.send_message(&path, vec![OscArg::Float(new_osc)]).await;
                                            // Update our local cache directly to prevent over-adjusting in rapid succession
                                            ha_gains.insert(*ch, new_osc);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
