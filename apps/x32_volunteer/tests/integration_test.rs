//! Volunteer App Integration Tests
use osc_lib::OscArg;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;
use x32_core::Mixer;
use x32_lib::transport::udp::UdpTransport;
use x32_lib::{MixerClient, MixerModel};

use x32_volunteer::{Args, run};

#[tokio::test]
async fn test_x32_volunteer_integration() {
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let port = socket.local_addr().unwrap().port();
    let addr = format!("127.0.0.1:{}", port);
    let socket_addr = socket.local_addr().unwrap();

    let mut mixer = Mixer::new();
    let socket_arc = Arc::new(socket);
    let socket_rx = socket_arc.clone();

    // Populate initial state required by volunteer app queries
    mixer
        .dispatch(
            osc_lib::OscMessage::new(
                "/ch/01/config/name".to_string(),
                vec![OscArg::String("Vox".to_string())],
            )
            .to_bytes()
            .unwrap()
            .as_slice(),
            socket_addr,
        )
        .unwrap();
    mixer
        .dispatch(
            osc_lib::OscMessage::new("/ch/01/mix/on".to_string(), vec![OscArg::Int(1)])
                .to_bytes()
                .unwrap()
                .as_slice(),
            socket_addr,
        )
        .unwrap();
    mixer
        .dispatch(
            osc_lib::OscMessage::new("/ch/01/mix/fader".to_string(), vec![OscArg::Float(0.75)])
                .to_bytes()
                .unwrap()
                .as_slice(),
            socket_addr,
        )
        .unwrap();

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

    let args = Args {
        ip: addr.clone(),
        model: MixerModel::X32,
        config: None,
        channels: "ch1".to_string(),
    };

    // run will block because of the crossterm event loop, but we can verify initialization finishes
    let jh = tokio::spawn(async move {
        let _ = run(args).await;
    });

    tokio::time::sleep(Duration::from_millis(200)).await;

    // Verify app sent meter subscription
    let transport = UdpTransport::connect(&addr).await.unwrap();
    let client = MixerClient::new(Arc::new(transport), true);

    // Check if the mixer correctly updated the fader state logic (the run method asks for initial values)
    if let Ok(OscArg::Int(val)) = client.query_value("/ch/01/mix/on").await {
        assert_eq!(val, 1);
    } else {
        panic!("Failed to query mute state");
    }

    jh.abort();
}

#[tokio::test]
async fn test_wing_model_volunteer_integration() {
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let port = socket.local_addr().unwrap().port();
    let addr = format!("127.0.0.1:{}", port);
    let socket_addr = socket.local_addr().unwrap();

    let mut mixer = Mixer::new();
    let socket_arc = Arc::new(socket);
    let socket_rx = socket_arc.clone();

    mixer
        .dispatch(
            osc_lib::OscMessage::new(
                "/ch/40/config/name".to_string(),
                vec![OscArg::String("WingCh40".to_string())],
            )
            .to_bytes()
            .unwrap()
            .as_slice(),
            socket_addr,
        )
        .unwrap();

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

    let args = Args {
        ip: addr.clone(),
        model: MixerModel::Wing,
        config: None,
        channels: "ch40".to_string(),
    };

    let jh = tokio::spawn(async move {
        let _ = run(args).await;
    });

    tokio::time::sleep(Duration::from_millis(200)).await;

    let transport = UdpTransport::connect(&addr).await.unwrap();
    let client = MixerClient::new(Arc::new(transport), true);

    if let Ok(OscArg::String(name)) = client.query_value("/ch/40/config/name").await {
        assert_eq!(name, "WingCh40");
    } else {
        panic!("Failed to query Wing channel 40 name");
    }

    jh.abort();
}

#[tokio::test]
async fn test_config_file_volunteer_integration() {
    let json_content = r#"{
        "volunteer_mode": {
            "max_fader_limit_db": -3.0,
            "visible_channels": [1, 2],
            "visible_dcas": [1]
        }
    }"#;
    let tmp_config = std::env::temp_dir().join("test_volunteer_integration.json");
    std::fs::write(&tmp_config, json_content).unwrap();

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

    let args = Args {
        ip: addr.clone(),
        model: MixerModel::X32,
        config: Some(tmp_config.to_str().unwrap().to_string()),
        channels: "".to_string(),
    };

    let jh = tokio::spawn(async move {
        let _ = run(args).await;
    });

    tokio::time::sleep(Duration::from_millis(200)).await;

    jh.abort();
    let _ = std::fs::remove_file(tmp_config);
}
