//! `x32_auto_gain` is a tool that monitors peak levels on selected channels
//! and automatically sets the HA (Headamp) gain to an optimal target (e.g., -18dBFS),
//! minimizing clipping for new operators across digital mixer models (X32, Wing, XR series).

pub mod profile;
use anyhow::Result;
use clap::Parser;
use osc_lib::OscArg;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::interval;
use x32_lib::{MixerClient, MixerModel};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about = "Auto-Gain / Smart Gain Staging for X32/M32/Wing/XAir", long_about = None)]
pub struct Args {
    /// IP address of the X32/M32/Wing console
    #[arg(short, long)]
    pub ip: String,

    /// Comma-separated list of channel numbers to monitor and auto-gain (e.g. 1,2,3)
    #[arg(short, long)]
    pub channels: String,

    /// Mixer model: X32, Wing, XR18, XR16, XR12
    #[arg(short = 'M', long, default_value = "X32")]
    pub model: MixerModel,

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

pub fn max_channels_for_model(model: MixerModel) -> u8 {
    match model {
        MixerModel::X32 => 32,
        MixerModel::Wing => 40,
        MixerModel::XR18 | MixerModel::XR16 => 16,
        MixerModel::XR12 => 12,
    }
}

pub fn parse_channels_for_model(channels_str: &str, model: MixerModel) -> Vec<u8> {
    let max_ch = max_channels_for_model(model);
    let mut channels = Vec::new();
    for part in channels_str.split(',') {
        if let Ok(ch) = part.trim().parse::<u8>() {
            if ch >= 1 && ch <= max_ch {
                channels.push(ch);
            }
        }
    }
    channels
}

pub fn get_headamp_gain_path(model: MixerModel, channel: u8) -> String {
    match model {
        MixerModel::X32 | MixerModel::Wing => format!("/headamp/{:02}/gain", channel),
        MixerModel::XR18 | MixerModel::XR16 | MixerModel::XR12 => {
            format!("/headamp/{:02}/gain", channel)
        }
    }
}

pub fn get_headamp_phantom_path(_model: MixerModel, channel: u8) -> String {
    format!("/headamp/{:02}/+48V", channel)
}

pub fn get_stereo_link_pairs(model: MixerModel) -> Vec<(u8, u8, String)> {
    let max_ch = max_channels_for_model(model);
    let mut pairs = Vec::new();
    for ch1 in (1..max_ch).step_by(2) {
        let ch2 = ch1 + 1;
        let path = format!("/config/chlink/{}-{}", ch1, ch2);
        pairs.push((ch1, ch2, path));
    }
    pairs
}

pub fn calculate_gain_adjustment(
    current_db: f32,
    target_dbfs: f32,
    tolerance_db: f32,
    max_step_db: f32,
) -> f32 {
    if current_db > -3.0 {
        -6.0
    } else if (current_db - target_dbfs).abs() > tolerance_db {
        let step = target_dbfs - current_db;
        step.clamp(-max_step_db, max_step_db)
    } else {
        0.0
    }
}

pub fn parse_channel_from_gain_path(path: &str) -> Option<u8> {
    if path.starts_with("/headamp/") && path.ends_with("/gain") {
        if let Some(ch_str) = path.split('/').nth(2) {
            return ch_str.parse::<u8>().ok();
        }
    }
    None
}

pub async fn run(args: Args) -> Result<()> {
    // Parse model-aware channels
    let channels = parse_channels_for_model(&args.channels, args.model);
    let max_ch = max_channels_for_model(args.model);

    if channels.is_empty() {
        println!(
            "No valid channels provided for model {} (max channel: {}). Expected format: --channels 1,2,3",
            args.model, max_ch
        );
        return Ok(());
    }

    println!("Connecting to {} ({}) ...", args.ip, args.model);
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
        let gain_path = get_headamp_gain_path(args.model, *ch);
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

    // Query model-aware stereo link configuration
    for (ch1, ch2, link_path) in get_stereo_link_pairs(args.model) {
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
                if let Some(ch) = parse_channel_from_gain_path(&msg.path) {
                    if let Some(OscArg::Float(f)) = msg.args.first() {
                        ha_gains.insert(ch, *f);
                    }
                }

                // Track phantom power changes to warn user
                if msg.path.starts_with("/headamp/") && msg.path.ends_with("/+48V") {
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
                        let required_len = 4 + (max_ch as usize) * 4;
                        if data.len() < required_len {
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

                                let delta_db = calculate_gain_adjustment(
                                    current_db,
                                    target_dbfs,
                                    args.tolerance_db,
                                    args.max_step_db,
                                );

                                if current_db > -3.0 {
                                    println!("CLIP PROTECTION: Ch {:02} peak {:.1} dBFS (> -3.0 dBFS). Dropping gain by -6.0 dB!", ch, current_db);
                                }

                                if delta_db.abs() > 0.01 {
                                    let delta_osc = delta_db / 72.0;

                                    if let Some(&current_osc) = ha_gains.get(ch) {
                                        let new_osc = (current_osc + delta_osc).clamp(0.0, 1.0);

                                        if (new_osc - current_osc).abs() > 0.005 {
                                            println!("Ch {:02} level {:.1}dBFS. Adjusting gain by {:.1}dB", ch, current_db, delta_db);
                                            let path = get_headamp_gain_path(args.model, *ch);
                                            let _ = client.send_message(&path, vec![OscArg::Float(new_osc)]).await;
                                            ha_gains.insert(*ch, new_osc);

                                            // Stereo Link Sync: Apply identical gain to paired channel if linked
                                            if let Some(&linked_ch) = channel_links.get(ch) {
                                                let linked_path = get_headamp_gain_path(args.model, linked_ch);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_channels_for_model() {
        assert_eq!(max_channels_for_model(MixerModel::X32), 32);
        assert_eq!(max_channels_for_model(MixerModel::Wing), 40);
        assert_eq!(max_channels_for_model(MixerModel::XR18), 16);
        assert_eq!(max_channels_for_model(MixerModel::XR16), 16);
        assert_eq!(max_channels_for_model(MixerModel::XR12), 12);
    }

    #[test]
    fn test_parse_channels_for_model() {
        assert_eq!(
            parse_channels_for_model("1, 12, 13, 16, 32, 40", MixerModel::XR12),
            vec![1, 12]
        );
        assert_eq!(
            parse_channels_for_model("1, 12, 13, 16, 32, 40", MixerModel::XR18),
            vec![1, 12, 13, 16]
        );
        assert_eq!(
            parse_channels_for_model("1, 12, 13, 16, 32, 40", MixerModel::X32),
            vec![1, 12, 13, 16, 32]
        );
        assert_eq!(
            parse_channels_for_model("1, 12, 13, 16, 32, 40", MixerModel::Wing),
            vec![1, 12, 13, 16, 32, 40]
        );
    }

    #[test]
    fn test_get_headamp_paths() {
        assert_eq!(get_headamp_gain_path(MixerModel::X32, 1), "/headamp/01/gain");
        assert_eq!(get_headamp_gain_path(MixerModel::Wing, 40), "/headamp/40/gain");
        assert_eq!(
            get_headamp_phantom_path(MixerModel::X32, 5),
            "/headamp/05/+48V"
        );
    }

    #[test]
    fn test_stereo_link_pairs() {
        let x32_pairs = get_stereo_link_pairs(MixerModel::X32);
        assert_eq!(x32_pairs.len(), 16);
        assert_eq!(x32_pairs[0], (1, 2, "/config/chlink/1-2".to_string()));
        assert_eq!(x32_pairs[15], (31, 32, "/config/chlink/31-32".to_string()));

        let wing_pairs = get_stereo_link_pairs(MixerModel::Wing);
        assert_eq!(wing_pairs.len(), 20);
        assert_eq!(wing_pairs[19], (39, 40, "/config/chlink/39-40".to_string()));

        let xr12_pairs = get_stereo_link_pairs(MixerModel::XR12);
        assert_eq!(xr12_pairs.len(), 6);
        assert_eq!(xr12_pairs[5], (11, 12, "/config/chlink/11-12".to_string()));
    }

    #[test]
    fn test_calculate_gain_adjustment() {
        // Clip protection: peak > -3.0 dBFS drops gain by -6.0 dB
        assert_eq!(calculate_gain_adjustment(-2.0, -18.0, 2.0, 1.0), -6.0);

        // Within tolerance deadzone: -18 dBFS target, current -17 dBFS, tol 2.0 -> 0.0
        assert_eq!(calculate_gain_adjustment(-17.0, -18.0, 2.0, 1.0), 0.0);

        // Step calculation clamped to max_step_db: current -30 dBFS, target -18 dBFS (step +12), max_step 1.0 -> 1.0
        assert_eq!(calculate_gain_adjustment(-30.0, -18.0, 2.0, 1.0), 1.0);

        // Negative step clamped to max_step_db: current -10 dBFS, target -18 dBFS (step -8), max_step 1.0 -> -1.0
        assert_eq!(calculate_gain_adjustment(-10.0, -18.0, 2.0, 1.0), -1.0);
    }

    #[test]
    fn test_parse_channel_from_gain_path() {
        assert_eq!(parse_channel_from_gain_path("/headamp/01/gain"), Some(1));
        assert_eq!(parse_channel_from_gain_path("/headamp/40/gain"), Some(40));
        assert_eq!(parse_channel_from_gain_path("/ch/01/mix/fader"), None);
        assert_eq!(parse_channel_from_gain_path("/headamp/01/+48V"), None);
    }
}
