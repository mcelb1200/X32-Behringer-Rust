use super::BeatDetector;
use std::collections::VecDeque;

/// Fallback Beat detector using OSC Level data.
///
/// Since OSC meter data is low resolution (~20-40Hz) and lacks waveform detail,
/// this uses a simple Peak tracking algorithm.
pub struct OscLevelDetector {
    last_level: f32,
    last_bpm: Option<f32>,
    beat_detected: bool,

    last_beat_time_ms: u64,
    beat_intervals_ms: VecDeque<u64>,
}

impl OscLevelDetector {
    pub fn new() -> Self {
        Self {
            last_level: 0.0,
            last_bpm: None,
            beat_detected: false,
            last_beat_time_ms: 0,
            beat_intervals_ms: VecDeque::with_capacity(10),
        }
    }
}

impl BeatDetector for OscLevelDetector {
    fn process(&mut self, _samples: &[f32], _sample_rate: u32) {
        // Not used for OSC Detector
    }

    fn process_level(&mut self, level: f32, timestamp_ms: u64) {
        self.beat_detected = false;

        // Simple rising edge detection with decay
        // If level jumps significantly compared to previous
        let threshold_jump = 0.15; // Requires tuning based on real X32 behavior

        if level > self.last_level + threshold_jump && level > 0.2 {
            // Debounce (e.g., 250ms = 240 BPM max)
            if timestamp_ms > self.last_beat_time_ms + 250 {
                self.beat_detected = true;

                let interval = timestamp_ms - self.last_beat_time_ms;
                self.last_beat_time_ms = timestamp_ms;

                if interval > 0 && interval < 2000 {
                    // 30 BPM min
                    self.beat_intervals_ms.push_back(interval);
                    if self.beat_intervals_ms.len() > 5 {
                        self.beat_intervals_ms.pop_front();
                    }

                    let avg_interval = self.beat_intervals_ms.iter().sum::<u64>()
                        / self.beat_intervals_ms.len() as u64;
                    let bpm = 60_000.0 / avg_interval as f32;

                    self.last_bpm = Some(match self.last_bpm {
                        Some(old) => old * 0.5 + bpm * 0.5,
                        None => bpm,
                    });
                }
            }
        }

        self.last_level = level;
    }

    fn current_bpm(&self) -> Option<f32> {
        self.last_bpm
    }

    fn is_beat(&self) -> bool {
        self.beat_detected
    }
}
