use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand, ValueEnum};
use osc_lib::OscArg;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use x32_lib::MixerClient;

#[derive(Parser, Debug)]
#[command(name = "x32_loudness")]
#[command(about = "Equal-Loudness (Fletcher-Munson) dynamic EQ automation for X32/M32", long_about = None)]
struct Cli {
    #[arg(short, long, default_value = "192.168.1.50")]
    ip: String,

    #[arg(long, default_value = ".x32_loudness.json")]
    config: String,

    #[arg(long)]
    room_eq: Option<String>,

    #[arg(long, value_enum, default_value_t = LoudnessMode::Peq)]
    mode: LoudnessMode,

    #[arg(long, default_value_t = 5)]
    fx_slot: usize,

    #[arg(long, default_value = "/main/st/mix/fader")]
    fader_path: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    /// Run calibration using the built-in pink noise generator
    Calibrate {
        /// Calibration target sound level in dBA SPL
        #[arg(long, default_value_t = 60.0)]
        target_db: f32,
    },
    /// Run the dynamic EQ automation daemon
    Run,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum LoudnessMode {
    Peq,
    Geq,
    Deq,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CalibrationConfig {
    c_room: f32,
    mode: LoudnessMode,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct RoomEqConfig {
    peq: Option<Vec<f32>>,
    geq: Option<Vec<f32>>,
    deq: Option<Vec<f32>>,
}

// Convert normalized fader [0..1] to decibels
fn fader_to_db(f: f32) -> f32 {
    let f = f.clamp(0.0, 1.0);
    if f >= 0.5 {
        40.0 * f - 30.0
    } else if f >= 0.25 {
        80.0 * f - 50.0
    } else if f >= 0.0625 {
        160.0 * f - 70.0
    } else if f > 0.0 {
        480.0 * f - 90.0
    } else {
        -120.0
    }
}

// Convert DB to EQ gain [0..1] mapping (representing [-15..15] dB)
fn db_to_eq_gain(db: f32) -> f32 {
    ((db + 15.0) / 30.0).clamp(0.0, 1.0)
}

// Center frequencies for 31-band TruEQ/GEQ
const GEQ_FREQ: [f32; 31] = [
    20.0, 25.0, 31.5, 40.0, 50.0, 63.0, 80.0, 100.0, 125.0, 160.0, 200.0, 250.0, 315.0, 400.0,
    500.0, 630.0, 800.0, 1000.0, 1250.0, 1600.0, 2000.0, 2500.0, 3150.0, 4000.0, 5000.0, 6300.0,
    8000.0, 10000.0, 12500.0, 16000.0, 20000.0,
];

// ISO 226 frequency sensitivity slope factor for graphic EQ
fn geq_slope_factor(freq: f32) -> f32 {
    if freq < 100.0 {
        0.22
    } else if freq < 1000.0 {
        0.22 - ((freq - 100.0) / 900.0) * 0.22
    } else if freq < 4000.0 {
        -0.08
    } else {
        0.12
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let full_ip = if cli.ip.contains(':') {
        cli.ip.clone()
    } else {
        format!("{}:10023", cli.ip)
    };

    println!("Connecting to X32 Mixer at {}...", full_ip);
    let client = Arc::new(MixerClient::connect(&full_ip, true).await?);
    println!("Connected successfully.");

    match cli.command {
        Commands::Calibrate { target_db } => {
            run_calibration(client, &cli.config, target_db).await?;
        }
        Commands::Run => {
            run_daemon(
                client,
                &cli.config,
                cli.room_eq.as_deref(),
                cli.mode,
                cli.fx_slot,
                &cli.fader_path,
            )
            .await?;
        }
    }

    Ok(())
}

async fn run_calibration(
    client: Arc<MixerClient>,
    config_path: &str,
    target_db: f32,
) -> Result<()> {
    println!("\n=== Starting Loudness Calibration ===");
    println!("This will activate the X32 built-in pink noise generator.");
    println!("Please ensure Main volume is down or comfortable before starting.");
    print!("Press ENTER to start tone generator...");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    // Configure oscillator: Pink Noise, target L/R, level -18 dBFS, Active = 1
    println!("Activating oscillator...");
    client
        .send_message("/config/osc/type", vec![OscArg::Int(1)])
        .await?; // Pink Noise
    client
        .send_message("/config/osc/dest", vec![OscArg::Int(16)])
        .await?; // Main L/R
    client
        .send_message("/config/osc/level", vec![OscArg::Float(0.166)])
        .await?; // approx -18 dBFS
    client
        .send_message("/config/osc/active", vec![OscArg::Int(1)])
        .await?;

    println!("\n---> Adjust main fader on the console until room volume");
    println!(
        "     matches a comfortable conversation level (~{} dBA SPL).",
        target_db
    );
    print!("Press ENTER when done...");
    io::stdout().flush()?;
    input.clear();
    io::stdin().read_line(&mut input)?;

    // Query main fader value
    println!("Querying main fader level...");
    let fader_arg = client.query_value("/main/st/mix/fader").await?;
    let fader_val = match fader_arg {
        OscArg::Float(f) => f,
        _ => return Err(anyhow!("Unexpected fader response type: {:?}", fader_arg)),
    };

    // Deactivate oscillator
    println!("Deactivating oscillator...");
    let _ = client
        .send_message("/config/osc/active", vec![OscArg::Int(0)])
        .await;

    let fader_db = fader_to_db(fader_val);
    let c_room = target_db - fader_db;

    println!("\nCalibration Results:");
    println!("  Fader Position: {:.3} ({:.1} dB)", fader_val, fader_db);
    println!("  Calculated Room Constant C_room: {:.1} dB", c_room);

    let config = CalibrationConfig {
        c_room,
        mode: LoudnessMode::Peq, // Default mode, can be overridden during run
    };

    let file = File::create(config_path)?;
    serde_json::to_writer_pretty(file, &config)?;
    println!("Saved calibration config to: {}", config_path);

    Ok(())
}

async fn run_daemon(
    client: Arc<MixerClient>,
    config_path: &str,
    room_eq_path: Option<&str>,
    mode: LoudnessMode,
    fx_slot: usize,
    fader_path: &str,
) -> Result<()> {
    if !Path::new(config_path).exists() {
        return Err(anyhow!(
            "Calibration config not found at '{}'. Please run calibration first.",
            config_path
        ));
    }

    let file = File::open(config_path)?;
    let cal: CalibrationConfig = serde_json::from_reader(file)?;
    println!("Loaded calibration offset C_room: {:.1} dB", cal.c_room);

    let room_eq = if let Some(path) = room_eq_path {
        if Path::new(path).exists() {
            let file = File::open(path)?;
            let r: RoomEqConfig = serde_json::from_reader(file)?;
            println!("Loaded room EQ correction file: {}", path);
            r
        } else {
            println!(
                "Warning: Room EQ file '{}' specified but not found. Using zero offsets.",
                path
            );
            RoomEqConfig::default()
        }
    } else {
        RoomEqConfig::default()
    };

    println!(
        "Running Fletcher-Munson dynamic EQ daemon in {:?} mode (polling {})...",
        mode, fader_path
    );

    let mut last_fader_val: Option<f32> = None;
    let mut rx = client.subscribe();

    // Query initial fader value to bootstrap the loop
    if let Ok(OscArg::Float(f)) = client.query_value(fader_path).await {
        last_fader_val = Some(f);
        if let Err(e) = update_eq(&client, f, cal.c_room, mode, fx_slot, &room_eq).await {
            eprintln!("Error applying initial EQ: {}", e);
        }
    }

    loop {
        tokio::select! {
            msg_res = rx.recv() => {
                match msg_res {
                    Ok(msg) => {
                        if msg.path == fader_path {
                            if let Some(OscArg::Float(f)) = msg.args.first() {
                                if Some(*f) != last_fader_val {
                                    last_fader_val = Some(*f);
                                    if let Err(e) = update_eq(&client, *f, cal.c_room, mode, fx_slot, &room_eq).await {
                                        eprintln!("Error updating EQ: {}", e);
                                    }
                                }
                            }
                        }
                    }
                    _ => break,
                }
            }
            _ = sleep(Duration::from_millis(100)) => {
                // Periodically query fader in case client missed updates or mixer wasn't sending /xremote updates
                if let Ok(OscArg::Float(f)) = client.query_value(fader_path).await {
                    if Some(f) != last_fader_val {
                        last_fader_val = Some(f);
                        if let Err(e) = update_eq(&client, f, cal.c_room, mode, fx_slot, &room_eq).await {
                            eprintln!("Error updating EQ: {}", e);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

async fn update_eq(
    client: &MixerClient,
    fader_val: f32,
    c_room: f32,
    mode: LoudnessMode,
    fx_slot: usize,
    room_eq: &RoomEqConfig,
) -> Result<()> {
    let fader_db = fader_to_db(fader_val);
    let spl_est = (fader_db + c_room).clamp(40.0, 100.0);

    let spl_diff = 85.0 - spl_est;

    match mode {
        LoudnessMode::Peq => {
            // Band 1: Low Shelf (80Hz). Band 6: High Shelf (10kHz).
            let dynamic_low = if spl_diff > 0.0 {
                (0.22 * spl_diff).clamp(0.0, 9.0)
            } else {
                0.0
            };
            let dynamic_high = if spl_diff > 0.0 {
                (0.12 * spl_diff).clamp(0.0, 5.0)
            } else {
                0.0
            };

            let offset_low = room_eq
                .peq
                .as_ref()
                .and_then(|v| v.first())
                .cloned()
                .unwrap_or(0.0);
            let offset_high = room_eq
                .peq
                .as_ref()
                .and_then(|v| v.get(5))
                .cloned()
                .unwrap_or(0.0);

            let final_low = (dynamic_low + offset_low).clamp(-15.0, 15.0);
            let final_high = (dynamic_high + offset_high).clamp(-15.0, 15.0);

            // Configure PEQ types (Low Shelf / High Shelf) to ensure correctness
            client
                .send_message("/main/st/eq/1/type", vec![OscArg::Int(1)])
                .await?; // Low Shelf
            client
                .send_message("/main/st/eq/6/type", vec![OscArg::Int(2)])
                .await?; // High Shelf

            // Send gains
            client
                .send_message(
                    "/main/st/eq/1/g",
                    vec![OscArg::Float(db_to_eq_gain(final_low))],
                )
                .await?;
            client
                .send_message(
                    "/main/st/eq/6/g",
                    vec![OscArg::Float(db_to_eq_gain(final_high))],
                )
                .await?;

            println!(
                "SPL: {:.1} dBA | PEQ Low: {:.1} dB, PEQ High: {:.1} dB",
                spl_est, final_low, final_high
            );
        }
        LoudnessMode::Geq => {
            // Update 31 Graphic EQ bands in the selected slot
            for (i, &freq) in GEQ_FREQ.iter().enumerate() {
                let k = geq_slope_factor(freq);
                let dynamic_gain = if spl_diff > 0.0 {
                    if k > 0.0 {
                        (k * spl_diff).clamp(0.0, 12.0)
                    } else {
                        (k * spl_diff).clamp(-4.0, 0.0)
                    }
                } else {
                    0.0
                };

                let offset = room_eq
                    .geq
                    .as_ref()
                    .and_then(|v| v.get(i))
                    .cloned()
                    .unwrap_or(0.0);
                let final_gain = (dynamic_gain + offset).clamp(-15.0, 15.0);

                let path = format!("/fx/{}/par/{:02}", fx_slot, i);
                client
                    .send_message(&path, vec![OscArg::Float(db_to_eq_gain(final_gain))])
                    .await?;
            }
            println!(
                "SPL: {:.1} dBA | GEQ Slot {} updated (31 bands)",
                spl_est, fx_slot
            );
        }
        LoudnessMode::Deq => {
            // Dynamic EQ: 4 bands.
            // Band 1 (Low Shelf): Gain param 0
            // Band 2 (Bell 500Hz): Gain param 10
            // Band 3 (Bell 3.2kHz): Gain param 20
            // Band 4 (High Shelf): Gain param 30
            let dyn_b1 = if spl_diff > 0.0 {
                (0.22 * spl_diff).clamp(0.0, 9.0)
            } else {
                0.0
            };
            let dyn_b2 = if spl_diff > 0.0 {
                (-0.05 * spl_diff).clamp(-2.0, 0.0)
            } else {
                0.0
            };
            let dyn_b3 = if spl_diff > 0.0 {
                (-0.10 * spl_diff).clamp(-4.0, 0.0)
            } else {
                0.0
            };
            let dyn_b4 = if spl_diff > 0.0 {
                (0.12 * spl_diff).clamp(0.0, 5.0)
            } else {
                0.0
            };

            let off_b1 = room_eq
                .deq
                .as_ref()
                .and_then(|v| v.first())
                .cloned()
                .unwrap_or(0.0);
            let off_b2 = room_eq
                .deq
                .as_ref()
                .and_then(|v| v.get(1))
                .cloned()
                .unwrap_or(0.0);
            let off_b3 = room_eq
                .deq
                .as_ref()
                .and_then(|v| v.get(2))
                .cloned()
                .unwrap_or(0.0);
            let off_b4 = room_eq
                .deq
                .as_ref()
                .and_then(|v| v.get(3))
                .cloned()
                .unwrap_or(0.0);

            let final_b1 = (dyn_b1 + off_b1).clamp(-15.0, 15.0);
            let final_b2 = (dyn_b2 + off_b2).clamp(-15.0, 15.0);
            let final_b3 = (dyn_b3 + off_b3).clamp(-15.0, 15.0);
            let final_b4 = (dyn_b4 + off_b4).clamp(-15.0, 15.0);

            client
                .send_message(
                    &format!("/fx/{}/par/00", fx_slot),
                    vec![OscArg::Float(db_to_eq_gain(final_b1))],
                )
                .await?;
            client
                .send_message(
                    &format!("/fx/{}/par/10", fx_slot),
                    vec![OscArg::Float(db_to_eq_gain(final_b2))],
                )
                .await?;
            client
                .send_message(
                    &format!("/fx/{}/par/20", fx_slot),
                    vec![OscArg::Float(db_to_eq_gain(final_b3))],
                )
                .await?;
            client
                .send_message(
                    &format!("/fx/{}/par/30", fx_slot),
                    vec![OscArg::Float(db_to_eq_gain(final_b4))],
                )
                .await?;

            println!(
                "SPL: {:.1} dBA | DEQ Slot {} (B1: {:.1}dB, B2: {:.1}dB, B3: {:.1}dB, B4: {:.1}dB)",
                spl_est, fx_slot, final_b1, final_b2, final_b3, final_b4
            );
        }
    }

    Ok(())
}
