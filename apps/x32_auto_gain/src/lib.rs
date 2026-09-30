//! `x32_auto_gain` is a tool that monitors peak levels on selected channels
//! and automatically sets the HA (Headamp) gain to an optimal target (e.g., -18dBFS),
//! minimizing clipping for new operators.

pub mod profile;
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

    // Maps for channel metadata, targets, gains, and stereo links
    let mut ha_gains: HashMap<u8, f32> = HashMap::new();
    let mut channel_targets: HashMap<u8, f32> = HashMap::new();
    let mut channel_links: HashMap<u8, u8> = HashMap::new();

    // Initial fetch of gains and scribble strip metadata for target calculation
    for ch in &channels {
        let gain_path = format!("/headamp/{:02}/gain", ch);
        if let Ok(OscArg::Float(f)) = client.query_value(&gain_path).await {
            ha_gains.insert(*ch, f);
        }

        let name_path = format!("/ch/{:02}/config/name", ch);
        let name = match client.query_value(&name_path).await {
            Ok(OscArg::String(s)) => s,
            _ => String::new(),
        };

        let icon_path = format!("/ch/{:02}/config/icon", ch);
        let icon_id = match client.query_value(&icon_path).await {
            Ok(OscArg::Int(i)) => i,
            _ => 0,
        };

        let profile = profile::match_instrument_profile(&name, icon_id);
        println!(
            "Ch {:02} Profile: {} (Target: {:.1} dBFS)",
            ch, profile.name, profile.target_dbfs
        );
        channel_targets.insert(*ch, profile.target_dbfs);
    }

    // Query stereo link configuration (/config/chlink/1-2 to 31-32)
    for pair_start in (1..=31).step_by(2) {
        let ch1 = pair_start as u8;
        let ch2 = pair_start as u8 + 1;
        let link_path = format!("/config/chlink/{}-{}", ch1, ch2);
        if let Ok(OscArg::Int(is_linked)) = client.query_value(&link_path).await {
            if is_linked == 1 {
                channel_links.insert(ch1, ch2);
                channel_links.insert(ch2, ch1);
            }
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
                            let bytes: [u8; 4] = match data.get(start..start + 4) {
                                Some(slice) => slice.try_into().unwrap_or([0; 4]),
                                None => continue,
                            };
                            let val = f32::from_le_bytes(bytes);

                            if val > 0.00001 {
                                let current_db = 20.0 * val.log10();
                                let target_dbfs = channel_targets.get(ch).copied().unwrap_or(args.target_dbfs);

                                // Clip Protection: If peak exceeds -3.0 dBFS, immediately drop gain by 6.0 dB
                                let delta_db = if current_db > -3.0 {
                                    println!("CLIP PROTECTION: Ch {:02} peak {:.1} dBFS (> -3.0 dBFS). Dropping gain by -6.0 dB!", ch, current_db);
                                    -6.0
                                } else if (current_db - target_dbfs).abs() > args.tolerance_db {
                                    let mut step = target_dbfs - current_db;
                                    if step > args.max_step_db { step = args.max_step_db; }
                                    if step < -args.max_step_db { step = -args.max_step_db; }
                                    step
                                } else {
                                    0.0
                                };

                                if delta_db.abs() > 0.01 {
                                    let delta_osc = delta_db / 72.0;

                                    if let Some(&current_osc) = ha_gains.get(ch) {
                                        let new_osc = (current_osc + delta_osc).clamp(0.0, 1.0);

                                        if (new_osc - current_osc).abs() > 0.005 {
                                            println!("Ch {:02} level {:.1}dBFS. Adjusting gain by {:.1}dB", ch, current_db, delta_db);
                                            let path = format!("/headamp/{:02}/gain", ch);
                                            let _ = client.send_message(&path, vec![OscArg::Float(new_osc)]).await;
                                            ha_gains.insert(*ch, new_osc);

                                            // Stereo Link Sync: Apply identical gain to paired channel if linked
                                            if let Some(&linked_ch) = channel_links.get(ch) {
                                                let linked_path = format!("/headamp/{:02}/gain", linked_ch);
                                                let _ = client.send_message(&linked_path, vec![OscArg::Float(new_osc)]).await;
                                                ha_gains.insert(linked_ch, new_osc);
                                                println!("Stereo link sync: Ch {:02} gain synced with Ch {:02}", linked_ch, ch);
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
    }

    Ok(())
}
