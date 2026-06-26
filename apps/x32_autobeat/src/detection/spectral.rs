use super::BeatDetector;
use num_complex::Complex;
use rustfft::num_traits::Zero;
use rustfft::{Fft, FftPlanner};
use std::collections::VecDeque;
use std::sync::Arc;

pub struct SpectralFluxDetector {
    fft: Arc<dyn Fft<f32>>,
    scratch: Vec<Complex<f32>>,

    prev_spectrum: Vec<f32>,
    flux_history: VecDeque<f32>,
    flux_history_size: usize,

    last_beat_time: u64,
    beat_intervals: VecDeque<u64>,
    total_samples_processed: u64,

    last_bpm: Option<f32>,
    beat_detected: bool,

    // Configuration
    sample_rate: u32,
    fft_size: usize,
    hop_size: usize,
    buffer: Vec<f32>, // Accumulate samples until we have fft_size
}

impl SpectralFluxDetector {
    pub fn new(sample_rate: u32, fft_size: usize) -> Self {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(fft_size);
        let scratch_len = fft.get_inplace_scratch_len();

        // History for thresholding (~1.5 seconds)
        let flux_history_size = 64;

        Self {
            fft,
            scratch: vec![Complex::zero(); scratch_len],
            prev_spectrum: vec![0.0; fft_size / 2 + 1],
            flux_history: VecDeque::with_capacity(flux_history_size),
            flux_history_size,

            last_beat_time: 0,
            beat_intervals: VecDeque::with_capacity(10),
            total_samples_processed: 0,

            last_bpm: None,
            beat_detected: false,

            sample_rate,
            fft_size,
            hop_size: fft_size / 2, // 50% overlap
            buffer: Vec::with_capacity(fft_size),
        }
    }

    fn apply_window(buffer: &[f32], output: &mut [Complex<f32>]) {
        let len = buffer.len();
        for (i, &sample) in buffer.iter().enumerate() {
            // Hann window: 0.5 * (1 - cos(2*pi*n / (N-1)))
            let window =
                0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (len - 1) as f32).cos());
            output[i] = Complex::new(sample * window, 0.0);
        }
    }

    fn compute_flux(&mut self, spectrum: &[f32]) -> f32 {
        let mut flux = 0.0;
        for (i, &mag) in spectrum.iter().enumerate() {
            let diff = mag - self.prev_spectrum[i];
            if diff > 0.0 {
                flux += diff;
            }
        }
        flux
    }
}

impl BeatDetector for SpectralFluxDetector {
    fn process(&mut self, samples: &[f32], _sample_rate: u32) {
        self.beat_detected = false;
        self.buffer.extend_from_slice(samples);
        self.total_samples_processed += samples.len() as u64;

        // Process as many frames as possible from the buffer
        // We implement overlapping windows.
        // If buffer >= fft_size, process, then remove hop_size
        while self.buffer.len() >= self.fft_size {
            let frame = &self.buffer[0..self.fft_size];

            // 1. Prepare FFT input with Windowing
            let mut fft_buffer = vec![Complex::zero(); self.fft_size];
            Self::apply_window(frame, &mut fft_buffer);

            // 2. Compute FFT
            self.fft
                .process_with_scratch(&mut fft_buffer, &mut self.scratch);

            // 3. Compute Magnitude Spectrum (only first half needed for real input)
            // Normalization? 1/N usually.
            let spectrum_len = self.fft_size / 2 + 1;
            let mut spectrum = Vec::with_capacity(spectrum_len);
            for item in fft_buffer.iter().take(spectrum_len) {
                spectrum.push(item.norm() / self.fft_size as f32);
            }

            // 4. Compute Flux
            let flux = self.compute_flux(&spectrum);

            // Update previous spectrum
            self.prev_spectrum = spectrum;

            // 5. Peak Picking
            let local_average: f32 = if self.flux_history.is_empty() {
                0.0
            } else {
                self.flux_history.iter().sum::<f32>() / self.flux_history.len() as f32
            };

            if self.flux_history.len() >= self.flux_history_size {
                self.flux_history.pop_front();
            }
            self.flux_history.push_back(flux);

            // Threshold multiplier
            let c = 1.5;

            if flux > c * local_average && flux > 0.0001 {
                // Min threshold to avoid noise
                let min_interval = (self.sample_rate as f32 * 0.2) as u64; // 200ms debounce (limit to ~300 BPM)
                                                                           // We need to estimate the time of this frame.
                                                                           // We are processing frames delayed.
                                                                           // The end of this frame is at current total processed - remaining buffer?
                                                                           // Or we can just use the current time as 'now'.
                                                                           // Since we process inside the loop, let's just approximate 'now' as the time corresponding to the *end* of this frame.
                                                                           // But self.total_samples_processed is the end of *newly added* samples.
                                                                           // The frame corresponds to the start of the buffer.
                                                                           // Let's track the "current frame time".
                                                                           // Actually, for simplicity, if we detect a beat in this chunk, we mark it.
                                                                           // The exact timing might be slightly off if we process multiple frames at once, but usually chunks are small.

                // Better time tracking:
                // We consumed samples up to current total.
                // The frame we just processed ended at (total_samples_processed - buffer.len() + fft_size)
                // But since we are removing hop_size later, let's act relative to total_samples_processed.

                // Actually, simply using `self.total_samples_processed` is okay if the buffer is kept small.
                // But if we are processing a backlog, `total_samples_processed` is the "head" of the stream.
                // The beat happened `self.buffer.len() - self.fft_size` samples ago relative to `total_samples_processed`.
                // Wait, no. `buffer` contains OLD samples at index 0.
                // So the frame is at the beginning of the buffer.
                // The end of the buffer is the most recent sample.
                // So this frame ended `self.buffer.len() - self.fft_size` samples BEFORE the most recent sample.

                let samples_ago = self.buffer.len() as u64; // Start of buffer
                let frame_time = self.total_samples_processed - samples_ago + self.fft_size as u64;

                if frame_time > self.last_beat_time + min_interval {
                    self.beat_detected = true;

                    let interval = frame_time - self.last_beat_time;
                    self.last_beat_time = frame_time;

                    if interval > 0 && interval < (self.sample_rate * 2) as u64 {
                        self.beat_intervals.push_back(interval);
                        if self.beat_intervals.len() > 8 {
                            self.beat_intervals.pop_front();
                        }

                        let avg_interval: u64 = self.beat_intervals.iter().sum::<u64>()
                            / self.beat_intervals.len() as u64;
                        let bpm = 60.0 * self.sample_rate as f32 / avg_interval as f32;

                        self.last_bpm = Some(match self.last_bpm {
                            Some(old) => old * 0.3 + bpm * 0.7,
                            None => bpm,
                        });
                    }
                }
            }

            // Advance buffer by hop_size
            // Vector draining is O(N), ringbuf might be better but Vec is fine for small buffers
            self.buffer.drain(0..self.hop_size);
        }
    }

    fn process_level(&mut self, _level: f32, _timestamp_ms: u64) {
        // Not used
    }

    fn current_bpm(&self) -> Option<f32> {
        self.last_bpm
    }

    fn is_beat(&self) -> bool {
        self.beat_detected
    }
}

impl Default for SpectralFluxDetector {
    fn default() -> Self {
        Self::new(44100, 1024)
    }
}
