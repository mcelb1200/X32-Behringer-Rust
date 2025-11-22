//! `x32_automix` is a command-line tool that provides automixing functionality for Behringer X32/M32 mixers.
//!
//! It monitors the input levels of specified channels and automatically adjusts their
//! faders (or mix sends) to maintain a balanced mix. It also includes a Number of
//! Open Mics (NOM) attenuation feature to prevent feedback and background noise buildup
//! when multiple microphones are open.

use clap::Parser;
use osc_lib::{OscArg, OscMessage};
use std::io::ErrorKind;
use std::net::UdpSocket;
use std::time::{Duration, Instant};
use x32_lib::{create_socket, error::Result};

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
fn main() -> Result<()> {
    let args = Args::parse();
    println!("Connecting to X32 at {}...", args.ip);

    let socket = create_socket(&args.ip, 1000)?;
    socket.set_nonblocking(true)?;

    run_automix(args, socket)
}

/// Runs the automixing loop.
///
/// This function continuously monitors channel levels and adjusts faders based on the
/// configured rules.
///
/// # Arguments
///
/// * `args` - The command-line arguments containing the automix configuration.
/// * `socket` - The UDP socket connected to the mixer.
fn run_automix(args: Args, socket: UdpSocket) -> Result<()> {
    let mut last_remote_time = Instant::now();
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

    loop {
        if last_remote_time.elapsed() > Duration::from_secs(9) {
            socket.send(&OscMessage::new("/xremote".to_string(), vec![]).to_bytes()?)?;
            socket.send(
                &OscMessage::new(
                    "/meters".to_string(),
                    vec![
                        OscArg::String("/meters/1".to_string()),
                        OscArg::Int(0),
                        OscArg::Int(0),
                        OscArg::Int(args.meter_rate_ms as i32 / 50),
                    ],
                )
                .to_bytes()?,
            )?;
            last_remote_time = Instant::now();
        }

        let mut buf = [0; 4096];
        match socket.recv(&mut buf) {
            Ok(len) => {
                let response = OscMessage::from_bytes(&buf[..len])?;
                if response.path == "/meters/1" {
                    if let Some(OscArg::Blob(data)) = response.args.first() {
                        let mut changed = false;
                        for ch in (args.start_channel - 1)..args.stop_channel {
                            let start = (ch * 4) as usize;
                            let end = start + 4;
                            if data.len() >= end {
                                let level =
                                    f32::from_be_bytes(data[start..end].try_into().unwrap());
                                let (is_active, last_active_time) =
                                    &mut channel_status[ch as usize];

                                if level > args.sensitivity {
                                    *last_active_time = Instant::now();
                                    if !*is_active {
                                        *is_active = true;
                                        active_channels += 1;
                                        changed = true;
                                        socket.send(
                                            &OscMessage::new(
                                                fader_addresses[ch as usize].1.clone(),
                                                vec![OscArg::Float(1.0)],
                                            )
                                            .to_bytes()?,
                                        )?;
                                    }
                                } else if *is_active
                                    && last_active_time.elapsed()
                                        > Duration::from_secs(args.down_delay)
                                {
                                    *is_active = false;
                                    active_channels -= 1;
                                    changed = true;
                                    socket.send(
                                        &OscMessage::new(
                                            fader_addresses[ch as usize].0.clone(),
                                            vec![OscArg::Float(0.0)],
                                        )
                                        .to_bytes()?,
                                    )?;
                                }
                            }
                        }
                        if args.nom && changed {
                            update_nom_gain(
                                &socket,
                                &mix_address,
                                active_channels,
                                &mut nom_level,
                            )?;
                        }
                    }
                }
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {}
            Err(e) => return Err(e.into()),
        }
        std::thread::sleep(Duration::from_millis(10));
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
fn update_nom_gain(
    socket: &UdpSocket,
    mix_address: &str,
    active_channels: i32,
    nom_level: &mut i32,
) -> Result<()> {
    if active_channels >= *nom_level * 2 {
        *nom_level *= 2;
        adjust_gain(socket, mix_address, -3.0)?;
    } else if active_channels <= *nom_level / 2 {
        *nom_level /= 2;
        if *nom_level < 1 {
            *nom_level = 1;
        }
        adjust_gain(socket, mix_address, 3.0)?;
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
fn adjust_gain(socket: &UdpSocket, address: &str, db_change: f32) -> Result<()> {
    socket.send(&OscMessage::new(address.to_string(), vec![]).to_bytes()?)?;
    let mut buf = [0; 512];
    let len = socket.recv(&mut buf)?;
    let response = OscMessage::from_bytes(&buf[..len])?;

    if let Some(OscArg::Float(current_level)) = response.args.first() {
        let db = level_to_db(*current_level);
        let new_db = (db + db_change).clamp(-90.0, 10.0);
        let new_level = db_to_level(new_db);
        socket.send(
            &OscMessage::new(address.to_string(), vec![OscArg::Float(new_level)]).to_bytes()?,
        )?;
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
    fn test_db_level_conversion() {
        assert!((level_to_db(1.0) - 10.0).abs() < 0.01);
        assert!((level_to_db(0.0) - -90.0).abs() < 0.01);
        assert!((db_to_level(10.0) - 1.0).abs() < 0.01);
        assert!((db_to_level(-90.0) - 0.0).abs() < 0.01);
    }
}
