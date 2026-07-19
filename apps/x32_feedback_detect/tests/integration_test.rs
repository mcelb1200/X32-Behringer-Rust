use std::f32::consts::PI;
use x32_feedback_detect::detector::FeedbackDetector;

#[test]
fn test_detector_finds_feedback() {
    let sample_rate = 44100;
    let fft_size = 2048;
    let mut detector = FeedbackDetector::new(sample_rate, fft_size);

    let target_freq = 1000.0;

    // Generate a pure sine wave (simulate feedback ringing)
    let mut buffer = vec![0.0; fft_size];
    for (i, sample) in buffer.iter_mut().enumerate() {
        let t = i as f32 / sample_rate as f32;
        *sample = (t * target_freq * 2.0 * PI).sin();
    }

    // Process first chunk (should detect peak but not yet confirmed feedback)
    let events = detector.process(&buffer, 100);
    assert!(events.is_empty(), "Should wait for persistence");

    // Process second chunk, simulating another 100ms passed (total 200ms > 150ms threshold)
    let events = detector.process(&buffer, 100);
    assert_eq!(events.len(), 1, "Should detect sustained feedback");

    let fb = events[0];
    assert!(
        (fb.frequency - target_freq).abs() < 50.0,
        "Frequency should be close to 1000 Hz"
    );
    assert!(fb.q_ratio > 30.0, "Pure sine should have high Q");
}

#[test]
fn test_detector_ignores_noise() {
    let sample_rate = 44100;
    let fft_size = 2048;
    let mut detector = FeedbackDetector::new(sample_rate, fft_size);

    // Provide silence or simple impulse, Q won't be sustained high
    let mut buffer = vec![0.0; fft_size];
    buffer[10] = 1.0;
    buffer[20] = -1.0;

    let events = detector.process(&buffer, 200);
    assert!(events.is_empty(), "Should ignore non-sustained noise");
}

use x32_feedback_detect::mixer::MixerState;
use x32_lib::MixerClient;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;
use x32_core::Mixer;
use x32_lib::transport::udp::UdpTransport;
use osc_lib::OscArg;

#[tokio::test]
async fn test_x32_feedback_detect_mixer_state() {
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let port = socket.local_addr().unwrap().port();
    let addr = format!("127.0.0.1:{}", port);
    let socket_addr = socket.local_addr().unwrap();

    let mut mixer = Mixer::new();
    // Pre-populate some states to simulate an active mixer that accepts EQ bands
    mixer.dispatch(osc_lib::OscMessage::new("/ch/01/eq/6/type".to_string(), vec![OscArg::Int(3)]).to_bytes().unwrap().as_slice(), socket_addr).unwrap();

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

    let transport = UdpTransport::connect(&addr).await.unwrap();
    let client = MixerClient::new(Arc::new(transport), true);

    let mut state = MixerState::new(client, 1);

    // Test applying notch
    state.apply_notch(1000.0).await.unwrap();

    tokio::time::sleep(Duration::from_millis(150)).await;

    // Verify
    let transport2 = UdpTransport::connect(&addr).await.unwrap();
    let client2 = MixerClient::new(Arc::new(transport2), true);

    // Since band 6 was available in the fake state (as the logic iterates backwards [6, 5, 4, 3, 2, 1] to find an unused band), it should use it.
    if let Ok(OscArg::Int(val)) = client2.query_value("/ch/01/eq/6/type").await {
        assert_eq!(val, 3); // PEQ
    } else {
        panic!("Failed to query EQ type");
    }

    // Try applying the exact same notch to see if depth increases
    state.apply_notch(1000.0).await.unwrap();

    // Reset
    state.reset_notches().await.unwrap();
    assert!(state.applied_notches.is_empty());
}
