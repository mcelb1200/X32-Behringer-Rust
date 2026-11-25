use anyhow::Result;
use clap::Parser;
use osc_lib::{OscArg, OscMessage};
use std::time::Duration;
use tokio::net::UdpSocket;
use x32_auto_ringout::{bus_meter_index, detect_rta_feedback_peak, Args};
use x32_lib::meters::{linear_to_dbfs, meter_value_at, parse_meter_blob};
use x32_lib::MixerModel;

async fn run_mock_server() -> Result<(String, tokio::task::JoinHandle<()>)> {
    let socket = UdpSocket::bind("127.0.0.1:0").await?;
    let addr = socket.local_addr()?.to_string();

    let mut state = std::collections::HashMap::new();

    let handle = tokio::spawn(async move {
        let mut buf = [0u8; 1024];
        let mut loop_count = 0;
        loop {
            if let Ok((len, src)) = socket.recv_from(&mut buf).await {
                if let Ok(msg) = OscMessage::from_bytes(&buf[..len]) {
                    if !msg.args.is_empty() {
                        state.insert(msg.path.clone(), msg.args[0].clone());
                    }

                    if msg.path == "/xremote" {
                        let _ = socket.send_to(b"/xremote\0\0\0\0,\0\0\0", src).await;
                    }

                    if msg.path == "/meters" {
                        let mut blob = Vec::new();
                        // X32 /meters/1 layout: 32 channels + 8 auxins + 16 buses + 6 matrices + 1 mainC + 3 LRM = 66 floats
                        let num_meters: i32 = 32 + 8 + 16 + 6 + 1 + 3;
                        blob.extend_from_slice(&(num_meters * 4).to_le_bytes());

                        loop_count += 1;
                        // Emit a > -6 dBFS peak (> 0.501 linear) on Bus 1 at its real X32 offset (32 + 8 = 40)
                        let bus1_idx = bus_meter_index(MixerModel::X32, 1) as i32;
                        let val: f32 = if loop_count > 1 { 0.6 } else { 0.001 };

                        for i in 0..num_meters {
                            if i == bus1_idx {
                                blob.extend_from_slice(&val.to_le_bytes());
                            } else {
                                blob.extend_from_slice(&0.0f32.to_le_bytes());
                            }
                        }

                        let meters_msg_bytes = OscMessage::serialize_to_bytes(
                            "/meters/1",
                            vec![&OscArg::Blob(blob)],
                        )
                        .unwrap();
                        let _ = socket.send_to(&meters_msg_bytes, src).await;
                    }
                }
            }
        }
    });

    Ok((addr, handle))
}

#[tokio::test]
async fn test_auto_ringout_bus_meter_offset_and_rta_peak() -> Result<()> {
    let (mock_ip, server_handle) = run_mock_server().await?;

    // Verify CLI parses cleanly without requiring --simulate
    let args = Args::parse_from([
        "x32_auto_ringout",
        "--ip",
        &mock_ip,
        "--buses",
        "1",
        "--target-dbfs=-6.0",
        "--max-notches=1",
    ]);
    assert!(!args.simulate);
    assert_eq!(bus_meter_index(MixerModel::X32, 1), 40);
    assert_eq!(bus_meter_index(MixerModel::XR18, 1), 18);

    // Query /meters twice from mock server and verify Bus 1 peaks > -6 dBFS at offset 40
    let client_sock = UdpSocket::bind("127.0.0.1:0").await?;
    let req = OscMessage::new(
        "/meters".to_string(),
        vec![OscArg::String("/meters/1".to_string())],
    )
    .to_bytes()?;

    let mut recv_buf = [0u8; 2048];
    let mut last_bus1_db = -144.0f32;
    for _ in 0..2 {
        client_sock.send_to(&req, &mock_ip).await?;
        let (len, _) =
            tokio::time::timeout(Duration::from_secs(1), client_sock.recv_from(&mut recv_buf))
                .await??;
        let resp = OscMessage::from_bytes(&recv_buf[..len])?;
        assert_eq!(resp.path, "/meters/1");
        if let Some(OscArg::Blob(blob)) = resp.args.first() {
            let bus1_val = meter_value_at(blob, bus_meter_index(MixerModel::X32, 1))
                .expect("Bus 1 meter value present at index 40");
            last_bus1_db = linear_to_dbfs(bus1_val);
            assert_eq!(parse_meter_blob(blob).len(), 66);
        }
    }
    assert!(
        last_bus1_db > -6.0,
        "Expected Bus 1 peak > -6 dBFS, got {:.2} dBFS",
        last_bus1_db
    );

    // Verify live RTA narrow-band feedback peak detection around ~1 kHz (bin ~56 of 100)
    let mut rta_bins = vec![0.002f32; 100];
    rta_bins[56] = 0.4;
    let peak_hz = detect_rta_feedback_peak(&rta_bins, 9.0).expect("Expected RTA peak");
    assert!(
        (800.0..=1500.0).contains(&peak_hz),
        "Expected peak near 1 kHz, got {:.1} Hz",
        peak_hz
    );

    server_handle.abort();
    Ok(())
}
