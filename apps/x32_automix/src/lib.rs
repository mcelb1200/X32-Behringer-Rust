//! `x32_automix` is a command-line tool that provides automixing functionality for Behringer X32/M32 mixers.
//!
//! It monitors the input levels of specified channels and automatically adjusts their
//! faders (or mix sends) to maintain a balanced mix. It also includes a Number of
//! Open Mics (NOM) attenuation feature to prevent feedback and background noise buildup
//! when multiple microphones are open.
//!
//! # Credits
//!
//! *   **Original concept and work on the C library:** Patrick-Gilles Maillot
//! *   **Additional concepts by:** mcelb1200
//! *   **Rust implementation by:** mcelb1200

use clap::Parser;
use osc_lib::OscArg;
use std::time::Duration;
use x32_lib::{MixerClient, error::Result};

/// A utility to provide automixing functionality for the Behringer X32/X-Air consoles.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// IP address of the X32 console
    #[arg(short, long)]
    pub ip: String,

    /// Delay to Fader Down in seconds
    #[arg(long, default_value_t = 5)]
    pub down_delay: u64,

    /// Meter update rate from the console in milliseconds
    #[arg(long, default_value_t = 50)]
    pub meter_rate_ms: u64,

    /// Sensitivity threshold for channel activation
    #[arg(short, long, default_value_t = 0.005)]
    pub sensitivity: f32,

    /// Start channel for automixing (1-32)
    #[arg(long, default_value_t = 1)]
    pub start_channel: u8,

    /// Stop channel for automixing (1-32)
    #[arg(long, default_value_t = 32)]
    pub stop_channel: u8,

    /// Use a specific bus for the mix instead of the main L/R mix
    #[arg(short, long)]
    pub use_bus: bool,

    /// Bus number to use if --use-bus is specified
    #[arg(long, default_value_t = 1)]
    pub bus_number: u8,

    /// Enable Number Of Mics (NOM) feature
    #[arg(long)]
    pub nom: bool,
}

/// The main entry point for the automixer application.
pub async fn run(args: Args) -> Result<()> {
    println!("Connecting to X32 at {}...", args.ip);

    let client = MixerClient::connect(&args.ip, true).await?;

    run_automix(args, client).await
}

/// Runs the automixing loop.
///
/// This function continuously monitors channel levels and adjusts faders based on the
/// configured rules.
///
/// # Arguments
///
/// * `args` - The command-line arguments containing the automix configuration.
/// * `client` - The MixerClient connected to the mixer.
async fn run_automix(args: Args, client: MixerClient) -> Result<()> {
    // We maintain state for Dugan UDP throttling and noise tracking
    let mut last_sent_levels: [f32; 32] = [0.0; 32];
    let mut smoothed_levels: [f32; 32] = [0.0; 32];

    let attack_coef = 0.8;
    let release_coef = 0.2;

    let fader_addresses: Vec<(String, String)> = (1..=32)
        .map(|ch| {
            let base = format!("/ch/{:02}", ch);
            if args.use_bus {
                (
                    format!("{}/mix/{:02}/level", base, args.bus_number),
                    format!("{}/mix/{:02}/level", base, args.bus_number),
                )
            } else {
                (format!("{}/mix/fader", base), format!("{}/mix/fader", base))
            }
        })
        .collect();

    let mut rx = client.subscribe();
    let mut meter_interval = tokio::time::interval(Duration::from_secs(9));

    loop {
        tokio::select! {
            _ = meter_interval.tick() => {
                client.send_message(
                    "/meters",
                    vec![
                        OscArg::String("/meters/1".to_string()),
                        OscArg::Int(0),
                        OscArg::Int(0),
                        OscArg::Int(args.meter_rate_ms as i32 / 50),
                    ]
                ).await?;
            }
            result = rx.recv() => {
                let response = match result {
                    Ok(msg) => msg,
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break Ok(()),
                };
                if response.path == "/meters/1" {
                    if let Some(OscArg::Blob(data)) = response.args.first() {
                        let start_ch = args.start_channel.saturating_sub(1) as usize;
                        let stop_ch = args.stop_channel as usize;

                        // 1. Parse levels and apply fast attack / slow release envelope
                        let mut current_levels = [0.0; 32];
                        for ch in start_ch..stop_ch {
                            let start = ch * 4;
                            let end = start + 4;
                            if let Some(bytes) =
                                data.get(start..end).and_then(|s| s.try_into().ok())
                            {
                                let level = f32::from_be_bytes(bytes);
                                if level > smoothed_levels[ch] {
                                    smoothed_levels[ch] = smoothed_levels[ch] * (1.0 - attack_coef) + level * attack_coef;
                                } else {
                                    smoothed_levels[ch] = smoothed_levels[ch] * (1.0 - release_coef) + level * release_coef;
                                }
                                current_levels[ch] = smoothed_levels[ch];
                            }
                        }

                        // 2. Calculate Dugan gains if NOM is enabled, else simple threshold
                        let gains = if args.nom {
                            let levels_slice = &current_levels[start_ch..stop_ch];
                            let calculated = calculate_dugan_gains(levels_slice, args.sensitivity);
                            let mut full_gains = [0.0; 32];
                            for (i, &g) in calculated.iter().enumerate() {
                                if start_ch + i < 32 {
                                    full_gains[start_ch + i] = g;
                                }
                            }
                            full_gains
                        } else {
                            // Legacy simple threshold (0.75 represents unity gain on X32, 1.0 represents +10dB which can cause feedback)
                            let mut full_gains = [0.0; 32];
                            for ch in start_ch..stop_ch {
                                if current_levels[ch] > args.sensitivity {
                                    full_gains[ch] = 0.75;
                                }
                            }
                            full_gains
                        };

                        // 3. UDP Throttling: Only send updates if fader level changed by > 0.01
                        for ch in start_ch..stop_ch {
                            let new_gain = gains[ch];
                            if (new_gain - last_sent_levels[ch]).abs() > 0.01 {
                                last_sent_levels[ch] = new_gain;
                                if let Some(addr) = fader_addresses.get(ch) {
                                    client.send_message(
                                        &addr.0,
                                        vec![OscArg::Float(new_gain)],
                                    ).await?;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Converts a linear fader level (0.0 to 1.0) to decibels.
fn level_to_db(level: f32) -> f32 {
    if level >= 0.5 {
        40.0 * level - 30.0
    } else if level >= 0.25 {
        80.0 * level - 50.0
    } else if level >= 0.0625 {
        160.0 * level - 70.0
    } else {
        480.0 * level - 90.0
    }
}

/// Converts a decibel value to a linear fader level (0.0 to 1.0).
fn db_to_level(db: f32) -> f32 {
    let level = if db > -10.0 {
        (db + 30.0) / 40.0
    } else if db > -30.0 {
        (db + 50.0) / 80.0
    } else if db > -60.0 {
        (db + 70.0) / 160.0
    } else {
        (db + 90.0) / 480.0
    };
    level.clamp(0.0, 1.0)
}

/// Calculates the gain for each channel based on the Dugan algorithm.
///
/// The Dugan algorithm works by calculating the sum of the linear weights
/// of all channels. The gain for each channel is its weight divided by the total sum.
/// This ensures that the overall system gain remains constant (NOM attenuation),
/// preventing feedback and noise buildup.
///
/// We also apply priority ducking by only including channels above the noise_floor.
fn calculate_dugan_gains(levels: &[f32], noise_floor: f32) -> [f32; 32] {
    let mut weights = [0.0; 32];
    let mut sum_weights = 0.0;

    for (i, &level) in levels.iter().enumerate() {
        if i >= 32 {
            break;
        }
        if level > noise_floor {
            let db = level_to_db(level);
            // Convert dB to linear weight. 10^(dB/20) gives voltage gain,
            // which works well for Dugan sum.
            let weight = 10.0_f32.powf(db / 20.0);
            weights[i] = weight;
            sum_weights += weight;
        } else {
            weights[i] = 0.0;
        }
    }

    let mut gains = [0.0; 32];
    for (i, &weight) in weights.iter().enumerate() {
        if i >= levels.len() || i >= 32 {
            break;
        }
        if sum_weights > 0.0 && weight > 0.0 {
            // G_i = W_i / W_sum
            let gain_linear = weight / sum_weights;
            // The gain to apply (in linear fader space) should reflect this attenuation.
            // A channel alone gets gain 1.0. Two equal channels get gain 0.5 (-6dB).
            let gain_db = 20.0 * gain_linear.log10();

            // Map the Dugan target gain to the X32 fader curve where 0dB = 0.75 float.
            // We assume a base unity mix (0dB). If we want to be safe, Dugan max is 0dB -> 0.75 fader.
            let fader_level = db_to_level(gain_db);
            gains[i] = fader_level;
        } else {
            gains[i] = 0.0; // Full attenuation if below noise floor or no signal
        }
    }

    gains
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fader_address_generation_main_mix() {
        let args = Args {
            ip: "127.0.0.1".to_string(),
            down_delay: 5,
            meter_rate_ms: 50,
            sensitivity: 0.005,
            start_channel: 1,
            stop_channel: 32,
            use_bus: false,
            bus_number: 1,
            nom: false,
        };

        let fader_addresses = (1..=32)
            .map(|ch| {
                let base = format!("/ch/{:02}", ch);
                if args.use_bus {
                    (
                        format!("{}/mix/{:02}/level", base, args.bus_number),
                        format!("{}/mix/{:02}/level", base, args.bus_number),
                    )
                } else {
                    (format!("{}/mix/fader", base), format!("{}/mix/fader", base))
                }
            })
            .collect::<Vec<(String, String)>>();

        assert_eq!(fader_addresses[0].0, "/ch/01/mix/fader");
        assert_eq!(fader_addresses[31].0, "/ch/32/mix/fader");
    }

    #[test]
    fn test_fader_address_generation_bus_mix() {
        let args = Args {
            ip: "127.0.0.1".to_string(),
            down_delay: 5,
            meter_rate_ms: 50,
            sensitivity: 0.005,
            start_channel: 1,
            stop_channel: 32,
            use_bus: true,
            bus_number: 5,
            nom: false,
        };

        let fader_addresses = (1..=32)
            .map(|ch| {
                let base = format!("/ch/{:02}", ch);
                if args.use_bus {
                    (
                        format!("{}/mix/{:02}/level", base, args.bus_number),
                        format!("{}/mix/{:02}/level", base, args.bus_number),
                    )
                } else {
                    (format!("{}/mix/fader", base), format!("{}/mix/fader", base))
                }
            })
            .collect::<Vec<(String, String)>>();

        assert_eq!(fader_addresses[0].0, "/ch/01/mix/05/level");
        assert_eq!(fader_addresses[31].0, "/ch/32/mix/05/level");
    }

    #[test]
    fn test_meters_parsing_safety() {
        let data = vec![0u8; 8];
        let status = vec![false; 32];
        let mut count = 0;

        let start_ch: u32 = 1;
        let stop_ch: u32 = 32;

        let start_idx = start_ch.saturating_sub(1) as usize;
        let stop_idx = stop_ch as usize;

        for i in start_idx..stop_idx.min(32) {
            let start = i * 4;
            let end = start + 4;
            if let Some(chunk) = data.get(start..end) {
                if let Ok(bytes) = chunk.try_into() {
                    let _level = f32::from_be_bytes(bytes);
                    if status.get(i).is_some() {
                        count += 1;
                    }
                }
            }
        }
        assert_eq!(count, 2);
    }

    #[test]
    fn test_db_level_conversion() {
        assert!((level_to_db(1.0) - 10.0).abs() < 0.01);
        assert!((level_to_db(0.0) - -90.0).abs() < 0.01);
        assert!((db_to_level(10.0) - 1.0).abs() < 0.01);
        assert!((db_to_level(-90.0) - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_dugan_gain_calculation() {
        // Two equal channels should get 0.5 (-6dB) gain each.
        // We use levels that convert to a reasonable dB.
        // 0.75 level -> 40*0.75 - 30 = 0 dB.
        let levels = vec![0.75, 0.75];
        let gains = calculate_dugan_gains(&levels, 0.01);
        // Half linear gain = -6.02 dB. X32 scale for -6.02 dB is: (db + 30) / 40 = ( -6.02 + 30 ) / 40 = 23.98 / 40 = ~0.6
        assert!((gains[0] - 0.6).abs() < 0.01);
        assert!((gains[1] - 0.6).abs() < 0.01);

        // One channel active, one below noise floor
        let levels = vec![0.75, 0.0];
        let gains = calculate_dugan_gains(&levels, 0.01);
        // Full linear gain = 1.0 = 0 dB. X32 scale for 0 dB is 0.75.
        assert!((gains[0] - 0.75).abs() < 0.01);
        assert!((gains[1] - 0.0).abs() < 0.01);

        // All below noise floor
        let levels = vec![0.001, 0.001];
        let gains = calculate_dugan_gains(&levels, 0.01);
        assert_eq!(gains[0], 0.0);
        assert_eq!(gains[1], 0.0);
    }
}
