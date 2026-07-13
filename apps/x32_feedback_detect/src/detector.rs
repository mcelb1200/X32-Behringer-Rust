use num_complex::Complex;
use rustfft::num_traits::Zero;
use rustfft::{Fft, FftPlanner};
use std::sync::Arc;

#[derive(Debug, Clone, Copy)]
pub struct ActiveFeedback {
    pub frequency: f32,
    pub persistence_ms: u64,
    pub q_ratio: f32,
    pub level: f32,
}

pub struct FeedbackDetector {
    fft: Arc<dyn Fft<f32>>,
    scratch: Vec<Complex<f32>>,
    sample_rate: u32,
    fft_size: usize,

    // Configurable thresholds
    q_threshold: f32,
    persistence_threshold_ms: u64,

    // Detection state
    candidates: Vec<ActiveFeedback>,
}

impl FeedbackDetector {
    pub fn new(sample_rate: u32, fft_size: usize) -> Self {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(fft_size);

        Self {
            fft,
            scratch: vec![Complex::zero(); fft_size],
            sample_rate,
            fft_size,
            q_threshold: 30.0, // High ratio peak to average
            persistence_threshold_ms: 150,
            candidates: Vec::new(),
        }
    }

    /// Process a new chunk of audio samples and return detected feedback events.
    pub fn process(&mut self, samples: &[f32], delta_ms: u64) -> Vec<ActiveFeedback> {
        if samples.len() < self.fft_size {
            return Vec::new();
        }

        let mut buffer: Vec<Complex<f32>> = samples
            .iter()
            .take(self.fft_size)
            .map(|&s| Complex::new(s, 0.0))
            .collect();

        self.fft.process_with_scratch(&mut buffer, &mut self.scratch);

        let mut magnitudes = Vec::with_capacity(self.fft_size / 2);
        for c in buffer.iter().take(self.fft_size / 2) {
            magnitudes.push(c.norm());
        }

        // Calculate average for Q estimation
        let total_mag: f32 = magnitudes.iter().sum();
        let avg_mag = total_mag / (self.fft_size / 2) as f32;

        let mut current_peaks = Vec::new();

        // Identify peaks
        if avg_mag > 0.0 {
            let mut peak_idx = 0;
            let mut peak_mag = 0.0;
            for (i, &mag) in magnitudes.iter().enumerate() {
                if mag > peak_mag {
                    peak_mag = mag;
                    peak_idx = i;
                }
            }

            let q_ratio = peak_mag / avg_mag;
            if q_ratio > self.q_threshold {
                let frequency = (peak_idx as f32 * self.sample_rate as f32) / self.fft_size as f32;
                current_peaks.push((frequency, q_ratio, peak_mag));
            }
        }

        // Update persistence
        let mut new_candidates = Vec::new();
        let mut confirmed_feedback = Vec::new();
        let tolerance = 50.0; // Hz tolerance

        for (freq, q_ratio, level) in current_peaks {
            let mut matched = false;
            for candidate in &mut self.candidates {
                if (candidate.frequency - freq).abs() < tolerance {
                    candidate.persistence_ms += delta_ms;
                    candidate.q_ratio = candidate.q_ratio.max(q_ratio); // keep max
                    candidate.level = candidate.level.max(level);
                    new_candidates.push(*candidate);
                    matched = true;

                    if candidate.persistence_ms >= self.persistence_threshold_ms {
                        confirmed_feedback.push(*candidate);
                        // Reset to avoid duplicate triggering immediately
                        candidate.persistence_ms = 0;
                    }
                    break;
                }
            }
            if !matched {
                new_candidates.push(ActiveFeedback {
                    frequency: freq,
                    persistence_ms: delta_ms,
                    q_ratio,
                    level,
                });
            }
        }

        self.candidates = new_candidates;

        confirmed_feedback
    }
}
