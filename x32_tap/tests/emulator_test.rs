use std::process::Command;
use std::time::Duration;
use osc_lib::{OscMessage, OscArg};

#[tokio::test]
async fn test_auto_mode_mock_server() {
    let mock_server = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let local_addr = mock_server.local_addr().unwrap();
    let port = local_addr.port();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_x32_tap"));
    cmd.args(&[
        "--ip", &format!("127.0.0.1:{}", port),
        "--slot", "1",
        "--auto",
        "--threshold", "0.5"
    ]);
    let mut child = cmd.spawn().expect("Failed to spawn x32_tap");

    let mut buf = [0u8; 1024];

    // 1. Respond to /info
    if let Ok((len, src)) = mock_server.recv_from(&mut buf).await {
        if let Ok(msg) = OscMessage::from_bytes(&buf[..len]) {
            if msg.path == "/info" {
                mock_server.send_to(&msg.to_bytes().unwrap(), src).await.unwrap();
            }
        }
    }

    // 2. Respond to /fx/1/type (Type 10 = Standard Delay)
    if let Ok((len, src)) = mock_server.recv_from(&mut buf).await {
        if let Ok(msg) = OscMessage::from_bytes(&buf[..len]) {
            if msg.path == "/fx/1/type" {
                let resp = OscMessage::new("/fx/1/type".to_string(), vec![OscArg::Int(10)]);
                mock_server.send_to(&resp.to_bytes().unwrap(), src).await.unwrap();
            }
        }
    }

    // 3. Receive /xremote and /meters
    let mut src_addr = None;
    for _ in 0..5 {
        if let Ok((len, src)) = tokio::time::timeout(Duration::from_secs(2), mock_server.recv_from(&mut buf)).await.unwrap() {
            if let Ok(msg) = OscMessage::from_bytes(&buf[..len]) {
                if msg.path == "/meters" {
                    src_addr = Some(src);
                    break;
                }
            }
        }
    }

    let src = src_addr.expect("Never received /meters subscription");

    // 4. Send back two mock /meters/6 blobs indicating high level, separated by >60ms
    let mut blob = vec![0u8; 16];
    let level_f32 = 0.8f32;
    blob[12..16].copy_from_slice(&level_f32.to_le_bytes());

    let meter_msg = OscMessage::new("/meters/6".to_string(), vec![OscArg::Blob(blob.clone())]);
    mock_server.send_to(&meter_msg.to_bytes().unwrap(), src).await.unwrap();

    // Sleep 100ms so delta > 60ms
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Send the second tap
    mock_server.send_to(&meter_msg.to_bytes().unwrap(), src).await.unwrap();

    // 5. Expect a set command to /fx/1/par/02
    let mut got_update = false;
    for _ in 0..5 {
        if let Ok(Ok((len, _))) = tokio::time::timeout(Duration::from_secs(2), mock_server.recv_from(&mut buf)).await {
            if let Ok(msg) = OscMessage::from_bytes(&buf[..len]) {
                if msg.path == "/fx/1/par/02" {
                    got_update = true;
                    break;
                }
            }
        } else {
            break;
        }
    }

    child.kill().unwrap();
    assert!(got_update, "Did not receive update to /fx/1/par/02 upon high meter value");
}
