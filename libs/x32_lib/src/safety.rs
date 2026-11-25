//! Shared stage-safety primitives: console state snapshots (undo/backup) and smooth exponential fades.

use crate::MixerClient;
use osc_lib::OscArg;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

/// Serializable OSC parameter value stored in a [`ConsoleSnapshot`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum SnapshotValue {
    /// Floating-point parameter (e.g., fader or EQ gain).
    Float(f32),
    /// Integer parameter (e.g., mute/on flag).
    Int(i32),
    /// String parameter (e.g., scribble-strip name).
    String(String),
}

impl From<&SnapshotValue> for OscArg {
    fn from(val: &SnapshotValue) -> Self {
        match val {
            SnapshotValue::Float(f) => OscArg::Float(*f),
            SnapshotValue::Int(i) => OscArg::Int(*i),
            SnapshotValue::String(s) => OscArg::String(s.clone()),
        }
    }
}

/// Captures OSC parameter values before destructive or automated actions so they can be undone or saved.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ConsoleSnapshot {
    /// Map of OSC address path to saved value.
    pub entries: BTreeMap<String, SnapshotValue>,
}

impl ConsoleSnapshot {
    /// Creates an empty snapshot.
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a float parameter (e.g., fader or EQ gain).
    pub fn record_float(&mut self, path: impl Into<String>, value: f32) {
        self.entries
            .insert(path.into(), SnapshotValue::Float(value));
    }

    /// Records an integer parameter (e.g., mute/on state).
    pub fn record_int(&mut self, path: impl Into<String>, value: i32) {
        self.entries.insert(path.into(), SnapshotValue::Int(value));
    }

    /// Records a string parameter (e.g., channel name).
    pub fn record_string(&mut self, path: impl Into<String>, value: impl Into<String>) {
        self.entries
            .insert(path.into(), SnapshotValue::String(value.into()));
    }

    /// Returns `true` if no entries have been recorded.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Restores every recorded entry back to the connected mixer.
    pub async fn restore(&self, client: &MixerClient) -> crate::Result<()> {
        for (path, val) in &self.entries {
            client.send_message(path, vec![OscArg::from(val)]).await?;
        }
        Ok(())
    }

    /// Persists the snapshot to disk as pretty-printed JSON.
    pub fn save_to_file(&self, path: &Path) -> crate::Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| crate::X32Error::Custom(e.to_string()))?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Loads a previously saved snapshot from disk (bounded to 1 MiB).
    pub fn load_from_file(path: &Path) -> crate::Result<Self> {
        use std::io::Read;
        let file = std::fs::File::open(path)?;
        let mut reader = file.take(1024 * 1024);
        let mut buf = String::new();
        reader.read_to_string(&mut buf)?;
        serde_json::from_str(&buf).map_err(|e| crate::X32Error::Custom(e.to_string()))
    }
}

/// Computes an exponential fade-out value at `step` (`1..=total_steps`) from `start_fader`.
pub fn exponential_fade_level(start_fader: f32, step: usize, total_steps: usize) -> f32 {
    if total_steps == 0 || step >= total_steps {
        return 0.0;
    }
    let remaining = 1.0 - (step as f32 / total_steps as f32);
    (start_fader * remaining * remaining).clamp(0.0, 1.0)
}

/// Smoothly fades a list of `(fader_path, start_fader)` targets down to `0.0` over `total_duration`.
pub async fn smooth_fade(
    client: &MixerClient,
    targets: &[(String, f32)],
    steps: usize,
    total_duration: Duration,
) -> crate::Result<()> {
    let steps = steps.max(1);
    let step_dur = total_duration / steps as u32;
    let mut ticker = tokio::time::interval(step_dur);

    for step in 1..=steps {
        ticker.tick().await;
        for (path, start_val) in targets {
            let next = exponential_fade_level(*start_val, step, steps);
            let _ = client.send_message(path, vec![OscArg::Float(next)]).await;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_roundtrip_and_fade_curve() {
        let mut snap = ConsoleSnapshot::new();
        snap.record_float("/ch/01/mix/fader", 0.75);
        snap.record_int("/ch/01/mix/on", 1);
        snap.record_string("/ch/01/config/name", "Vox");

        let dir = std::env::temp_dir();
        let file = dir.join("x32_lib_snapshot_test.json");
        snap.save_to_file(&file).unwrap();
        let loaded = ConsoleSnapshot::load_from_file(&file).unwrap();
        let _ = std::fs::remove_file(&file);
        assert_eq!(snap, loaded);

        assert!((exponential_fade_level(0.8, 0, 5) - 0.8).abs() < 1e-6);
        assert!(exponential_fade_level(0.8, 2, 5) < 0.8);
        assert_eq!(exponential_fade_level(0.8, 5, 5), 0.0);
    }
}
