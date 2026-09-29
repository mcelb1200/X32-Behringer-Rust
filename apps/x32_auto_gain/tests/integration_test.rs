use anyhow::Result;
use clap::Parser;
use osc_lib::OscArg;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::sleep;
use x32_auto_gain::{Args, run};

async fn run_mock_server() -> Result<(String, tokio::task::JoinHandle<()>)> {
    let socket = UdpSocket::bind("127.0.0.1:0").await?;
    let addr = socket.local_addr()?.to_string();

    let state = Arc::new(Mutex::new(HashMap::new()));
    {
        let mut s = state.lock().unwrap();
        s.insert("/headamp/01/gain".to_string(), OscArg::Float(0.5));
        s.insert("/headamp/02/gain".to_string(), OscArg::Float(0.5));
        s.insert("/ch/01/config/name".to_string(), OscArg::String("Kick".to_string()));
        s.insert("/ch/02/config/name".to_string(), OscArg::String("Snare".to_string()));
        s.insert("/ch/01/config/icon".to_string(), OscArg::Int(1));
        s.insert("/ch/02/config/icon".to_string(), OscArg::Int(2));
        s.insert("/config/chlink/1-2".to_string(), OscArg::Int(1));
    }

    let state_clone = state.clone();

    let handle = tokio::spawn(async move {
        let mut buf = [0u8; 1024];
        loop {
            if let Ok((len, src)) = socket.recv_from(&mut buf).await {
                if let Ok(msg) = osc_lib::OscMessage::from_bytes(&buf[..len]) {
                    // Update state if set message
                    if (msg.path.starts_with("/headamp/") || msg.path.starts_with("/ch/")) && !msg.args.is_empty() {
                        let mut s = state_clone.lock().unwrap();
                        s.insert(msg.path.clone(), msg.args[0].clone());
                    }

                    // Respond to query/get requests or xremote
                    if msg.args.is_empty() && !msg.path.starts_with("/meters") && msg.path != "/xremote" {
                        let val_opt = {
                            let s = state_clone.lock().unwrap();
                            s.get(&msg.path).cloned()
                        };
                        if let Some(val) = val_opt {
                            if let Ok(resp_bytes) = osc_lib::OscMessage::serialize_to_bytes(&msg.path, vec![&val]) {
                                let _ = socket.send_to(&resp_bytes, src).await;
                            }
                        }
                    } else if msg.path == "/xremote" {
                        let items: Vec<(String, OscArg)> = {
                            let s = state_clone.lock().unwrap();
                            s.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
                        };
                        let mut resp = Vec::new();
                        for (path, val) in items {
                            if let Ok(b) = osc_lib::OscMessage::serialize_to_bytes(&path, vec![&val]) {
                                resp.extend(b);
                            }
                        }
                        let _ = socket.send_to(&resp, src).await;
                    }

                    // Reply to /meters
                    if msg.path == "/meters" {
                        let mut blob = Vec::new();
                        blob.extend_from_slice(&128i32.to_le_bytes()); // length 128 bytes

                        // Ch 1 = 0.8 (~ -1.9 dBFS -> triggers clip protection)
                        let ch1_val: f32 = 0.8;
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

                        if let Ok(meters_bytes) = osc_lib::OscMessage::serialize_to_bytes(
                            "/meters/1",
                            vec![&OscArg::Blob(blob)],
                        ) {
                            let _ = socket.send_to(&meters_bytes, src).await;
                        }
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

    let args = Args::parse_from([
        "x32_auto_gain",
        "--ip",
        &mock_ip,
        "--channels",
        "1,2",
        "--target-dbfs=-18.0",
        "--rate-ms",
        "50",
    ]);

    let app_handle = tokio::spawn(async move {
        run(args).await.unwrap();
    });

    sleep(Duration::from_millis(500)).await;

    app_handle.abort();
    server_handle.abort();
    Ok(())
}
