
pub mod energy;
pub mod osc_level;

pub use energy::EnergyDetector;
pub use osc_level::OscLevelDetector;

/// Trait for beat detection algorithms.
///
/// Implementors should process incoming audio samples (or level data)
/// and return a detected tempo (BPM) and/or beat onset information.
pub trait BeatDetector {
    /// Process a new chunk of audio samples.
    ///
    /// * `samples`: A slice of audio samples (mono).
    /// * `sample_rate`: The sample rate of the audio data.
    fn process(&mut self, samples: &[f32], sample_rate: u32);

    /// Process a new level value (e.g. from OSC).
    ///
    /// * `level`: A normalized level value (0.0 - 1.0).
    /// * `timestamp_ms`: The time the level was received.
    fn process_level(&mut self, level: f32, timestamp_ms: u64);

    /// Get the current estimated BPM.
    fn current_bpm(&self) -> Option<f32>;

    /// Check if a beat onset was detected in the last processed chunk.
    fn is_beat(&self) -> bool;
}
