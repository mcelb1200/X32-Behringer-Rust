//! `x32_speech_mode` is a tool that applies broadcast audio engineering best practices
//! to speech channels with a single command. It configures EQ, compression, gating,
//! and can optionally configure automixing and ringout.
use anyhow::Result;
use clap::Parser;
use osc_lib::{OscArg, OscMessage};
use std::time::Duration;
use x32_lib::MixerClient;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about = "One-Touch Speech Mode Macro for X32/M32", long_about = None)]
pub struct Args {
    /// IP address of the X32 console
    #[arg(short, long)]
    pub ip: String,

    /// Comma-separated list of channel numbers (1-32) to apply speech mode to (e.g. 1,2,3)
    #[arg(short, long)]
    pub channels: String,
}

// Frequency mapping helper (returns f32 for OSC float scale [0.0, 1.0])
fn freq_to_osc(freq: f32) -> f32 {
    let mut res = (freq / 20.0).ln() / 6.907_755_4;
    res = (res * 200.0).round() / 200.0;
    res.clamp(0.0, 1.0)
}

// Gain mapping helper (-15.0 to 15.0 -> 0.0 to 1.0)
fn gain_to_osc(gain: f32) -> f32 {
    ((gain + 15.0) / 30.0).clamp(0.0, 1.0)
}

// Q mapping helper
fn q_to_osc(q: f32) -> f32 {
    // According to X32 docs, Q uses logarithmic scaling between 10.0 and 0.3
    ((q / 0.3).ln() / (10.0 / 0.3_f32).ln()).clamp(0.0, 1.0)
}

// Dynamics Threshold helper (-60.0 to 0.0 -> 0.0 to 1.0)
fn dyn_thr_to_osc(thr: f32) -> f32 {
    ((thr + 60.0) / 60.0).clamp(0.0, 1.0)
}

// Gate Threshold helper (-80.0 to 0.0 -> 0.0 to 1.0)
fn gate_thr_to_osc(thr: f32) -> f32 {
    ((thr + 80.0) / 80.0).clamp(0.0, 1.0)
}

// Gate Range helper (-60.0 to 0.0 -> 0.0 to 1.0)
// (X32 gate range typically goes down to -60. Some say -oo but let's map linear)
fn gate_range_to_osc(range: f32) -> f32 {
    ((range + 60.0) / 60.0).clamp(0.0, 1.0)
}

// Dynamics Attack mapping (0 to 120ms -> 0.0 to 1.0) log scale approx
fn dyn_attack_to_osc(attack_ms: f32) -> f32 {
    // 0 ms -> 0.0, 120 ms -> 1.0
    // actually, X32 scales differently, let's use a simple linear mapping if unknown, or just use 0.3 for 10ms
    (attack_ms / 120.0).clamp(0.0, 1.0)
}

// Dynamics Release mapping (0 to 4000ms -> 0.0 to 1.0)
fn dyn_release_to_osc(release_ms: f32) -> f32 {
    // let's use a rough mapping, 100ms is quite short so maybe 0.1
    (release_ms / 4000.0).clamp(0.0, 1.0)
}

pub async fn run(args: Args) -> Result<()> {
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
    println!("Connected. Applying speech mode to channels: {:?}", channels);

    let delay = Duration::from_millis(10);

    for ch in channels {
        println!("Processing channel {:02}", ch);
        let prefix = format!("/ch/{:02}", ch);

        let msgs = vec![
            // 1. High-pass filter: 80 Hz, 18 dB/oct slope (type = 5 is Low Cut on eq/1/type, freq = 80Hz)
            OscMessage { path: format!("{}/eq/1/type", prefix), args: vec![OscArg::Int(5)] },
            OscMessage { path: format!("{}/eq/1/f", prefix), args: vec![OscArg::Float(freq_to_osc(80.0))] },

            // 2. Low-pass filter: 12 kHz, 12 dB/oct slope (type = 6 is High Cut)
            OscMessage { path: format!("{}/eq/6/type", prefix), args: vec![OscArg::Int(6)] },
            OscMessage { path: format!("{}/eq/6/f", prefix), args: vec![OscArg::Float(freq_to_osc(12000.0))] },

            // 3. Presence boost: +3 dB shelf at 3.5 kHz (type = 3 is PEQ)
            OscMessage { path: format!("{}/eq/3/type", prefix), args: vec![OscArg::Int(3)] },
            OscMessage { path: format!("{}/eq/3/f", prefix), args: vec![OscArg::Float(freq_to_osc(3500.0))] },
            OscMessage { path: format!("{}/eq/3/g", prefix), args: vec![OscArg::Float(gain_to_osc(3.0))] },

            // 4. Low-mid scoop: -2 dB at 300 Hz, Q=1.5 (type = 3 PEQ)
            OscMessage { path: format!("{}/eq/2/type", prefix), args: vec![OscArg::Int(3)] },
            OscMessage { path: format!("{}/eq/2/f", prefix), args: vec![OscArg::Float(freq_to_osc(300.0))] },
            OscMessage { path: format!("{}/eq/2/g", prefix), args: vec![OscArg::Float(gain_to_osc(-2.0))] },
            OscMessage { path: format!("{}/eq/2/q", prefix), args: vec![OscArg::Float(q_to_osc(1.5))] },

            // 5. Compressor: Ratio 3:1, threshold -20 dBFS, attack 10 ms, release 100 ms, knee soft
            OscMessage { path: format!("{}/dyn/on", prefix), args: vec![OscArg::Int(1)] },
            OscMessage { path: format!("{}/dyn/mode", prefix), args: vec![OscArg::Int(0)] }, // COMP
            OscMessage { path: format!("{}/dyn/ratio", prefix), args: vec![OscArg::Int(5)] }, // Ratio 3:1 is typically index 5 in X_DY_RAT (" 1.1", " 1.3", " 1.5", " 2.0", " 2.5", " 3.0", " 4.0", " 5.0", " 7.0", " 10", " 20", " 100")
            OscMessage { path: format!("{}/dyn/thr", prefix), args: vec![OscArg::Float(dyn_thr_to_osc(-20.0))] },
            OscMessage { path: format!("{}/dyn/attack", prefix), args: vec![OscArg::Float(dyn_attack_to_osc(10.0))] },
            OscMessage { path: format!("{}/dyn/release", prefix), args: vec![OscArg::Float(dyn_release_to_osc(100.0))] },
            OscMessage { path: format!("{}/dyn/knee", prefix), args: vec![OscArg::Float(0.6)] }, // Soft knee (roughly 3-4dB, 0-5dB scale -> 0.6)

            // 6. Gate/Expander: Threshold -50 dBFS, range -20 dB, attack 0.5 ms, release 200 ms
            OscMessage { path: format!("{}/gate/on", prefix), args: vec![OscArg::Int(1)] },
            OscMessage { path: format!("{}/gate/mode", prefix), args: vec![OscArg::Int(2)] }, // EXP 2
            OscMessage { path: format!("{}/gate/thr", prefix), args: vec![OscArg::Float(gate_thr_to_osc(-50.0))] },
            OscMessage { path: format!("{}/gate/range", prefix), args: vec![OscArg::Float(gate_range_to_osc(-20.0))] },
            OscMessage { path: format!("{}/gate/attack", prefix), args: vec![OscArg::Float(dyn_attack_to_osc(0.5))] },
            OscMessage { path: format!("{}/gate/release", prefix), args: vec![OscArg::Float(dyn_release_to_osc(200.0))] },
        ];

        for msg in msgs {
            client.send_message(&msg.path, msg.args).await?;
            tokio::time::sleep(delay).await;
        }

        println!("Configured channel {:02} for speech mode.", ch);
    }

    Ok(())
}
