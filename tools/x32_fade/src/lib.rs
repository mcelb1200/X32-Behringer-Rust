//! `x32_fade` is a command-line utility for controlling and fading faders on a
//! Behringer X32 or Midas M32 digital mixing console.
//!
//! # Credits
//!
//! *   **Original concept and work on the C library:** Patrick-Gilles Maillot
//! *   **Additional concepts by:** mcelb1200
//! *   **Rust implementation by:** mcelb1200

use anyhow::Result;
use clap::Parser;
use osc_lib::{OscArg, OscMessage};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;
use x32_lib::{MixerClient, MixerModel, get_parameter_async};

/// Resolves default target OSC port based on digital mixer model when port is omitted.
pub fn resolve_target_address(ip: &str, model: MixerModel) -> String {
    model.resolve_target_address(ip)
}

/// Checks whether a fader OSC address path is within the hardware bounds of the specified digital mixer model.
///
/// Returns `Ok(())` if valid or non-standard/auxiliary, or an informative warning message if out-of-bounds.
pub fn check_fader_path_for_model(path: &str, model: MixerModel) -> Result<(), String> {
    let (max_ch, max_bus, max_dca, max_mtx, max_aux, max_fxrtn) = match model {
        MixerModel::Wing => (40, 28, 16, 16, 16, 16),
        MixerModel::X32 => (32, 16, 8, 6, 8, 8),
        MixerModel::XR18 => (16, 6, 4, 0, 2, 4),
        MixerModel::XR16 => (16, 4, 4, 0, 2, 4),
        MixerModel::XR12 => (12, 2, 4, 0, 2, 4),
    };

    if let Some(stripped) = path.strip_prefix("/ch/") {
        if let Some((idx_str, rest)) = stripped.split_once('/') {
            if rest == "mix/fader" || rest == "fdr" {
                if let Ok(idx) = idx_str.parse::<u32>() {
                    if idx < 1 || idx > max_ch {
                        return Err(format!(
                            "Channel {} out of bounds for {:?} (max: {})",
                            idx, model, max_ch
                        ));
                    }
                }
            }
        }
    } else if let Some(stripped) = path.strip_prefix("/bus/") {
        if let Some((idx_str, rest)) = stripped.split_once('/') {
            if rest == "mix/fader" || rest == "fdr" {
                if let Ok(idx) = idx_str.parse::<u32>() {
                    if idx < 1 || idx > max_bus {
                        return Err(format!(
                            "Bus {} out of bounds for {:?} (max: {})",
                            idx, model, max_bus
                        ));
                    }
                }
            }
        }
    } else if let Some(stripped) = path.strip_prefix("/dca/") {
        if let Some((idx_str, rest)) = stripped.split_once('/') {
            if rest == "fader" || rest == "fdr" {
                if let Ok(idx) = idx_str.parse::<u32>() {
                    if idx < 1 || idx > max_dca {
                        return Err(format!(
                            "DCA {} out of bounds for {:?} (max: {})",
                            idx, model, max_dca
                        ));
                    }
                }
            }
        }
    } else if let Some(stripped) = path.strip_prefix("/mtx/") {
        if let Some((idx_str, rest)) = stripped.split_once('/') {
            if rest == "mix/fader" || rest == "fdr" {
                if let Ok(idx) = idx_str.parse::<u32>() {
                    if max_mtx == 0 || idx < 1 || idx > max_mtx {
                        return Err(format!(
                            "Matrix {} out of bounds for {:?} (max: {})",
                            idx, model, max_mtx
                        ));
                    }
                }
            }
        }
    } else if let Some(stripped) = path.strip_prefix("/auxin/") {
        if let Some((idx_str, rest)) = stripped.split_once('/') {
            if rest == "mix/fader" || rest == "fdr" {
                if let Ok(idx) = idx_str.parse::<u32>() {
                    if idx < 1 || idx > max_aux {
                        return Err(format!(
                            "Auxin {} out of bounds for {:?} (max: {})",
                            idx, model, max_aux
                        ));
                    }
                }
            }
        }
    } else if let Some(stripped) = path.strip_prefix("/fxrtn/") {
        if let Some((idx_str, rest)) = stripped.split_once('/') {
            if rest == "mix/fader" || rest == "fdr" {
                if let Ok(idx) = idx_str.parse::<u32>() {
                    if idx < 1 || idx > max_fxrtn {
                        return Err(format!(
                            "FX Return {} out of bounds for {:?} (max: {})",
                            idx, model, max_fxrtn
                        ));
                    }
                }
            }
        }
    }

    Ok(())
}

/// A command-line tool to control and fade X32 faders.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    pub ip: Option<String>,

    /// Digital mixer model family (X32, Wing, XR18, XR16, XR12)
    #[arg(short = 'M', long, value_enum, default_value = "X32")]
    pub model: MixerModel,

    #[arg(long, default_value = "auto")]
    pub transport: String,

    #[arg(long, default_value = "")]
    pub usb_port: String,

    #[arg(long, default_value = "")]
    pub aes50_ip: String,

    /// A fader to control. Can be specified multiple times.
    /// E.g., --fader /ch/01/mix/fader
    #[arg(long)]
    pub fader: Vec<String>,

    /// Fade-in time in seconds.
    #[arg(long)]
    pub fade_in: Option<f32>,

    /// Fade-out time in seconds.
    #[arg(long)]
    pub fade_out: Option<f32>,

    /// Number of steps for the fade.
    #[arg(long)]
    pub steps: Option<u32>,

    /// Path to a config file to load.
    #[arg(long)]
    pub load_config: Option<PathBuf>,

    /// Path to save the config file to.
    #[arg(long)]
    pub save_config: Option<PathBuf>,

    /// Verbose mode.
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,

    /// Debug mode.
    #[arg(short, long, default_value_t = false)]
    pub debug: bool,

    /// Listen mode: stay alive and listen for OSC userpar commands 17 (Fade In), 18 (Fade Out), and 19 (Stop).
    #[arg(short, long, default_value_t = false)]
    pub listen: bool,
}

/// Represents the configuration for a fade operation, serializable to/from JSON.
#[derive(Serialize, Deserialize, Debug, Default)]
struct Config {
    /// The IP address of the X32 console.
    ip: Option<String>,
    /// A list of OSC fader addresses to control.
    faders: Vec<String>,
    /// The duration of the fade-in in seconds.
    fade_in: Option<f32>,
    /// The duration of the fade-out in seconds.
    fade_out: Option<f32>,
    /// The number of steps to use for the fade.
    steps: Option<u32>,
}

/// The main entry point for the application.
pub async fn run(args: Args) -> Result<()> {
    let mut config = Config::default();

    // Load configuration from a file if specified.
    if let Some(path) = &args.load_config {
        let f = std::fs::File::open(path)?;

        // Sentinel: Prevent OOM from maliciously large or corrupted config files
        if f.metadata()?.len() > 1024 * 1024 {
            return Err(anyhow::anyhow!("Config file too large to load (max 1MB)"));
        }

        let mut data = String::new();
        f.take(1024 * 1024 + 1).read_to_string(&mut data)?;
        if data.len() > 1024 * 1024 {
            return Err(anyhow::anyhow!("Config file too large to load (max 1MB)"));
        }
        config = serde_json::from_str(&data)?;
        if args.debug {
            println!("Loaded config from {:?}: {:?}", path, config);
        }
    }

    // Command-line arguments override config file values.
    if args.ip.is_some() {
        config.ip = args.ip;
    }
    if !args.fader.is_empty() {
        config.faders = args.fader;
    }
    if args.fade_in.is_some() {
        config.fade_in = args.fade_in;
    }
    if args.fade_out.is_some() {
        config.fade_out = args.fade_out;
    }
    if args.steps.is_some() {
        config.steps = args.steps;
    }

    // If a save path is specified, write the config and exit.
    if let Some(path) = &args.save_config {
        let data = serde_json::to_string_pretty(&config)?;
        fs::write(path, data)?;
        if args.verbose {
            println!("Saved config to {:?}", path);
        }
        return Ok(());
    }

    // Validate fader paths against mixer model
    for fader in &config.faders {
        if let Err(warning) = check_fader_path_for_model(fader, args.model) {
            if args.verbose {
                eprintln!("Warning: {}", warning);
            }
        }
    }

    // Ensure we have an IP address before proceeding.
    if let Some(ip) = &config.ip {
        let target_ip = resolve_target_address(ip, args.model);
        let (client, _) = MixerClient::connect_with_transport(
            &target_ip,
            &args.aes50_ip,
            &args.usb_port,
            &args.transport,
            args.listen, // Use listen flag to keep /xremote alive
        )
        .await?;
        let client = std::sync::Arc::new(client);
        if args.verbose {
            println!("Connected to {:?} at {}", args.model, target_ip);
        }

        let steps = config.steps.unwrap_or(32);

        let initial_cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

        // Perform fade-in if specified.
        if let Some(fade_in_duration) = config.fade_in {
            if fade_in_duration > 0.0 {
                if args.verbose {
                    println!(
                        "Fading in over {} seconds in {} steps.",
                        fade_in_duration, steps
                    );
                }
                fade(
                    &client,
                    &config.faders,
                    fade_in_duration,
                    steps,
                    true,
                    args.verbose,
                    initial_cancel.clone(),
                )
                .await?;
            }
        }

        // Perform fade-out if specified.
        if let Some(fade_out_duration) = config.fade_out {
            if fade_out_duration > 0.0 {
                if args.verbose {
                    println!(
                        "Fading out over {} seconds in {} steps.",
                        fade_out_duration, steps
                    );
                }
                fade(
                    &client,
                    &config.faders,
                    fade_out_duration,
                    steps,
                    false,
                    args.verbose,
                    initial_cancel.clone(),
                )
                .await?;
            }
        }

        if args.listen {
            if args.verbose {
                println!(
                    "Listening for userpar commands 17 (Fade In), 18 (Fade Out), and 19 (Stop)..."
                );
            }
            let mut rx = client.subscribe();
            let mut current_cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

            while let Ok(msg) = rx.recv().await {
                let triggered = match msg.args.first() {
                    Some(OscArg::Float(f)) => *f > 0.5,
                    Some(OscArg::Int(i)) => *i == 1,
                    _ => false,
                };

                if triggered {
                    match msg.path.as_str() {
                        "/-stat/userpar/17/value" => {
                            // Fade in
                            current_cancel.store(true, std::sync::atomic::Ordering::Relaxed);
                            current_cancel =
                                std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

                            let cancel_clone = current_cancel.clone();
                            let faders_clone = config.faders.clone();
                            let client_clone = client.clone();
                            let fade_time = config.fade_in.unwrap_or(0.0);
                            let verb = args.verbose;

                            if fade_time > 0.0 {
                                tokio::spawn(async move {
                                    let _ = fade(
                                        &client_clone,
                                        &faders_clone,
                                        fade_time,
                                        steps,
                                        true,
                                        verb,
                                        cancel_clone,
                                    )
                                    .await;
                                });
                            }
                        }
                        "/-stat/userpar/18/value" => {
                            // Fade out
                            current_cancel.store(true, std::sync::atomic::Ordering::Relaxed);
                            current_cancel =
                                std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

                            let cancel_clone = current_cancel.clone();
                            let faders_clone = config.faders.clone();
                            let client_clone = client.clone();
                            let fade_time = config.fade_out.unwrap_or(0.0);
                            let verb = args.verbose;

                            if fade_time > 0.0 {
                                tokio::spawn(async move {
                                    let _ = fade(
                                        &client_clone,
                                        &faders_clone,
                                        fade_time,
                                        steps,
                                        false,
                                        verb,
                                        cancel_clone,
                                    )
                                    .await;
                                });
                            }
                        }
                        "/-stat/userpar/19/value" => {
                            // Stop
                            current_cancel.store(true, std::sync::atomic::Ordering::Relaxed);
                        }
                        _ => {}
                    }
                }
            }
        }
    } else {
        println!("Error: No IP address specified either via --ip or --load-config.");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Instant;

    #[tokio::test]
    async fn test_fade_cancellation() {
        let cancel = Arc::new(AtomicBool::new(false));

        let faders = vec!["/ch/01/mix/fader".to_string()];

        let cancel_clone = cancel.clone();
        tokio::spawn(async move {
            sleep(Duration::from_millis(100)).await;
            cancel_clone.store(true, Ordering::Relaxed);
        });

        // Start an embedded mock emulator so `get_parameter_async` inside `fade` does not timeout
        let emulator_socket = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let emulator_addr = emulator_socket.local_addr().unwrap();

        tokio::spawn(async move {
            let mut buf = [0u8; 1024];
            while let Ok((len, src)) = emulator_socket.recv_from(&mut buf).await {
                if let Ok(msg) = OscMessage::from_bytes(&buf[..len]) {
                    if msg.args.is_empty() {
                        let response = OscMessage::new(msg.path, vec![OscArg::Float(0.5)]);
                        let _ = emulator_socket
                            .send_to(&response.to_bytes().unwrap(), src)
                            .await;
                    }
                }
            }
        });

        let client = MixerClient::connect(&emulator_addr.to_string(), false)
            .await
            .unwrap();
        let client = Arc::new(client);

        let start = Instant::now();
        // This fade should take 5 seconds if not cancelled, but we cancel it after 100ms
        fade(&client, &faders, 5.0, 50, true, false, cancel)
            .await
            .unwrap();
        let duration = start.elapsed();

        // Ensure it cancelled early
        assert!(duration < Duration::from_secs(4));
    }

    #[test]
    fn test_resolve_target_address() {
        assert_eq!(
            resolve_target_address("192.168.1.10", MixerModel::X32),
            "192.168.1.10:10023"
        );
        assert_eq!(
            resolve_target_address("192.168.1.10", MixerModel::Wing),
            "192.168.1.10:2222"
        );
        assert_eq!(
            resolve_target_address("192.168.1.10", MixerModel::XR18),
            "192.168.1.10:10024"
        );
        assert_eq!(
            resolve_target_address("192.168.1.10", MixerModel::XR16),
            "192.168.1.10:10024"
        );
        assert_eq!(
            resolve_target_address("192.168.1.10", MixerModel::XR12),
            "192.168.1.10:10024"
        );
        // Custom port override preserved
        assert_eq!(
            resolve_target_address("192.168.1.10:9999", MixerModel::X32),
            "192.168.1.10:9999"
        );
    }

    #[test]
    fn test_check_fader_path_for_model() {
        // Main and generic paths
        assert!(check_fader_path_for_model("/main/st/mix/fader", MixerModel::X32).is_ok());
        assert!(check_fader_path_for_model("/main/m/mix/fader", MixerModel::Wing).is_ok());
        assert!(check_fader_path_for_model("/lr/mix/fader", MixerModel::XR18).is_ok());

        // Channels (X32 & Wing syntax)
        assert!(check_fader_path_for_model("/ch/32/mix/fader", MixerModel::X32).is_ok());
        assert!(check_fader_path_for_model("/ch/33/mix/fader", MixerModel::X32).is_err());
        assert!(check_fader_path_for_model("/ch/40/mix/fader", MixerModel::Wing).is_ok());
        assert!(check_fader_path_for_model("/ch/40/fdr", MixerModel::Wing).is_ok());
        assert!(check_fader_path_for_model("/ch/41/mix/fader", MixerModel::Wing).is_err());
        assert!(check_fader_path_for_model("/ch/16/mix/fader", MixerModel::XR18).is_ok());
        assert!(check_fader_path_for_model("/ch/17/mix/fader", MixerModel::XR18).is_err());
        assert!(check_fader_path_for_model("/ch/12/mix/fader", MixerModel::XR12).is_ok());
        assert!(check_fader_path_for_model("/ch/13/mix/fader", MixerModel::XR12).is_err());
        assert!(check_fader_path_for_model("/ch/0/mix/fader", MixerModel::X32).is_err());

        // Buses
        assert!(check_fader_path_for_model("/bus/16/mix/fader", MixerModel::X32).is_ok());
        assert!(check_fader_path_for_model("/bus/17/mix/fader", MixerModel::X32).is_err());
        assert!(check_fader_path_for_model("/bus/28/mix/fader", MixerModel::Wing).is_ok());
        assert!(check_fader_path_for_model("/bus/29/mix/fader", MixerModel::Wing).is_err());
        assert!(check_fader_path_for_model("/bus/6/mix/fader", MixerModel::XR18).is_ok());
        assert!(check_fader_path_for_model("/bus/7/mix/fader", MixerModel::XR18).is_err());
        assert!(check_fader_path_for_model("/bus/4/mix/fader", MixerModel::XR16).is_ok());
        assert!(check_fader_path_for_model("/bus/5/mix/fader", MixerModel::XR16).is_err());
        assert!(check_fader_path_for_model("/bus/2/mix/fader", MixerModel::XR12).is_ok());
        assert!(check_fader_path_for_model("/bus/3/mix/fader", MixerModel::XR12).is_err());

        // DCAs
        assert!(check_fader_path_for_model("/dca/8/fader", MixerModel::X32).is_ok());
        assert!(check_fader_path_for_model("/dca/9/fader", MixerModel::X32).is_err());
        assert!(check_fader_path_for_model("/dca/16/fader", MixerModel::Wing).is_ok());
        assert!(check_fader_path_for_model("/dca/17/fader", MixerModel::Wing).is_err());
        assert!(check_fader_path_for_model("/dca/4/fader", MixerModel::XR18).is_ok());
        assert!(check_fader_path_for_model("/dca/5/fader", MixerModel::XR18).is_err());

        // Matrices
        assert!(check_fader_path_for_model("/mtx/6/mix/fader", MixerModel::X32).is_ok());
        assert!(check_fader_path_for_model("/mtx/7/mix/fader", MixerModel::X32).is_err());
        assert!(check_fader_path_for_model("/mtx/16/mix/fader", MixerModel::Wing).is_ok());
        assert!(check_fader_path_for_model("/mtx/17/mix/fader", MixerModel::Wing).is_err());
        assert!(check_fader_path_for_model("/mtx/1/mix/fader", MixerModel::XR18).is_err());

        // Auxin & FX Return
        assert!(check_fader_path_for_model("/auxin/08/mix/fader", MixerModel::X32).is_ok());
        assert!(check_fader_path_for_model("/auxin/09/mix/fader", MixerModel::X32).is_err());
        assert!(check_fader_path_for_model("/fxrtn/08/mix/fader", MixerModel::X32).is_ok());
        assert!(check_fader_path_for_model("/fxrtn/09/mix/fader", MixerModel::X32).is_err());
    }

    #[test]
    fn test_args_parsing_with_model() {
        let args_default = Args::parse_from(["x32_fade", "--ip", "127.0.0.1"]);
        assert_eq!(args_default.model, MixerModel::X32);

        let args_wing = Args::parse_from(["x32_fade", "--ip", "127.0.0.1", "-M", "Wing"]);
        assert_eq!(args_wing.model, MixerModel::Wing);

        let args_xr18 = Args::parse_from(["x32_fade", "--ip", "127.0.0.1", "--model", "xr18"]);
        assert_eq!(args_xr18.model, MixerModel::XR18);
    }
}

/// Performs a fade operation on a set of faders.
///
/// # Arguments
///
/// * `client` - The MixerClient connected to the X32.
/// * `faders` - A slice of strings representing the OSC addresses of the faders to control.
/// * `duration_s` - The duration of the fade in seconds.
/// * `steps` - The number of steps to use for the fade.
/// * `is_fade_in` - If `true`, performs a fade-in (to 0.75). If `false`, performs a fade-out (to 0.0).
/// * `verbose` - If `true`, prints the OSC messages being sent.
/// * `cancel` - An atomic bool that aborts the fade loop early if set to true.
async fn fade(
    client: &MixerClient,
    faders: &[String],
    duration_s: f32,
    steps: u32,
    is_fade_in: bool,
    verbose: bool,
    cancel: std::sync::Arc<std::sync::atomic::AtomicBool>,
) -> Result<()> {
    if steps == 0 || duration_s <= 0.0 {
        return Ok(());
    }

    // Query the initial level of each fader.
    let mut initial_levels = Vec::new();
    for fader_addr in faders {
        initial_levels.push(get_parameter_async(client, fader_addr).await?);
    }

    let target_level = if is_fade_in { 0.75 } else { 0.0 };
    let step_interval = Duration::from_secs_f32(duration_s / steps as f32);

    for i in 1..=steps {
        if cancel.load(std::sync::atomic::Ordering::Relaxed) {
            if verbose {
                println!("Fade aborted midway.");
            }
            return Ok(());
        }

        for (fader_addr, &initial_level) in faders.iter().zip(&initial_levels) {
            let current_level = if is_fade_in {
                initial_level + ((target_level - initial_level) / steps as f32) * i as f32
            } else {
                initial_level - ((initial_level - target_level) / steps as f32) * i as f32
            };

            let msg = OscMessage::new(fader_addr.clone(), vec![OscArg::Float(current_level)]);

            client.send_message(&msg.path, msg.args.clone()).await?;
            if verbose {
                println!("Sent: {} {}", fader_addr, current_level);
            }
        }

        sleep(step_interval).await;
    }

    if cancel.load(std::sync::atomic::Ordering::Relaxed) {
        if verbose {
            println!("Fade aborted midway.");
        }
        return Ok(());
    }

    // Send the final target level to ensure accuracy.
    for fader_addr in faders {
        let msg = OscMessage::new(fader_addr.clone(), vec![OscArg::Float(target_level)]);

        client.send_message(&msg.path, msg.args.clone()).await?;
        if verbose {
            println!("Sent final: {} {}", fader_addr, target_level);
        }
    }

    Ok(())
}
