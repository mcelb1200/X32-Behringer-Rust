use anyhow::Result;
use clap::Parser;
use std::time::Duration;
use tokio::time::sleep;
use x32_auto_gain::{Args, run};
use osc_lib::OscArg;

// A simple local mock
use tokio::net::UdpSocket;

async fn run_mock_server() -> Result<(String, tokio::task::JoinHandle<()>)> {
    let socket = UdpSocket::bind("127.0.0.1:0").await?;
    let addr = socket.local_addr()?.to_string();

    let mut state = std::collections::HashMap::new();
    state.insert("/headamp/01/gain".to_string(), OscArg::Float(0.5));
    state.insert("/headamp/02/gain".to_string(), OscArg::Float(0.5));

    let handle = tokio::spawn(async move {
        let mut buf = [0u8; 1024];
        loop {
            if let Ok((len, src)) = socket.recv_from(&mut buf).await {
                if let Ok(msg) = osc_lib::OscMessage::from_bytes(&buf[..len]) {
                    // Update state if it's a set message
                    if msg.path.starts_with("/headamp/") && !msg.args.is_empty() {
                        state.insert(msg.path.clone(), msg.args[0].clone());
                    }

                    // Respond to xremote to keep the client alive
                    if msg.path == "/xremote" {
                        // send initial states
                        let mut resp = Vec::new();
                        let state1 = state.get("/headamp/01/gain").unwrap();
                        resp.extend(osc_lib::OscMessage::serialize_to_bytes("/headamp/01/gain", vec![state1]).unwrap());
                        let state2 = state.get("/headamp/02/gain").unwrap();
                        resp.extend(osc_lib::OscMessage::serialize_to_bytes("/headamp/02/gain", vec![state2]).unwrap());
                        let _ = socket.send_to(&resp, src).await;
                    }

                    // Reply to /meters
                    if msg.path == "/meters" {
                        let mut blob = Vec::new();
                        blob.extend_from_slice(&128i32.to_le_bytes()); // length 128 bytes

                        let ch1_val: f32 = 0.5;
                        let ch2_val: f32 = 0.01;

                        for i in 0..32 {
                            let val = if i == 0 {
                                ch1_val
                            } else if i == 1 {
                                ch2_val
                            } else {
                                0.0
                            };
                            blob.extend_from_slice(&val.to_le_bytes());
                        }

                        let meters_msg_bytes = osc_lib::OscMessage::serialize_to_bytes("/meters/1", vec![&OscArg::Blob(blob)]).unwrap();
                        let _ = socket.send_to(&meters_msg_bytes, src).await;
                    }
                }
            }
        }
    });

    Ok((addr, handle))
}


#[tokio::test]
async fn test_auto_gain_adjusts_levels() -> Result<()> {
    let (mock_ip, server_handle) = run_mock_server().await?;

    let mock_ip_clone = mock_ip.clone();

    // Spawn the app in background
    let args = Args::parse_from(&[
        "x32_auto_gain",
        "--ip",
        &mock_ip_clone,
        "--channels",
        "1,2",

        "--target-dbfs=-18.0",
        "--rate-ms",
        "50",
    ]);

    let app_handle = tokio::spawn(async move {
        run(args).await.unwrap();
    });

    // Wait for connection and initial fetch, then some meter polls
    sleep(Duration::from_millis(500)).await;

    app_handle.abort();
    server_handle.abort();
    Ok(())
}
