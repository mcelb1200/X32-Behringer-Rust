//! Streaming output loudness, peak-hold, and silence-drop monitoring.

use crate::meters::linear_to_dbfs;
use serde::{Deserialize, Serialize};

/// Configuration for monitoring a dedicated broadcast/livestream bus or matrix.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StreamConfig {
    /// Source bus or matrix (e.g. `"bus15"`, `"mtx1"`, or `"main"`).
    pub source: String,
    /// Target RMS level in dBFS (e.g. `-16.0`).
    pub target_rms_db: f32,
    /// Minimum acceptable RMS level in dBFS before alerting `TooQuiet` (e.g. `-24.0`).
    pub min_rms_db: f32,
    /// Peak ceiling in dBFS before alerting `Clipping` (e.g. `-2.0`).
    pub max_peak_db: f32,
    /// Whether bounded automatic trim nudges are enabled (defaults to `false` for monitor-only safety).
    #[serde(default)]
    pub auto_trim: bool,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            source: "bus15".to_string(),
            target_rms_db: -16.0,
            min_rms_db: -24.0,
            max_peak_db: -2.0,
            auto_trim: false,
        }
    }
}

/// Health classification for the livestream/recording feed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamStatus {
    /// Audio has fallen below `-60 dBFS` for multiple consecutive frames.
    Silent,
    /// Average level is below `min_rms_db`.
    TooQuiet,
    /// Level sits cleanly in the broadcast target window.
    Optimal,
    /// Level is approaching the peak ceiling without clipping yet.
    Hot,
    /// Peak exceeds `max_peak_db`.
    Clipping,
}

/// Snapshot of stream metering telemetry returned on each update.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StreamTelemetry {
    /// Smoothed RMS estimate in dBFS.
    pub rms_db: f32,
    /// Decaying peak-hold level in dBFS.
    pub peak_db: f32,
    /// Classified stream health status.
    pub status: StreamStatus,
    /// Suggested bounded trim adjustment in dB (`0.0` when `auto_trim` is disabled).
    pub trim_nudge_db: f32,
}

/// Tracks stream RMS/peak loudness and emits alerts or bounded trim nudges.
#[derive(Debug, Clone)]
pub struct StreamMonitor {
    config: StreamConfig,
    ema_linear: f32,
    peak_linear: f32,
    silent_frames: u32,
}

impl StreamMonitor {
    /// Creates a new [`StreamMonitor`] from the given [`StreamConfig`].
    pub fn new(config: StreamConfig) -> Self {
        Self {
            config,
            ema_linear: 0.0,
            peak_linear: 0.0,
            silent_frames: 0,
        }
    }

    /// Returns the monitor's configuration.
    pub fn config(&self) -> &StreamConfig {
        &self.config
    }

    /// Updates stream telemetry with a new linear `[0.0, 1.0]` meter sample.
    pub fn update(&mut self, level_linear: f32) -> StreamTelemetry {
        let lvl = level_linear.clamp(0.0, 1.0);
        self.ema_linear = self.ema_linear * 0.85 + lvl * 0.15;
        self.peak_linear = (self.peak_linear * 0.92).max(lvl);

        let rms_db = linear_to_dbfs(self.ema_linear);
        let peak_db = linear_to_dbfs(self.peak_linear);

        if rms_db <= -60.0 {
            self.silent_frames = self.silent_frames.saturating_add(1);
        } else {
            self.silent_frames = 0;
        }

        let status = if peak_db > self.config.max_peak_db {
            StreamStatus::Clipping
        } else if self.silent_frames >= 5 {
            StreamStatus::Silent
        } else if rms_db < self.config.min_rms_db {
            StreamStatus::TooQuiet
        } else if rms_db > self.config.target_rms_db + 4.0 {
            StreamStatus::Hot
        } else {
            StreamStatus::Optimal
        };

        let trim_nudge_db = if self.config.auto_trim {
            match status {
                StreamStatus::Clipping => -1.5,
                StreamStatus::Hot => -0.5,
                StreamStatus::TooQuiet => 0.5,
                StreamStatus::Optimal | StreamStatus::Silent => 0.0,
            }
        } else {
            0.0
        };

        StreamTelemetry {
            rms_db,
            peak_db,
            status,
            trim_nudge_db,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_monitor_states_and_auto_trim() {
        let mut mon = StreamMonitor::new(StreamConfig::default());
        let mut t = mon.update(0.0);
        for _ in 0..6 {
            t = mon.update(0.0);
        }
        assert_eq!(t.status, StreamStatus::Silent);
        assert_eq!(t.trim_nudge_db, 0.0);

        // ~ -16 dBFS -> 10^(-16/20) = ~0.158
        for _ in 0..20 {
            t = mon.update(0.16);
        }
        assert_eq!(t.status, StreamStatus::Optimal);

        // Clipping (> -2 dBFS -> ~0.794)
        let mut trim_mon = StreamMonitor::new(StreamConfig {
            auto_trim: true,
            ..Default::default()
        });
        let clip = trim_mon.update(0.95);
        assert_eq!(clip.status, StreamStatus::Clipping);
        assert_eq!(clip.trim_nudge_db, -1.5);
    }
}
