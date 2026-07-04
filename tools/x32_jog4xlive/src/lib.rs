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
use std::time::Duration;
use tokio::time::timeout;
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

    let client = MixerClient::connect(&args.ip, true).await?;

    if args.verbose {
        println!("Connected to X32. Initializing User Assign section bank C encoders...");
    }

    // Initialize User Assign section bank C encoders 1 and 3
    // Set X32 Bank C Encoder 1 to its default value: 64
    client
        .send_message(
            "/config/userctrl/C/enc/1",
            vec![OscArg::String("MP13000".to_string())],
        )
        .await?;
    client
        .send_message("/-stat/userpar/33/value", vec![OscArg::Int(64)])
        .await?;

    // Set X32 Bank C Encoder 3 to its default value: 0
    client
        .send_message(
            "/config/userctrl/C/enc/3",
            vec![OscArg::String("MP14000".to_string())],
        )
        .await?;
    client
        .send_message("/-stat/userpar/35/value", vec![OscArg::Int(0)])
        .await?;

    // Select X32 Bank C
    client
        .send_message("/-stat/userbank", vec![OscArg::Int(2)])
        .await?;

    if args.verbose {
        println!("Initialization complete.");
    }

    let mut delta_time: i32 = 10; // delta_time: [10..161300] ms

    // Setup polling logic
    let mut rx = client.subscribe();

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
                            println!(
                                "Time between tics: {:02}m{:02}s{:02}",
                                minutes, seconds, tenths
                            );
                        }
                    }
                }
            }
            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
            Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
        }
    }

    Ok(())
}

pub fn calculate_delta_time(move_val: i32) -> i32 {
    let delta = move_val * move_val + 1;
    delta * 10
}

async fn handle_jog_move(client: &MixerClient, move_val: i32, delta_time: i32) -> Result<()> {
    let mut rx = client.subscribe();

    // get X-Live! transport status
    client.send_message("/-stat/urec/state", vec![]).await?;

    let start = std::time::Instant::now();
    let timeout_duration = Duration::from_millis(500);

    // We expect state back
    while start.elapsed() < timeout_duration {
        if let Ok(Ok(msg)) = timeout(Duration::from_millis(50), rx.recv()).await {
            if msg.path == "/-stat/urec/state" {
                if let Some(OscArg::Int(state)) = msg.args.first() {
                    if (*state & 3) != 0 {
                        // In play (2) or pause (1)
                        client.send_message("/-stat/urec/etime", vec![]).await?;

                        let start2 = std::time::Instant::now();
                        while start2.elapsed() < timeout_duration {
                            if let Ok(Ok(msg2)) =
                                timeout(Duration::from_millis(50), rx.recv()).await
                            {
                                if msg2.path == "/-stat/urec/etime" {
                                    if let Some(OscArg::Int(etime)) = msg2.args.first() {
                                        let mut new_etime = *etime;
                                        if move_val > 64 {
                                            new_etime += delta_time;
                                        } else {
                                            new_etime -= delta_time;
                                        }
                                        new_etime += 1;

                                        // Set new position
                                        client
                                            .send_message(
                                                "/-action/setposition",
                                                vec![OscArg::Int(new_etime)],
                                            )
                                            .await?;

                                        // Reset rotary cursor
                                        client
                                            .send_message(
                                                "/-stat/userpar/33/value",
                                                vec![OscArg::Int(64)],
                                            )
                                            .await?;

                                        return Ok(());
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

#[tokio::test]
async fn test_handle_jog_move() {
    let port = 44444;
    let addr = format!("127.0.0.1:{}", port);

    // Setup mock server
    let mock_server = tokio::net::UdpSocket::bind(&addr).await.unwrap();

    let client = MixerClient::connect(&addr, false).await.unwrap();
    let client = std::sync::Arc::new(client);

    // Spawn jog handler
    let client_clone = client.clone();
    tokio::spawn(async move {
        let _ = handle_jog_move(&client_clone, 65, 10).await;
    });

    let mut buf = vec![0u8; 1024];

    // Expect /state query
    let (len, peer) = mock_server.recv_from(&mut buf).await.unwrap();
    let query_msg = osc_lib::OscMessage::from_bytes(&buf[..len]).unwrap();
    assert_eq!(query_msg.path, "/-stat/urec/state");

    // Reply with play state
    let reply_msg = osc_lib::OscMessage::new(
        "/-stat/urec/state".to_string(),
        vec![osc_lib::OscArg::Int(2)],
    );
    mock_server
        .send_to(&reply_msg.to_bytes().unwrap(), peer)
        .await
        .unwrap();

    // Expect /etime query
    let (len, peer) = mock_server.recv_from(&mut buf).await.unwrap();
    let query_msg2 = osc_lib::OscMessage::from_bytes(&buf[..len]).unwrap();
    assert_eq!(query_msg2.path, "/-stat/urec/etime");

    // Reply with etime
    let reply_msg2 = osc_lib::OscMessage::new(
        "/-stat/urec/etime".to_string(),
        vec![osc_lib::OscArg::Int(500)],
    );
    mock_server
        .send_to(&reply_msg2.to_bytes().unwrap(), peer)
        .await
        .unwrap();

    // Expect /action/setposition
    let (len, _) = mock_server.recv_from(&mut buf).await.unwrap();
    let set_msg = osc_lib::OscMessage::from_bytes(&buf[..len]).unwrap();
    assert_eq!(set_msg.path, "/-action/setposition");
    assert_eq!(set_msg.args[0], osc_lib::OscArg::Int(511));

    // Expect cursor reset
    let (len, _) = mock_server.recv_from(&mut buf).await.unwrap();
    let reset_msg = osc_lib::OscMessage::from_bytes(&buf[..len]).unwrap();
    assert_eq!(reset_msg.path, "/-stat/userpar/33/value");
    assert_eq!(reset_msg.args[0], osc_lib::OscArg::Int(64));
}
