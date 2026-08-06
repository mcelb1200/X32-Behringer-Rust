//! Vocal Ducking Integration Tests
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;
use x32_core::Mixer;

use x32_vocal_ducking::{Cli, run};

// We will test the pure OSC fallback mode, as testing cpal requires an active audio driver.
// The `run` function holds a `cpal::Stream` which is `!Send` on ALSA, so we can't easily run it inside a `tokio::spawn` task.
// Therefore, we just test argument logic or a subset if possible. Or we don't spawn it, but await it directly with a timeout.

#[tokio::test]
async fn test_x32_vocal_ducking_integration() {
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

    let args = Cli {
        ip: addr.clone(),
        instrument_bus: 5,
        threshold: -40.0,
        ratio: 4.0,
        max_duck: 12.0,
        use_key_filter: true,
        attack: 5.0,
        release: 50.0,
        audio_device: None,
        card_channel: None,
        vocal_bus: Some(2),
        list_devices: false,
    };

    // run is !Send on ALSA. So we can't tokio::spawn it, but we can tokio::select! with a timeout to verify it starts and doesn't panic.
    let _ = tokio::time::timeout(Duration::from_millis(50), run(args)).await;
}
