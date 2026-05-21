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
use std::time::{Duration, Instant};
use x32_lib::{MixerClient, error::Result};

/// A utility to provide automixing functionality for the Behringer X32/X-Air consoles.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// IP address of the X32 console
    #[arg(short, long)]
    ip: String,

    /// Delay to Fader Down in seconds
    #[arg(long, default_value_t = 5)]
    down_delay: u64,

    /// Meter update rate from the console in milliseconds
    #[arg(long, default_value_t = 50)]
    meter_rate_ms: u64,

    /// Sensitivity threshold for channel activation
    #[arg(short, long, default_value_t = 0.005)]
    sensitivity: f32,

    /// Start channel for automixing (1-32)
    #[arg(long, default_value_t = 1)]
    start_channel: u8,

    /// Stop channel for automixing (1-32)
    #[arg(long, default_value_t = 32)]
    stop_channel: u8,

    /// Use a specific bus for the mix instead of the main L/R mix
    #[arg(short, long)]
    use_bus: bool,

    /// Bus number to use if --use-bus is specified
    #[arg(long, default_value_t = 1)]
    bus_number: u8,

    /// Enable Number Of Mics (NOM) feature
    #[arg(long)]
    nom: bool,
}

/// The main entry point for the automixer application.
#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
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
    let mut channel_status: Vec<(bool, Instant)> = vec![(false, Instant::now()); 32];
    let mut active_channels = 0;
    let mut nom_level = 1;

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
    let mix_address = if args.use_bus {
        format!("/bus/{:02}/mix/fader", args.bus_number)
    } else {
        "/main/st/mix/fader".to_string()
    };

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
                        let mut changed = false;
                        let start_ch = args.start_channel.saturating_sub(1) as usize;
                        let stop_ch = args.stop_channel as usize;

                        for ch in start_ch..stop_ch {
                            let start = ch * 4;
                            let end = start + 4;
                            if let Some(bytes) =
                                data.get(start..end).and_then(|s| s.try_into().ok())
                            {
                                let level = f32::from_be_bytes(bytes);
                                if let Some((is_active, last_active_time)) =
                                    channel_status.get_mut(ch)
                                {
                                    if level > args.sensitivity {
                                        *last_active_time = Instant::now();
                                        if !*is_active {
                                            *is_active = true;
                                            active_channels += 1;
                                            changed = true;
                                            if let Some(addr) = fader_addresses.get(ch) {
                                                client.send_message(
                                                    &addr.1,
                                                    vec![OscArg::Float(1.0)],
                                                ).await?;
                                            }
                                        }
                                    } else if *is_active
                                        && last_active_time.elapsed()
                                            > Duration::from_secs(args.down_delay)
                                    {
                                        *is_active = false;
                                        active_channels -= 1;
                                        changed = true;
                                        if let Some(addr) = fader_addresses.get(ch) {
                                            client.send_message(
                                                &addr.0,
                                                vec![OscArg::Float(0.0)],
                                            ).await?;
                                        }
                                    }
                                }
                            }
                        }
                        if args.nom && changed {
                            update_nom_gain(
                                &client,
                                &mix_address,
                                active_channels,
                                &mut nom_level,
                            ).await?;
                        }
                    }
                }
            }
        }
    }
}

/// Updates the master gain based on the Number of Open Mics (NOM).
///
/// # Arguments
///
/// * `socket` - The UDP socket connected to the mixer.
/// * `mix_address` - The OSC address of the master fader.
/// * `active_channels` - The count of currently active channels.
/// * `nom_level` - A mutable reference to the current NOM attenuation level.
async fn update_nom_gain(
    client: &MixerClient,
    mix_address: &str,
    active_channels: i32,
    nom_level: &mut i32,
) -> Result<()> {
    if active_channels >= *nom_level * 2 {
        *nom_level *= 2;
        adjust_gain(client, mix_address, -3.0).await?;
    } else if active_channels <= *nom_level / 2 {
        *nom_level /= 2;
        if *nom_level < 1 {
            *nom_level = 1;
        }
        adjust_gain(client, mix_address, 3.0).await?;
    }
    Ok(())
}

/// Adjusts the gain of a fader by a relative decibel amount.
///
/// # Arguments
///
/// * `socket` - The UDP socket connected to the mixer.
/// * `address` - The OSC address of the fader to adjust.
/// * `db_change` - The amount to change the gain by, in decibels.
async fn adjust_gain(client: &MixerClient, address: &str, db_change: f32) -> Result<()> {
    let response = client.query_value(address).await?;

    if let OscArg::Float(current_level) = response {
        let db = level_to_db(current_level);
        let new_db = (db + db_change).clamp(-90.0, 10.0);
        let new_level = db_to_level(new_db);
        client
            .send_message(address, vec![OscArg::Float(new_level)])
            .await?;
    }
    Ok(())
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
        let status = vec![(false, Instant::now()); 32];
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
}
