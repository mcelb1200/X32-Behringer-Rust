//! `x32_jog4xlive`
//!
//! This application brings two rotary knobs to the X-Live! expansion board on X32.
//! It replaces knobs 1 and 3 in Bank C of User Assign section.
//! Knob 1 acts as an audio jog, enabling moving up and down in a song.
//! Knob 3 sets the difference in time between two consecutive increments of knob 1.
//!
//! # Credits
//! * Original concept and work on the C library: Patrick-Gilles Maillot
//! * Additional concepts by: [User]
//! * Rust implementation by: [User]

use anyhow::{Context, Result};
use clap::Parser;
use osc_lib::OscMessage;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::{interval, timeout};

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
struct Args {
    /// IP address of the X32 console.
    #[arg(short, long, default_value = "192.168.0.64")]
    ip: String,

    /// Verbose output
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    if args.verbose {
        println!("Connecting to X32 at {}", args.ip);
    }

    let local_addr: SocketAddr = "0.0.0.0:0".parse()?;
    let remote_addr: SocketAddr = format!("{}:10023", args.ip).parse()?;

    let socket = UdpSocket::bind(local_addr)
        .await
        .context("Failed to bind UDP socket")?;
    socket
        .connect(remote_addr)
        .await
        .context("Failed to connect to X32")?;

    if args.verbose {
        println!("Connected to X32. Initializing User Assign section bank C encoders...");
    }

    // Initialize User Assign section bank C encoders 1 and 3
    // Set X32 Bank C Encoder 1 to its default value: 64
    send_osc(&socket, "/config/userctrl/C/enc/1", "MP13000").await?;
    send_osc_int(&socket, "/-stat/userpar/33/value", 64).await?;

    // Set X32 Bank C Encoder 3 to its default value: 0
    send_osc(&socket, "/config/userctrl/C/enc/3", "MP14000").await?;
    send_osc_int(&socket, "/-stat/userpar/35/value", 0).await?;

    // Select X32 Bank C
    send_osc_int(&socket, "/-stat/userbank", 2).await?;

    if args.verbose {
        println!("Initialization complete.");
    }

    let mut delta_time: i32 = 10; // delta_time: [10..161300] ms

    // Setup polling logic
    let mut xremote_interval = interval(Duration::from_secs(9));
    let mut buf = vec![0u8; 1024];

    loop {
        tokio::select! {
            _ = xremote_interval.tick() => {
                let msg = osc_lib::OscMessage::new("/xremote".to_string(), vec![]);
                if let Ok(bytes) = msg.to_bytes() {
                    let _ = socket.send(&bytes).await;
                }
            }
            res = socket.recv(&mut buf) => {
                match res {
                    Ok(len) => {
                        let bytes = &buf[..len];
                        if let Ok(msg) = OscMessage::from_bytes(bytes) {
                            if msg.path == "/-stat/userpar/33/value" {
                                if let Some(osc_lib::OscArg::Int(move_val)) = msg.args.get(0) {
                                    handle_jog_move(&socket, *move_val, delta_time).await?;
                                }
                            } else if msg.path == "/-stat/userpar/35/value" {
                                if let Some(osc_lib::OscArg::Int(move_val)) = msg.args.get(0) {
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
                    }
                    Err(_) => {
                        // Ignore recv errors
                    }
                }
            }
        }
    }
}

pub fn calculate_delta_time(move_val: i32) -> i32 {
    let delta = move_val * move_val + 1;
    delta * 10
}

async fn handle_jog_move(socket: &UdpSocket, move_val: i32, delta_time: i32) -> Result<()> {
    // get X-Live! transport status
    query_osc(socket, "/-stat/urec/state").await?;

    let mut buf = vec![0u8; 1024];
    let start = std::time::Instant::now();
    let timeout_duration = Duration::from_millis(500);

    // We expect state back
    while start.elapsed() < timeout_duration {
        if let Ok(Ok(len)) = timeout(Duration::from_millis(50), socket.recv(&mut buf)).await {
            let bytes = &buf[..len];
            if let Ok(msg) = OscMessage::from_bytes(bytes) {
                if msg.path == "/-stat/urec/state" {
                    if let Some(osc_lib::OscArg::Int(state)) = msg.args.get(0) {
                        if (*state & 3) != 0 {
                            // In play (2) or pause (1)
                            query_osc(socket, "/-stat/urec/etime").await?;

                            let start2 = std::time::Instant::now();
                            while start2.elapsed() < timeout_duration {
                                if let Ok(Ok(len2)) = timeout(Duration::from_millis(50), socket.recv(&mut buf)).await {
                                    let bytes2 = &buf[..len2];
                                    if let Ok(msg2) = OscMessage::from_bytes(bytes2) {
                                        if msg2.path == "/-stat/urec/etime" {
                                            if let Some(osc_lib::OscArg::Int(etime)) = msg2.args.get(0) {
                                                let mut new_etime = *etime;
                                                if move_val > 64 {
                                                    new_etime += delta_time;
                                                } else {
                                                    new_etime -= delta_time;
                                                }
                                                new_etime += 1;

                                                // Set new position
                                                send_osc_int(socket, "/-action/setposition", new_etime).await?;
                                                // Reset rotary cursor
                                                send_osc_int(socket, "/-stat/userpar/33/value", 64).await?;
                                                return Ok(());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    return Ok(()); // Handled state response but not playing
                }
            }
        }
    }

    Ok(())
}

async fn send_osc(socket: &UdpSocket, path: &str, s: &str) -> Result<()> {
    let msg = osc_lib::OscMessage::new(path.to_string(), vec![osc_lib::OscArg::String(s.to_string())]);
    let buf = msg.to_bytes().map_err(|e| anyhow::anyhow!(e))?;
    socket.send(&buf).await?;
    Ok(())
}

async fn query_osc(socket: &UdpSocket, path: &str) -> Result<()> {
    let msg = osc_lib::OscMessage::new(path.to_string(), vec![]);
    let buf = msg.to_bytes().map_err(|e| anyhow::anyhow!(e))?;
    socket.send(&buf).await?;
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

async fn send_osc_int(socket: &UdpSocket, path: &str, i: i32) -> Result<()> {
    let msg = osc_lib::OscMessage::new(path.to_string(), vec![osc_lib::OscArg::Int(i)]);
    let buf = msg.to_bytes().map_err(|e| anyhow::anyhow!(e))?;
    socket.send(&buf).await?;
    Ok(())
}
