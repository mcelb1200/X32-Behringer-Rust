use super::BeatDetector;
use std::collections::VecDeque;

/// Beat detector using Energy Moving Average.
pub struct EnergyDetector {
    history: VecDeque<f32>,
    history_size: usize,
    last_bpm: Option<f32>,
    beat_detected: bool,

    last_beat_time: u64,
    beat_intervals: VecDeque<u64>,
    total_samples_processed: u64,
}

impl EnergyDetector {
    pub fn new(history_seconds: f32, _sample_rate: u32) -> Self {
        let history_size = 43 * history_seconds as usize;

        Self {
            history: VecDeque::with_capacity(history_size),
            history_size,
            last_bpm: None,
            beat_detected: false,
            last_beat_time: 0,
            beat_intervals: VecDeque::with_capacity(10),
            total_samples_processed: 0,
        }
    }

    fn calculate_energy(samples: &[f32]) -> f32 {
        let sum_squares: f32 = samples.iter().map(|&s| s * s).sum();
        if samples.is_empty() {
            0.0
        } else {
            sum_squares / samples.len() as f32
        }
    }
}

impl BeatDetector for EnergyDetector {
    fn process(&mut self, samples: &[f32], sample_rate: u32) {
        self.beat_detected = false;

        let instant_energy = Self::calculate_energy(samples);

        let local_average: f32 = if self.history.is_empty() {
            0.0
        } else {
            self.history.iter().sum::<f32>() / self.history.len() as f32
        };

        if self.history.len() >= self.history_size {
            self.history.pop_front();
        }
        self.history.push_back(instant_energy);

        let c = 1.4;

        if instant_energy > c * local_average && instant_energy > 0.001 {
            let min_interval = (sample_rate as f32 * 0.1) as u64;
            if self.total_samples_processed > self.last_beat_time + min_interval {
                self.beat_detected = true;

                let interval = self.total_samples_processed - self.last_beat_time;
                self.last_beat_time = self.total_samples_processed;

                if interval > 0 && interval < (sample_rate * 2) as u64 {
                    self.beat_intervals.push_back(interval);
                    if self.beat_intervals.len() > 8 {
                        self.beat_intervals.pop_front();
                    }

                    let avg_interval: u64 =
                        self.beat_intervals.iter().sum::<u64>() / self.beat_intervals.len() as u64;
                    let bpm = 60.0 * sample_rate as f32 / avg_interval as f32;

                    self.last_bpm = Some(match self.last_bpm {
                        Some(old) => old * 0.3 + bpm * 0.7,
                        None => bpm,
                    });
                }
            }
        }

        self.total_samples_processed += samples.len() as u64;
    }

    fn process_level(&mut self, _level: f32, _timestamp_ms: u64) {
        // Not used for Audio Detector
    }

    fn current_bpm(&self) -> Option<f32> {
        self.last_bpm
    }

    fn is_beat(&self) -> bool {
        self.beat_detected
    }
}
