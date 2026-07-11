//! `x32_auto_gain` is a tool that monitors peak levels on selected channels
//! and automatically sets the HA (Headamp) gain to an optimal target (e.g., -18dBFS),
//! minimizing clipping for new operators.
use anyhow::Result;
use clap::Parser;
use osc_lib::OscArg;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::{Instant, interval};
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

    /// Target level in dBFS. If not set, inferred from instrument type (Smart Gain).
    #[arg(short, long)]
    pub target_dbfs: Option<f32>,

    /// Target Tolerance in dB (e.g. stop adjusting if within 2dB of target)
    #[arg(short = 'e', long, default_value_t = 2.0)]
    pub tolerance_db: f32,

    /// Max gain adjustment per step in dB (slow down the auto-gain)
    #[arg(short = 'm', long, default_value_t = 3.0)]
    pub max_step_db: f32,

    /// Polling rate in milliseconds
    #[arg(short, long, default_value_t = 100)]
    pub rate_ms: u64,

    /// Measurement window in milliseconds
    #[arg(short = 'w', long, default_value_t = 5000)]
    pub window_ms: u64,

    /// Minimum signal threshold in dBFS to measure (-60 by default)
    #[arg(short = 's', long, default_value_t = -60.0)]
    pub signal_threshold_dbfs: f32,
}

#[derive(Debug, Clone)]
pub struct TargetLevels {
    pub rms_dbfs: f32,
    pub peak_dbfs: f32,
}

pub fn infer_target_levels(name: &str, icon: i32) -> TargetLevels {
    let lower_name = name.to_lowercase();

    // Kick drum / Bass drum
    if lower_name.contains("kick") || lower_name.contains("bass drum") || icon == 1 {
        return TargetLevels {
            rms_dbfs: -12.0,
            peak_dbfs: -6.0,
        };
    }
    // Snare drum
    if lower_name.contains("snare") || icon == 2 {
        return TargetLevels {
            rms_dbfs: -14.0,
            peak_dbfs: -6.0,
        };
    }
    // Toms
    if lower_name.contains("tom") || icon == 3 {
        return TargetLevels {
            rms_dbfs: -14.0,
            peak_dbfs: -6.0,
        };
    }
    // Overhead / Cymbal
    if lower_name.contains("oh")
        || lower_name.contains("overhead")
        || lower_name.contains("cymbal")
        || icon == 5
    {
        return TargetLevels {
            rms_dbfs: -18.0,
            peak_dbfs: -10.0,
        };
    }
    // Bass guitar (DI)
    if lower_name.contains("bass") || icon == 6 {
        return TargetLevels {
            rms_dbfs: -14.0,
            peak_dbfs: -8.0,
        };
    }
    // Electric guitar (amp mic)
    if lower_name.contains("egtr")
        || lower_name.contains("elec gtr")
        || lower_name.contains("electric guitar")
        || icon == 8
    {
        return TargetLevels {
            rms_dbfs: -16.0,
            peak_dbfs: -8.0,
        };
    }
    // Acoustic guitar (mic)
    if lower_name.contains("agtr")
        || lower_name.contains("acou")
        || lower_name.contains("acoustic")
        || icon == 9
    {
        return TargetLevels {
            rms_dbfs: -18.0,
            peak_dbfs: -10.0,
        };
    }
    // Piano / Keys (DI/stereo)
    if lower_name.contains("piano")
        || lower_name.contains("keys")
        || lower_name.contains("synth")
        || icon == 11
    {
        return TargetLevels {
            rms_dbfs: -18.0,
            peak_dbfs: -8.0,
        };
    }
    // Lead vocal
    if lower_name.contains("vox") || lower_name.contains("vocal") || icon == 14 {
        return TargetLevels {
            rms_dbfs: -18.0,
            peak_dbfs: -8.0,
        };
    }
    // Speech / Lectern
    if lower_name.contains("speech")
        || lower_name.contains("lectern")
        || lower_name.contains("pastor")
        || lower_name.contains("mc")
        || icon == 16
    {
        return TargetLevels {
            rms_dbfs: -20.0,
            peak_dbfs: -10.0,
        };
    }
    // Choir / Ensemble (stereo)
    if lower_name.contains("choir") || lower_name.contains("ensemble") || icon == 15 {
        return TargetLevels {
            rms_dbfs: -20.0,
            peak_dbfs: -10.0,
        };
    }
    // Wireless lavalier
    if lower_name.contains("lav") || lower_name.contains("headset") || icon == 18 {
        return TargetLevels {
            rms_dbfs: -22.0,
            peak_dbfs: -12.0,
        };
    }
    // DJ / Playback (line level)
    if lower_name.contains("dj")
        || lower_name.contains("ipod")
        || lower_name.contains("mac")
        || lower_name.contains("pc")
        || lower_name.contains("track")
        || lower_name.contains("playback")
        || icon == 19
    {
        return TargetLevels {
            rms_dbfs: -14.0,
            peak_dbfs: -6.0,
        };
    }

    // Best-guess (unknown)
    TargetLevels {
        rms_dbfs: -18.0,
        peak_dbfs: -10.0,
    }
}

#[derive(Debug, Clone)]
pub enum ChannelState {
    Measuring {
        start_time: Option<Instant>,
        rms_sum: f32,
        rms_count: usize,
        max_peak: f32,
    },
    Adjusting {
        required_delta_db: f32,
    },
    Done,
}

pub async fn run(args: Args) -> Result<()> {
    // Parse channels
    let mut channels: Vec<u8> = Vec::new();
    for part in args.channels.split(',') {
        if let Ok(ch) = part.trim().parse::<u8>() {
            if ch >= 1 && ch <= 32 {
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
    let mut channel_states: HashMap<u8, ChannelState> = HashMap::new();
    let mut target_levels: HashMap<u8, TargetLevels> = HashMap::new();

    // Initial fetch of gains and info
    for ch in &channels {
        let path = format!("/headamp/{:02}/gain", ch);
        let val = client.query_value(&path).await?;
        if let OscArg::Float(f) = val {
            ha_gains.insert(*ch, f);
        }

        let mut name = String::new();
        let mut icon = 0;

        if let Ok(OscArg::String(n)) = client
            .query_value(&format!("/ch/{:02}/config/name", ch))
            .await
        {
            name = n;
        }
        if let Ok(OscArg::Int(i)) = client
            .query_value(&format!("/ch/{:02}/config/icon", ch))
            .await
        {
            icon = i;
        }

        target_levels.insert(*ch, infer_target_levels(&name, icon));
        channel_states.insert(
            *ch,
            ChannelState::Measuring {
                start_time: None,
                rms_sum: 0.0,
                rms_count: 0,
                max_peak: -144.0,
            },
        );
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
                        let parts: Vec<&str> = msg.path.split('/').collect();
                        if parts.len() == 4 {
                            if let Ok(ch) = parts[2].parse::<u8>() {
                                ha_gains.insert(ch, *f);
                            }
                        }
                    }
                }

                // Track phantom power changes to warn user
                if msg.path.starts_with("/headamp/") && msg.path.ends_with("/+48V") {
                    let parts: Vec<&str> = msg.path.split('/').collect();
                    if parts.len() == 4 {
                        if let Ok(ch) = parts[2].parse::<u8>() {
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

                            let current_db = if val > 0.00001 { 20.0 * val.log10() } else { -144.0 };

                            let mut state_update = None;

                            if let Some(state) = channel_states.get(ch) {
                                match state {
                                    ChannelState::Measuring { start_time, rms_sum, rms_count, max_peak } => {
                                        if current_db >= args.signal_threshold_dbfs {
                                            let mut new_start = *start_time;
                                            if start_time.is_none() {
                                                new_start = Some(Instant::now());
                                            }

                                            let new_rms_sum = *rms_sum + current_db;
                                            let new_rms_count = *rms_count + 1;
                                            let new_max_peak = max_peak.max(current_db);

                                            if let Some(st) = new_start {
                                                if st.elapsed().as_millis() as u64 >= args.window_ms {
                                                    // Measurement complete
                                                    let avg_rms = new_rms_sum / new_rms_count as f32;
                                                    println!("Ch {:02} measurement complete: RMS={:.1}dBFS, Peak={:.1}dBFS", ch, avg_rms, new_max_peak);

                                                    // Decide on adjustment
                                                    let target = args.target_dbfs.unwrap_or_else(|| {
                                                        target_levels.get(ch).map(|t| t.rms_dbfs).unwrap_or(-18.0)
                                                    });

                                                    if (avg_rms - target).abs() > args.tolerance_db {
                                                        state_update = Some(ChannelState::Adjusting {
                                                            required_delta_db: target - avg_rms,
                                                        });
                                                    } else {
                                                        println!("Ch {:02} is within tolerance of target {:.1}dBFS.", ch, target);
                                                        state_update = Some(ChannelState::Done);
                                                    }
                                                } else {
                                                    state_update = Some(ChannelState::Measuring {
                                                        start_time: new_start,
                                                        rms_sum: new_rms_sum,
                                                        rms_count: new_rms_count,
                                                        max_peak: new_max_peak,
                                                    });
                                                }
                                            }
                                        }
                                    }
                                    ChannelState::Adjusting { required_delta_db } => {
                                        // Wait until signal is active to prevent blind adjustments
                                        if current_db >= args.signal_threshold_dbfs {
                                            if current_db > -3.0 {
                                                // Clipping detected, cut gain and reset!
                                                println!("CLIP DETECTED on Ch {:02} ({:.1}dBFS)! Cutting gain by 6dB and restarting measurement.", ch, current_db);
                                                if let Some(current_osc) = ha_gains.get(ch) {
                                                    let delta_osc = -6.0 / 72.0;
                                                    let mut new_osc = current_osc + delta_osc;
                                                    if new_osc < 0.0 { new_osc = 0.0; }

                                                    let path = format!("/headamp/{:02}/gain", ch);
                                                    let _ = client.send_message(&path, vec![OscArg::Float(new_osc)]).await;
                                                    ha_gains.insert(*ch, new_osc);
                                                }
                                                state_update = Some(ChannelState::Measuring {
                                                    start_time: None,
                                                    rms_sum: 0.0,
                                                    rms_count: 0,
                                                    max_peak: -144.0,
                                                });
                                            } else {
                                                let mut delta_db = *required_delta_db;

                                                // Clamp max step
                                                if delta_db > args.max_step_db { delta_db = args.max_step_db; }
                                                if delta_db < -args.max_step_db { delta_db = -args.max_step_db; }

                                                let delta_osc = delta_db / 72.0;

                                                if let Some(current_osc) = ha_gains.get(ch) {
                                                    let mut new_osc = current_osc + delta_osc;
                                                    if new_osc < 0.0 { new_osc = 0.0; }
                                                    if new_osc > 1.0 { new_osc = 1.0; }

                                                    if (new_osc - current_osc).abs() > 0.005 {
                                                        println!("Ch {:02} Adjusting gain by {:.1}dB", ch, delta_db);
                                                        let path = format!("/headamp/{:02}/gain", ch);
                                                        let _ = client.send_message(&path, vec![OscArg::Float(new_osc)]).await;
                                                        ha_gains.insert(*ch, new_osc);
                                                    }
                                                }

                                                // After making an adjustment, go back to measuring to verify
                                                state_update = Some(ChannelState::Measuring {
                                                    start_time: None,
                                                    rms_sum: 0.0,
                                                    rms_count: 0,
                                                    max_peak: -144.0,
                                                });
                                            }
                                        }
                                    }
                                    ChannelState::Done => {}
                                }
                            }

                            if let Some(new_state) = state_update {
                                channel_states.insert(*ch, new_state);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
