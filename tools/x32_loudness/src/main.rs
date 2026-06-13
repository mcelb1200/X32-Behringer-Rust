use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand, ValueEnum};
use osc_lib::OscArg;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, BufRead, Read, Write};
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

    #[arg(long, default_value = "")]
    aes50_ip: String,

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

    #[arg(long, value_enum, default_value_t = TransportType::Auto)]
    transport: TransportType,

    #[arg(long, default_value = "")]
    usb_port: String,

    #[arg(
        long,
        help = "Use ISO 226:2003 equal-loudness contours for calculation"
    )]
    iso226: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum TransportType {
    Auto,
    Osc,
    Usb,
    Aes50,
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
    /// Print the EQ gains that would be applied for a given SPL or fader position
    ShowCurve {
        /// Estimated SPL at the mixing position (optional, overrides fader/calibration)
        #[arg(long)]
        spl: Option<f32>,

        /// Fader value (0.0 to 1.0) to estimate SPL
        #[arg(long, default_value_t = 0.75)]
        fader: f32,
    },
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

// ISO 226:2003 Equal-Loudness Contours standard data tables
const ISO_226_FREQ: [f32; 29] = [
    20.0, 25.0, 31.5, 40.0, 50.0, 63.0, 80.0, 100.0, 125.0, 160.0, 200.0, 250.0, 315.0, 400.0,
    500.0, 630.0, 800.0, 1000.0, 1250.0, 1600.0, 2000.0, 2500.0, 3150.0, 4000.0, 5000.0, 6300.0,
    8000.0, 10000.0, 12500.0,
];
const ISO_226_ALPHA: [f32; 29] = [
    0.532, 0.506, 0.480, 0.455, 0.432, 0.409, 0.387, 0.367, 0.349, 0.330, 0.315, 0.301, 0.288,
    0.276, 0.267, 0.259, 0.253, 0.250, 0.246, 0.244, 0.243, 0.243, 0.243, 0.242, 0.242, 0.245,
    0.254, 0.271, 0.301,
];
const ISO_226_LU: [f32; 29] = [
    -31.6, -27.2, -23.0, -19.1, -15.9, -13.0, -10.3, -8.1, -6.2, -4.5, -3.1, -2.0, -1.1, -0.4, 0.0,
    0.3, 0.5, 0.0, -2.7, -4.1, -1.0, 1.7, 2.5, 1.2, -2.1, -7.1, -11.2, -10.7, -3.1,
];
const ISO_226_TF: [f32; 29] = [
    78.5, 68.7, 59.5, 51.1, 44.0, 37.5, 31.5, 26.5, 22.1, 17.9, 14.4, 11.4, 8.6, 6.2, 4.4, 3.0,
    2.2, 2.4, 3.5, 1.7, -1.3, -4.2, -6.0, -5.4, -1.5, 6.0, 12.6, 13.9, 12.3,
];

// Linear interpolation helper for ISO 226 frequency parameters
fn interpolate_iso_226(freq: f32) -> (f32, f32, f32) {
    let f = freq.clamp(20.0, 12500.0);
    if f <= 20.0 {
        return (ISO_226_ALPHA[0], ISO_226_LU[0], ISO_226_TF[0]);
    }
    let len = ISO_226_FREQ.len();
    if f >= 12500.0 {
        return (
            ISO_226_ALPHA[len - 1],
            ISO_226_LU[len - 1],
            ISO_226_TF[len - 1],
        );
    }
    for i in 0..len - 1 {
        if f >= ISO_226_FREQ[i] && f <= ISO_226_FREQ[i + 1] {
            let t = (f - ISO_226_FREQ[i]) / (ISO_226_FREQ[i + 1] - ISO_226_FREQ[i]);
            let alpha = ISO_226_ALPHA[i] + t * (ISO_226_ALPHA[i + 1] - ISO_226_ALPHA[i]);
            let lu = ISO_226_LU[i] + t * (ISO_226_LU[i + 1] - ISO_226_LU[i]);
            let tf = ISO_226_TF[i] + t * (ISO_226_TF[i + 1] - ISO_226_TF[i]);
            return (alpha, lu, tf);
        }
    }
    (ISO_226_ALPHA[0], ISO_226_LU[0], ISO_226_TF[0])
}

// Calculate sound pressure level (SPL) in dB for a given frequency and loudness level (phons)
fn iso_226_spl(freq: f32, phon: f32) -> f32 {
    let phon = phon.clamp(0.0, 90.0);
    let (alpha_f, lu_f, tf_f) = interpolate_iso_226(freq);
    let term1 = 10.0f32.powf(0.025 * phon) - 1.15;
    let term2 = (0.4 * 10.0f32.powf((tf_f + lu_f) / 10.0 - 9.0)).powf(alpha_f);
    let a_f = 0.00447 * term1 + term2;
    (10.0 / alpha_f) * a_f.log10() - lu_f + 94.0
}

// Calculate required equal-loudness dynamic EQ gain for ISO 226 algorithm
fn calculate_iso_226_gain(freq: f32, spl_est: f32) -> f32 {
    let spl_ref = 85.0f32;
    let spl_est_clamped = spl_est.clamp(40.0, 90.0);
    let lp_est = iso_226_spl(freq, spl_est_clamped);
    let lp_ref = iso_226_spl(freq, spl_ref);
    lp_est - lp_ref + (spl_ref - spl_est_clamped)
}

// Print calculated EQ gains for a given SPL or fader value
fn show_curve_calculation(
    config_path: &str,
    spl_opt: Option<f32>,
    fader: f32,
    iso226: bool,
) -> Result<()> {
    let spl = if let Some(s) = spl_opt {
        s
    } else {
        let c_room = if Path::new(config_path).exists() {
            let file = File::open(config_path)?;
            let mut file_take = std::io::Read::take(file, 1024 * 1024 + 1);
            let mut cal_buf = Vec::new();
            std::io::Read::read_to_end(&mut file_take, &mut cal_buf)?;
            if cal_buf.len() > 1024 * 1024 {
                return Err(anyhow::anyhow!("File too large"));
            }
            let cal: CalibrationConfig = serde_json::from_slice(&cal_buf)?;
            cal.c_room
        } else {
            println!(
                "No calibration config found at '{}'. Using default C_room: 90.0 dB",
                config_path
            );
            90.0
        };
        let fader_db = fader_to_db(fader);
        fader_db + c_room
    };

    let spl_clamped = spl.clamp(40.0, 100.0);
    let spl_diff = 85.0 - spl_clamped;

    println!("============================================================");
    println!(" Equal-Loudness Compensation Curve Analysis");
    println!("============================================================");
    println!(
        "Estimated SPL: {:.1} dBA  | Reference SPL: 85.0 dBA",
        spl_clamped
    );
    println!("SPL Difference (Reference - Current): {:.1} dB", spl_diff);
    println!(
        "Model in use:  {}",
        if iso226 {
            "ISO 226:2003 Standard"
        } else {
            "Linear Approximation (Legacy)"
        }
    );
    println!("------------------------------------------------------------");

    println!("\n--- PEQ Compensation (Low Shelf 80Hz | High Shelf 10kHz) ---");
    let peq_low = if spl_diff > 0.0 {
        if iso226 {
            calculate_iso_226_gain(80.0, spl_clamped).clamp(0.0, 9.0)
        } else {
            (0.22 * spl_diff).clamp(0.0, 9.0)
        }
    } else {
        0.0
    };
    let peq_high = if spl_diff > 0.0 {
        if iso226 {
            calculate_iso_226_gain(10000.0, spl_clamped).clamp(0.0, 5.0)
        } else {
            (0.12 * spl_diff).clamp(0.0, 5.0)
        }
    } else {
        0.0
    };
    println!("  80 Hz (Low Shelf):  {:+.1} dB", peq_low);
    println!("  10 kHz (High Shelf): {:+.1} dB", peq_high);

    println!("\n--- DEQ Compensation (4-Band Dynamic EQ) ---");
    let deq_b1 = if spl_diff > 0.0 {
        if iso226 {
            calculate_iso_226_gain(80.0, spl_clamped).clamp(0.0, 9.0)
        } else {
            (0.22 * spl_diff).clamp(0.0, 9.0)
        }
    } else {
        0.0
    };
    let deq_b2 = if spl_diff > 0.0 {
        if iso226 {
            calculate_iso_226_gain(500.0, spl_clamped).clamp(-2.0, 0.0)
        } else {
            (-0.05 * spl_diff).clamp(-2.0, 0.0)
        }
    } else {
        0.0
    };
    let deq_b3 = if spl_diff > 0.0 {
        if iso226 {
            calculate_iso_226_gain(3200.0, spl_clamped).clamp(-4.0, 0.0)
        } else {
            (-0.10 * spl_diff).clamp(-4.0, 0.0)
        }
    } else {
        0.0
    };
    let deq_b4 = if spl_diff > 0.0 {
        if iso226 {
            calculate_iso_226_gain(10000.0, spl_clamped).clamp(0.0, 5.0)
        } else {
            (0.12 * spl_diff).clamp(0.0, 5.0)
        }
    } else {
        0.0
    };
    println!("  80 Hz (Low Shelf):  {:+.1} dB", deq_b1);
    println!("  500 Hz (Bell):      {:+.1} dB", deq_b2);
    println!("  3.2 kHz (Bell):     {:+.1} dB", deq_b3);
    println!("  10 kHz (High Shelf): {:+.1} dB", deq_b4);

    println!("\n--- GEQ Compensation (31-Band Graphic EQ Curve) ---");
    println!("  Freq (Hz) | Gain (dB) | Visualization");
    println!("  ----------|-----------|-----------------------------------");
    for &freq in &GEQ_FREQ {
        let gain = if spl_diff > 0.0 {
            if iso226 {
                let raw_gain = calculate_iso_226_gain(freq, spl_clamped);
                if raw_gain >= 0.0 {
                    raw_gain.clamp(0.0, 12.0)
                } else {
                    raw_gain.clamp(-4.0, 0.0)
                }
            } else {
                let k = geq_slope_factor(freq);
                if k > 0.0 {
                    (k * spl_diff).clamp(0.0, 12.0)
                } else {
                    (k * spl_diff).clamp(-4.0, 0.0)
                }
            }
        } else {
            0.0
        };

        let bar_width = 15;
        let bar = if gain >= 0.0 {
            let num_chars = (gain / 12.0 * bar_width as f32).round() as usize;
            format!("|{}", "#".repeat(num_chars))
        } else {
            let num_chars = (gain.abs() / 4.0 * bar_width as f32).round() as usize;
            format!(
                "{}{}|",
                " ".repeat(bar_width - num_chars),
                "#".repeat(num_chars)
            )
        };

        println!("  {:>9.1} | {:>9.2} | {}", freq, gain, bar);
    }
    println!("============================================================");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle offline commands first to avoid attempting to connect to a mixer
    if let Commands::ShowCurve { spl, fader } = &cli.command {
        show_curve_calculation(&cli.config, *spl, *fader, cli.iso226)?;
        return Ok(());
    }

    let transport_str = format!("{:?}", cli.transport).to_lowercase();
    let (client_raw, transport_used) = MixerClient::connect_with_transport(
        &cli.ip,
        &cli.aes50_ip,
        &cli.usb_port,
        &transport_str,
        true,
    )
    .await?;
    let client = Arc::new(client_raw);
    println!("Connected successfully via {}.", transport_used);

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
                &transport_used,
                cli.iso226,
            )
            .await?;
        }
        Commands::ShowCurve { .. } => unreachable!(),
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
    io::stdin().lock().take(1024).read_line(&mut input)?;

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
    io::stdin().lock().take(1024).read_line(&mut input)?;

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

#[allow(clippy::too_many_arguments)]
async fn run_daemon(
    client: Arc<MixerClient>,
    config_path: &str,
    room_eq_path: Option<&str>,
    mode: LoudnessMode,
    fx_slot: usize,
    fader_path: &str,
    transport_used: &str,
    iso226: bool,
) -> Result<()> {
    if !Path::new(config_path).exists() {
        return Err(anyhow!(
            "Calibration config not found at '{}'. Please run calibration first.",
            config_path
        ));
    }

    let file = File::open(config_path)?;
    let mut file_take = std::io::Read::take(file, 1024 * 1024 + 1);
    let mut cal_buf = Vec::new();
    std::io::Read::read_to_end(&mut file_take, &mut cal_buf)?;
    if cal_buf.len() > 1024 * 1024 {
        return Err(anyhow::anyhow!("File too large"));
    }
    let cal: CalibrationConfig = serde_json::from_slice(&cal_buf)?;
    println!("Loaded calibration offset C_room: {:.1} dB", cal.c_room);

    let room_eq = if let Some(path) = room_eq_path {
        if Path::new(path).exists() {
            let file = File::open(path)?;
            let mut file_take = std::io::Read::take(file, 1024 * 1024 + 1);
            let mut req_buf = Vec::new();
            std::io::Read::read_to_end(&mut file_take, &mut req_buf)?;
            if req_buf.len() > 1024 * 1024 {
                return Err(anyhow::anyhow!("File too large"));
            }
            let r: RoomEqConfig = serde_json::from_slice(&req_buf)?;
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
        if let Err(e) = update_eq(&client, f, cal.c_room, mode, fx_slot, &room_eq, iso226).await {
            eprintln!("Error applying initial EQ: {}", e);
        }
    }

    let is_midi = transport_used == "usb";
    let poll_interval = if is_midi {
        Duration::from_millis(200)
    } else {
        Duration::from_millis(100)
    };

    loop {
        if is_midi {
            // In USB MIDI mode: no subscription broadcasts, poll actively with delta thresholding
            sleep(poll_interval).await;
            if let Ok(OscArg::Float(f)) = client.query_value(fader_path).await {
                let delta = last_fader_val.map(|last| (f - last).abs()).unwrap_or(1.0);
                // Delta threshold: only update if change >= 0.01 (~0.5 dB in active fader range)
                if delta >= 0.01 {
                    last_fader_val = Some(f);
                    if let Err(e) =
                        update_eq(&client, f, cal.c_room, mode, fx_slot, &room_eq, iso226).await
                    {
                        eprintln!("Error updating EQ: {}", e);
                    }
                }
            }
        } else {
            tokio::select! {
                msg_res = rx.recv() => {
                    match msg_res {
                        Ok(msg) => {
                            if msg.path == fader_path {
                                if let Some(OscArg::Float(f)) = msg.args.first() {
                                    if Some(*f) != last_fader_val {
                                        last_fader_val = Some(*f);
                                        if let Err(e) = update_eq(&client, *f, cal.c_room, mode, fx_slot, &room_eq, iso226).await {
                                            eprintln!("Error updating EQ: {}", e);
                                        }
                                    }
                                }
                            }
                        }
                        _ => break,
                    }
                }
                _ = sleep(poll_interval) => {
                    // Periodically query fader in case client missed updates or mixer wasn't sending /xremote updates
                    if let Ok(OscArg::Float(f)) = client.query_value(fader_path).await {
                        if Some(f) != last_fader_val {
                            last_fader_val = Some(f);
                            if let Err(e) = update_eq(&client, f, cal.c_room, mode, fx_slot, &room_eq, iso226).await {
                                eprintln!("Error updating EQ: {}", e);
                            }
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
    iso226: bool,
) -> Result<()> {
    let fader_db = fader_to_db(fader_val);
    let spl_est = (fader_db + c_room).clamp(40.0, 100.0);

    let spl_diff = 85.0 - spl_est;

    match mode {
        LoudnessMode::Peq => {
            // Band 1: Low Shelf (80Hz). Band 6: High Shelf (10kHz).
            let dynamic_low = if spl_diff > 0.0 {
                if iso226 {
                    calculate_iso_226_gain(80.0, spl_est).clamp(0.0, 9.0)
                } else {
                    (0.22 * spl_diff).clamp(0.0, 9.0)
                }
            } else {
                0.0
            };
            let dynamic_high = if spl_diff > 0.0 {
                if iso226 {
                    calculate_iso_226_gain(10000.0, spl_est).clamp(0.0, 5.0)
                } else {
                    (0.12 * spl_diff).clamp(0.0, 5.0)
                }
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
                let dynamic_gain = if spl_diff > 0.0 {
                    if iso226 {
                        let raw_gain = calculate_iso_226_gain(freq, spl_est);
                        if raw_gain >= 0.0 {
                            raw_gain.clamp(0.0, 12.0)
                        } else {
                            raw_gain.clamp(-4.0, 0.0)
                        }
                    } else {
                        let k = geq_slope_factor(freq);
                        if k > 0.0 {
                            (k * spl_diff).clamp(0.0, 12.0)
                        } else {
                            (k * spl_diff).clamp(-4.0, 0.0)
                        }
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
                if iso226 {
                    calculate_iso_226_gain(80.0, spl_est).clamp(0.0, 9.0)
                } else {
                    (0.22 * spl_diff).clamp(0.0, 9.0)
                }
            } else {
                0.0
            };
            let dyn_b2 = if spl_diff > 0.0 {
                if iso226 {
                    calculate_iso_226_gain(500.0, spl_est).clamp(-2.0, 0.0)
                } else {
                    (-0.05 * spl_diff).clamp(-2.0, 0.0)
                }
            } else {
                0.0
            };
            let dyn_b3 = if spl_diff > 0.0 {
                if iso226 {
                    calculate_iso_226_gain(3200.0, spl_est).clamp(-4.0, 0.0)
                } else {
                    (-0.10 * spl_diff).clamp(-4.0, 0.0)
                }
            } else {
                0.0
            };
            let dyn_b4 = if spl_diff > 0.0 {
                if iso226 {
                    calculate_iso_226_gain(10000.0, spl_est).clamp(0.0, 5.0)
                } else {
                    (0.12 * spl_diff).clamp(0.0, 5.0)
                }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iso_226_interpolation() {
        let (a_low, l_low, t_low) = interpolate_iso_226(10.0);
        assert_eq!(a_low, ISO_226_ALPHA[0]);
        assert_eq!(l_low, ISO_226_LU[0]);
        assert_eq!(t_low, ISO_226_TF[0]);

        let (a_high, l_high, t_high) = interpolate_iso_226(20000.0);
        let len = ISO_226_FREQ.len();
        assert_eq!(a_high, ISO_226_ALPHA[len - 1]);
        assert_eq!(l_high, ISO_226_LU[len - 1]);
        assert_eq!(t_high, ISO_226_TF[len - 1]);

        let (a_1k, l_1k, t_1k) = interpolate_iso_226(1000.0);
        assert_eq!(a_1k, 0.250);
        assert_eq!(l_1k, 0.0);
        assert_eq!(t_1k, 2.4);
    }

    #[test]
    fn test_iso_226_spl_accuracy() {
        for phon in (0..=90).step_by(10) {
            let spl = iso_226_spl(1000.0, phon as f32);
            assert!(
                (spl - phon as f32).abs() < 0.1,
                "Phon {} got SPL {}",
                phon,
                spl
            );
        }
    }

    #[test]
    fn test_calculate_iso_226_gain() {
        let gain_1k_ref = calculate_iso_226_gain(1000.0, 85.0);
        assert!(gain_1k_ref.abs() < 0.001);

        let gain_80_ref = calculate_iso_226_gain(80.0, 85.0);
        assert!(gain_80_ref.abs() < 0.001);

        let gain_1k_low = calculate_iso_226_gain(1000.0, 60.0);
        assert!(gain_1k_low.abs() < 0.1);

        let gain_80_low = calculate_iso_226_gain(80.0, 60.0);
        assert!(gain_80_low > 0.0);

        let gain_10k_low = calculate_iso_226_gain(10000.0, 60.0);
        assert!(gain_10k_low > 0.0);
    }
}
