//! Scene checker tests
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;
use x32_core::Mixer;
use x32_lib::MixerClient;
use x32_lib::transport::udp::UdpTransport;
use osc_lib::OscArg;
use std::io::Write;
use tempfile::NamedTempFile;

use x32_scene_checker::{Args, run, classify_risk, RiskLevel};

#[test]
fn test_classify_risk() {
    let path = "/routing/in/1";
    let current = OscArg::Int(0);
    let scene = OscArg::Int(1);

    let issue = classify_risk(path, &current, &scene).unwrap();
    assert_eq!(issue.level, RiskLevel::Critical);
    assert_eq!(issue.path, "/routing/in/1");

    let path = "/ch/01/mix/fader";
    let current = OscArg::Float(0.5);
    let scene = OscArg::Float(0.8);
    let issue = classify_risk(path, &current, &scene).unwrap();
    assert_eq!(issue.level, RiskLevel::Moderate);

    let path = "/ch/01/config/name";
    let current = OscArg::String("Vox 1".to_string());
    let scene = OscArg::String("Lead".to_string());
    let issue = classify_risk(path, &current, &scene).unwrap();
    assert_eq!(issue.level, RiskLevel::Info);
}

#[tokio::test]
async fn test_x32_scene_checker_integration() {
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let port = socket.local_addr().unwrap().port();
    let addr = format!("127.0.0.1:{}", port);

    let mut mixer = Mixer::new();
    let socket_arc = Arc::new(socket);
    let socket_rx = socket_arc.clone();

    let _ = tokio::spawn(async move {
        let mut buf = [0u8; 1024];
        while let Ok((len, src)) = socket_rx.recv_from(&mut buf).await {
            let responses_opt = mixer.dispatch(&buf[..len], src).ok();
            if let Some(responses) = responses_opt {
                for (addr, response_bytes) in responses {
                    let _ = socket_rx.send_to(&response_bytes, addr).await;
                }
            }
        }
    });

    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "/ch/01/mix/on 1").unwrap();
    let file_path = temp_file.path().to_str().unwrap().to_string();

    let args = Args {
        ip: addr.clone(),
        scene: file_path,
        auto_load: true,
    };

    // We run the tool, which creates its own MixerClient internally.
    let _ = run(args).await;

    tokio::time::sleep(Duration::from_millis(50)).await;

    // Verify it sent something (scene checker should send if auto_load is true)
    let transport = UdpTransport::connect(&addr).await.unwrap();
    let client = MixerClient::new(Arc::new(transport), true);

    if let Ok(OscArg::Int(val)) = client.query_value("/ch/01/mix/on").await {
        assert_eq!(val, 1);
    } else {
        panic!("Failed to query mute state");
    }
}
