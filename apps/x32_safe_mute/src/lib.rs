#![allow(clippy::manual_range_contains)]
//! `x32_safe_mute` is a Panic Button feature that instantly silences the entire system safely
//! by using a rapid exponential fade instead of an instantaneous hard mute to avoid pops and thumps.

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use osc_lib::OscArg;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use x32_lib::{MixerClient, MixerModel, transport::udp::UdpTransport};

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};

/// The scope of the safe mute action.
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum Mode {
    /// Mutes Main L/R, all buses, all matrices
    All,
    /// Mutes all bus outputs (stage monitors)
    Monitors,
    /// Mutes Main L/R and matrices
    Main,
    /// Mutes only selected DCA groups
    Dca,
}

/// CLI arguments for x32_safe_mute
#[derive(Parser, Debug, Clone)]
#[command(author, version, about = "Safe Mute / Panic Button for X32/M32", long_about = None)]
pub struct Args {
    /// IP address of the X32 console
    #[arg(short, long)]
    pub ip: String,

    /// Mixer model: X32, Wing, XR18, XR16, XR12
    #[arg(short, long, default_value = "X32")]
    pub model: MixerModel,

    /// Mode to operate in (all, monitors, main, dca)
    #[arg(short, long, value_enum, default_value_t = Mode::All)]
    pub mode: Mode,

    /// Comma-separated list of DCA groups (1-8) if mode is dca (e.g. "1,2,3")
    #[arg(short, long, default_value = "")]
    pub dcas: String,
}

/// Resolves the base OSC paths for the given mode and mixer model.
/// Returns a vector of strings representing the base path (e.g. `/main/st/mix` or `/dca/1`).
pub fn resolve_target_paths(mode: &Mode, dcas: &str, model: MixerModel) -> Vec<String> {
    let mut paths = Vec::new();
    let (max_buses, max_matrices, max_dcas) = match model {
        MixerModel::X32 => (16, 6, 8),
        MixerModel::Wing => (28, 8, 16),
        MixerModel::XR18 => (6, 0, 4),
        MixerModel::XR16 => (4, 0, 4),
        MixerModel::XR12 => (2, 0, 4),
    };

    match mode {
        Mode::All => {
            paths.push("/main/st/mix".to_string());
            for i in 1..=max_buses {
                paths.push(format!("/bus/{:02}/mix", i));
            }
            for i in 1..=max_matrices {
                paths.push(format!("/mtx/{:02}/mix", i));
            }
        }
        Mode::Monitors => {
            for i in 1..=max_buses {
                paths.push(format!("/bus/{:02}/mix", i));
            }
        }
        Mode::Main => {
            paths.push("/main/st/mix".to_string());
            for i in 1..=max_matrices {
                paths.push(format!("/mtx/{:02}/mix", i));
            }
        }
        Mode::Dca => {
            if !dcas.is_empty() {
                for part in dcas.split(',') {
                    if let Ok(num) = part.trim().parse::<u8>() {
                        if num >= 1 && num <= max_dcas {
                            paths.push(format!("/dca/{}", num));
                        }
                    }
                }
            }
        }
    }
    paths
}

/// Represents the stored state of a channel or output
#[derive(Clone, Debug)]
pub struct ChannelState {
    /// The fader level
    pub fader: f32,
    /// The mute state (1 = on, 0 = off/muted)
    pub on: i32,
}

/// Fetches the current fader levels and mute states for all target paths
pub async fn capture_state(
    client: &MixerClient,
    paths: &[String],
) -> Result<HashMap<String, ChannelState>> {
    let mut state = HashMap::new();
    for path in paths {
        let mut cstate = ChannelState { fader: 0.0, on: 0 };
        let fader_path = format!("{}/fader", path);
        let on_path = format!("{}/on", path);
        if let Ok(OscArg::Float(fval)) = client.query_value(&fader_path).await {
            cstate.fader = fval;
        }
        if let Ok(OscArg::Int(ival)) = client.query_value(&on_path).await {
            cstate.on = ival;
        }
        state.insert(path.clone(), cstate);
    }
    Ok(state)
}

/// Executes a rapid fade out and hard mute
pub async fn execute_panic(
    client: &MixerClient,
    paths: &[String],
    state: &HashMap<String, ChannelState>,
) -> Result<()> {
    // 250ms fade down, step size 50ms (5 steps)
    let steps = 5;
    let mut interval = interval(Duration::from_millis(50));

    for step in 1..=steps {
        interval.tick().await;
        // exponential curve: t^2 towards 0
        let factor = 1.0 - (step as f32 / steps as f32);
        let factor = factor * factor;

        for path in paths {
            if let Some(initial_state) = state.get(path) {
                let current_val = initial_state.fader * factor;
                let _ = client
                    .send_message(&format!("{}/fader", path), vec![OscArg::Float(current_val)])
                    .await;
            }
        }
    }

    // Hard mute (set /on to 0)
    for path in paths {
        let _ = client
            .send_message(&format!("{}/on", path), vec![OscArg::Int(0)])
            .await;
        // ensure fader is fully at 0
        let _ = client
            .send_message(&format!("{}/fader", path), vec![OscArg::Float(0.0)])
            .await;
    }

    Ok(())
}

/// Executes a slow fade in back to the stored state
pub async fn execute_restore(
    client: &MixerClient,
    paths: &[String],
    state: &HashMap<String, ChannelState>,
) -> Result<()> {
    // Unmute based on stored state
    for path in paths {
        if let Some(cstate) = state.get(path) {
            let _ = client
                .send_message(&format!("{}/on", path), vec![OscArg::Int(cstate.on)])
                .await;
        }
    }

    // 1000ms fade up, step size 50ms (20 steps)
    let steps = 20;
    let mut interval = interval(Duration::from_millis(50));

    for step in 1..=steps {
        interval.tick().await;
        // linear fade in
        let factor = step as f32 / steps as f32;

        for path in paths {
            if let Some(cstate) = state.get(path) {
                let current_val = cstate.fader * factor;
                let _ = client
                    .send_message(&format!("{}/fader", path), vec![OscArg::Float(current_val)])
                    .await;
            }
        }
    }

    Ok(())
}

/// Main entry point for the tool
pub async fn run(args: Args) -> Result<()> {
    let paths = resolve_target_paths(&args.mode, &args.dcas, args.model);
    let transport = UdpTransport::connect(&args.ip)
        .await
        .context("Failed to connect UDP transport")?;
    let mut client = MixerClient::new(Arc::new(transport), true);
    run_ui_loop(&args, &mut client, &paths).await?;
    Ok(())
}

/// Run the TUI event loop for panic monitoring
pub async fn run_ui_loop(args: &Args, client: &mut MixerClient, paths: &[String]) -> Result<()> {
    enable_raw_mode()?;
    scopeguard::defer! {
        let _ = disable_raw_mode();
    }

    println!("Safe Mute / Panic Button Armed.");
    println!("Mode: {:?}", args.mode);
    println!("Target paths: {}", paths.len());
    println!("Press 'Esc' to PANIC (Mute all targets).");
    println!("Press 'r' to RESTORE (Unmute and fade back up).");
    println!("Press 'q' to quit.\r");

    let mut saved_state: Option<HashMap<String, ChannelState>> = None;
    let mut is_panicked = false;

    loop {
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Esc => {
                        if !is_panicked {
                            is_panicked = true;
                            println!("\r\n[PANIC] Initiating safe mute fade out...\r");
                            // Capture current state before panicking
                            if let Ok(state) = capture_state(client, paths).await {
                                saved_state = Some(state.clone());
                                let _ = execute_panic(client, paths, &state).await;
                                println!("Fade out complete. Outputs hard-muted.\r");
                            } else {
                                println!("Failed to capture state!\r");
                                is_panicked = false;
                            }

                            // Flush any subsequent keystrokes that arrived while blocking
                            while event::poll(Duration::from_secs(0))? {
                                let _ = event::read()?;
                            }
                        } else {
                            println!("\r\n[PANIC] System already panicked.\r");
                        }
                    }
                    KeyCode::Char('r') => {
                        if let Some(state) = &saved_state {
                            println!("\r\n[RESTORE] Restore all outputs? (y/N)\r");
                            let mut confirm = false;
                            loop {
                                if event::poll(Duration::from_millis(100))? {
                                    if let Event::Key(k) = event::read()? {
                                        match k.code {
                                            KeyCode::Char('y') | KeyCode::Char('Y') => {
                                                confirm = true;
                                                break;
                                            }
                                            KeyCode::Char('n')
                                            | KeyCode::Char('N')
                                            | KeyCode::Esc => {
                                                break;
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                            if confirm {
                                println!("\r\n[RESTORE] Restoring fader levels...\r");
                                let _ = execute_restore(client, paths, state).await;
                                println!("Restore complete.\r");
                                is_panicked = false;
                                saved_state = None;
                            } else {
                                println!("\r\n[RESTORE] Canceled.\r");
                            }
                        } else {
                            println!("\r\n[RESTORE] No saved state to restore!\r");
                        }
                    }
                    KeyCode::Char('q') | KeyCode::Char('c') => {
                        break;
                    }
                    _ => {}
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
    fn test_x32_target_paths() {
        let all_paths = resolve_target_paths(&Mode::All, "", MixerModel::X32);
        assert!(all_paths.contains(&"/main/st/mix".to_string()));
        assert!(all_paths.contains(&"/bus/01/mix".to_string()));
        assert!(all_paths.contains(&"/bus/16/mix".to_string()));
        assert!(all_paths.contains(&"/mtx/01/mix".to_string()));
        assert!(all_paths.contains(&"/mtx/06/mix".to_string()));
        assert_eq!(all_paths.len(), 1 + 16 + 6);

        let mon_paths = resolve_target_paths(&Mode::Monitors, "", MixerModel::X32);
        assert_eq!(mon_paths.len(), 16);
        assert!(!mon_paths.contains(&"/main/st/mix".to_string()));

        let main_paths = resolve_target_paths(&Mode::Main, "", MixerModel::X32);
        assert_eq!(main_paths.len(), 1 + 6);

        let dca_paths = resolve_target_paths(&Mode::Dca, "1, 3, 8, 9, foo", MixerModel::X32);
        assert_eq!(dca_paths.len(), 3);
        assert!(dca_paths.contains(&"/dca/1".to_string()));
        assert!(dca_paths.contains(&"/dca/3".to_string()));
        assert!(dca_paths.contains(&"/dca/8".to_string()));
        assert!(!dca_paths.contains(&"/dca/9".to_string()));
    }

    #[test]
    fn test_wing_target_paths() {
        let all_paths = resolve_target_paths(&Mode::All, "", MixerModel::Wing);
        assert_eq!(all_paths.len(), 1 + 28 + 8);
        assert!(all_paths.contains(&"/bus/28/mix".to_string()));
        assert!(all_paths.contains(&"/mtx/08/mix".to_string()));

        let mon_paths = resolve_target_paths(&Mode::Monitors, "", MixerModel::Wing);
        assert_eq!(mon_paths.len(), 28);

        let main_paths = resolve_target_paths(&Mode::Main, "", MixerModel::Wing);
        assert_eq!(main_paths.len(), 1 + 8);

        let dca_paths = resolve_target_paths(&Mode::Dca, "1, 10, 16, 17", MixerModel::Wing);
        assert_eq!(dca_paths.len(), 3);
        assert!(dca_paths.contains(&"/dca/16".to_string()));
        assert!(!dca_paths.contains(&"/dca/17".to_string()));
    }

    #[test]
    fn test_xr18_target_paths() {
        let all_paths = resolve_target_paths(&Mode::All, "", MixerModel::XR18);
        assert_eq!(all_paths.len(), 1 + 6 + 0);
        assert!(all_paths.contains(&"/bus/06/mix".to_string()));
        assert!(!all_paths.contains(&"/bus/07/mix".to_string()));

        let main_paths = resolve_target_paths(&Mode::Main, "", MixerModel::XR18);
        assert_eq!(main_paths.len(), 1);

        let dca_paths = resolve_target_paths(&Mode::Dca, "1, 4, 5", MixerModel::XR18);
        assert_eq!(dca_paths.len(), 2);
        assert!(dca_paths.contains(&"/dca/4".to_string()));
        assert!(!dca_paths.contains(&"/dca/5".to_string()));
    }

    #[test]
    fn test_xr16_target_paths() {
        let all_paths = resolve_target_paths(&Mode::All, "", MixerModel::XR16);
        assert_eq!(all_paths.len(), 1 + 4 + 0);

        let mon_paths = resolve_target_paths(&Mode::Monitors, "", MixerModel::XR16);
        assert_eq!(mon_paths.len(), 4);
    }

    #[test]
    fn test_xr12_target_paths() {
        let all_paths = resolve_target_paths(&Mode::All, "", MixerModel::XR12);
        assert_eq!(all_paths.len(), 1 + 2 + 0);

        let dca_paths = resolve_target_paths(&Mode::Dca, "1, 2, 3, 4, 5", MixerModel::XR12);
        assert_eq!(dca_paths.len(), 4);
    }
}
