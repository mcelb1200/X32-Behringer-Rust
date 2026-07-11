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
