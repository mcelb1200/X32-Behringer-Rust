//! Scene checker tests
use osc_lib::OscArg;
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;
use tempfile::NamedTempFile;
use tokio::net::UdpSocket;
use x32_core::Mixer;
use x32_lib::transport::udp::UdpTransport;
use x32_lib::{MixerClient, MixerModel};

use x32_scene_checker::{Args, RiskLevel, classify_risk, classify_risk_with_model, run};

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

#[test]
fn test_classify_risk_models() {
    let models = [
        MixerModel::X32,
        MixerModel::Wing,
        MixerModel::XR18,
        MixerModel::XR16,
        MixerModel::XR12,
    ];

    for model in models {
        let issue =
            classify_risk_with_model(model, "/main/st/mix/on", &OscArg::Int(1), &OscArg::Int(0))
                .unwrap();
        assert_eq!(issue.level, RiskLevel::Critical);

        let issue_gain = classify_risk_with_model(
            model,
            "/ch/01/preamp/trim",
            &OscArg::Float(0.0),
            &OscArg::Float(0.5),
        )
        .unwrap();
        assert_eq!(issue_gain.level, RiskLevel::High);
    }
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
        model: MixerModel::X32,
        auto_load: true,
        locked_paths: None,
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

#[tokio::test]
async fn test_x32_scene_checker_locked_paths() {
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
    writeln!(temp_file, "/ch/02/mix/on 1").unwrap();
    writeln!(temp_file, "/routing/in/1 3").unwrap();
    let file_path = temp_file.path().to_str().unwrap().to_string();

    let args = Args {
        ip: addr.clone(),
        scene: file_path,
        model: MixerModel::X32,
        auto_load: true,
        locked_paths: Some("/routing,/ch/02".to_string()),
    };

    let _ = run(args).await;

    tokio::time::sleep(Duration::from_millis(50)).await;

    let transport = UdpTransport::connect(&addr).await.unwrap();
    let client = MixerClient::new(Arc::new(transport), true);

    // /ch/01/mix/on should be set to 1
    if let Ok(OscArg::Int(val)) = client.query_value("/ch/01/mix/on").await {
        assert_eq!(val, 1);
    } else {
        panic!("Failed to query /ch/01/mix/on");
    }

    // /ch/02/mix/on should not be set (so the query should return an error or None)
    let res = client.query_value("/ch/02/mix/on").await;
    assert!(
        res.is_err(),
        "Expected query to fail because path was not set due to lock"
    );

    // /routing/in/1 should not be set
    let res = client.query_value("/routing/in/1").await;
    assert!(
        res.is_err(),
        "Expected query to fail because path was not set due to lock"
    );
}

#[tokio::test]
async fn test_x32_scene_checker_xr18_model_integration() {
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
    writeln!(temp_file, "/ch/16/mix/on 1").unwrap();
    // /ch/32 is invalid on XR18 (only 16 channels)
    writeln!(temp_file, "/ch/32/mix/on 1").unwrap();
    let file_path = temp_file.path().to_str().unwrap().to_string();

    let args = Args {
        ip: addr.clone(),
        scene: file_path,
        model: MixerModel::XR18,
        auto_load: true,
        locked_paths: None,
    };

    let _ = run(args).await;

    tokio::time::sleep(Duration::from_millis(50)).await;

    let transport = UdpTransport::connect(&addr).await.unwrap();
    let client = MixerClient::new(Arc::new(transport), true);

    // /ch/16/mix/on should be set to 1
    if let Ok(OscArg::Int(val)) = client.query_value("/ch/16/mix/on").await {
        assert_eq!(val, 1);
    } else {
        panic!("Failed to query /ch/16/mix/on");
    }

    // /ch/32/mix/on should not be set because XR18 parser ignored it
    let res = client.query_value("/ch/32/mix/on").await;
    assert!(
        res.is_err(),
        "Expected query to fail because path was invalid for XR18 model"
    );
}

#[tokio::test]
async fn test_x32_scene_checker_wing_model_integration() {
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
    writeln!(temp_file, "/ch/40/mix/on 1").unwrap();
    let file_path = temp_file.path().to_str().unwrap().to_string();

    let args = Args {
        ip: addr.clone(),
        scene: file_path,
        model: MixerModel::Wing,
        auto_load: true,
        locked_paths: None,
    };

    let _ = run(args).await;

    tokio::time::sleep(Duration::from_millis(50)).await;

    let transport = UdpTransport::connect(&addr).await.unwrap();
    let client = MixerClient::new(Arc::new(transport), true);

    // /ch/40/mix/on should be set to 1
    if let Ok(OscArg::Int(val)) = client.query_value("/ch/40/mix/on").await {
        assert_eq!(val, 1);
    } else {
        panic!("Failed to query /ch/40/mix/on");
    }
}
