//! `x32_jog4xlive`
//!
//! This application brings two rotary knobs to the X-Live! expansion board on X32.
//! It replaces knobs 1 and 3 in Bank C of User Assign section.
//! Knob 1 acts as an audio jog, enabling moving up and down in a song.
//! Knob 3 sets the difference in time between two consecutive increments of knob 1.
//!
//! # Credits
//! * Original concept and work on the C library: Patrick-Gilles Maillot
//! * Additional concepts by: mcelb1200
//! * Rust implementation by: mcelb1200

use anyhow::Result;
use clap::Parser;


use osc_lib::OscArg;
use x32_lib::MixerClient;

/// Command-line arguments for the `x32_jog4xlive` tool.
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Jog for X32 X-Live!",
    long_about = "This application brings two rotary knobs to the X-Live! expansion board on X32. \
                  It replaces knobs 1 and 3 in Bank C of User Assign section. \
                  Knob 1 acts as an audio jog, enabling moving up and down in a song. \
                  Knob 3 sets the difference in time between two consecutive increments of knob 1."
)]
pub struct Args {
    /// IP address of the X32 console.
    #[arg(short, long, default_value = "192.168.0.64")]
    pub ip: String,

    /// Verbose output
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,
}

pub async fn run(args: Args) -> Result<()> {
    if args.verbose {
        println!("Connecting to X32 at {}", args.ip);
    }

    let addr = format!("{}:10023", args.ip);
    let client = MixerClient::connect(&addr, true).await?;
    let mut rx = client.subscribe();

    if args.verbose {
        println!("Connected to X32. Initializing User Assign section bank C encoders...");
    }

    // Initialize User Assign section bank C encoders 1 and 3
    // Set X32 Bank C Encoder 1 to its default value: 64
    client.send_message("/config/userctrl/C/enc/1", vec![OscArg::String("MP13000".to_string())]).await?;
    client.send_message("/-stat/userpar/33/value", vec![OscArg::Int(64)]).await?;

    // Set X32 Bank C Encoder 3 to its default value: 0
    client.send_message("/config/userctrl/C/enc/3", vec![OscArg::String("MP14000".to_string())]).await?;
    client.send_message("/-stat/userpar/35/value", vec![OscArg::Int(0)]).await?;

    // Select X32 Bank C
    client.send_message("/-stat/userbank", vec![OscArg::Int(2)]).await?;

    if args.verbose {
        println!("Initialization complete.");
    }

    let mut delta_time: i32 = 10; // delta_time: [10..161300] ms

    // Setup polling logic
    loop {
        match rx.recv().await {
            Ok(msg) => {
                if msg.path == "/-stat/userpar/33/value" {
                    if let Some(OscArg::Int(move_val)) = msg.args.first() {
                        handle_jog_move(&client, *move_val, delta_time).await?;
                    }
                } else if msg.path == "/-stat/userpar/35/value" {
                    if let Some(OscArg::Int(move_val)) = msg.args.first() {
                        delta_time = calculate_delta_time(*move_val);
                        if args.verbose {
                            let tensofms = delta_time / 10;
                            let minutes = tensofms / 6000;
                            let remaining = tensofms % 6000;
                            let seconds = remaining / 100;
                            let tenths = remaining % 100;
                            println!("Time between tics: {:02}m{:02}s{:02}", minutes, seconds, tenths);
                        }
                    }
                }
            }
            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
            Err(_) => break Ok(()),
        }
    }
}

pub fn calculate_delta_time(move_val: i32) -> i32 {
    let delta = move_val * move_val + 1;
    delta * 10
}

async fn handle_jog_move(client: &MixerClient, move_val: i32, delta_time: i32) -> Result<()> {
    if let Ok(OscArg::Int(state)) = client.query_value("/-stat/urec/state").await {
        if (state & 3) != 0 {
            // In play (2) or pause (1)
            if let Ok(OscArg::Int(etime)) = client.query_value("/-stat/urec/etime").await {
                let mut new_etime = etime;
                if move_val > 64 {
                    new_etime += delta_time;
                } else {
                    new_etime -= delta_time;
                }
                new_etime += 1;

                client.send_message("/-action/setposition", vec![OscArg::Int(new_etime)]).await?;
                client.send_message("/-stat/userpar/33/value", vec![OscArg::Int(64)]).await?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_delta_time() {
        assert_eq!(calculate_delta_time(0), 10);
        assert_eq!(calculate_delta_time(1), 20);
        assert_eq!(calculate_delta_time(2), 50);
        assert_eq!(calculate_delta_time(64), 40970);
        assert_eq!(calculate_delta_time(127), 161300);
    }
}
