//! Full console OSC event recorder & synchronized replayer (`.x32events.jsonl`).
//!
//! Captures an initial [`x32_lib::safety::ConsoleSnapshot`] keyframe followed by timestamped OSC parameter
//! changes and user markers, flushed line-by-line for crash safety and replayed in sync
//! with DAW Virtual Soundcheck playback.

use crate::mixer::MixerManager;
use crate::UiEvent;
use anyhow::{Context, Result};
use osc_lib::{OscArg, OscMessage};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Read as _, Write as _};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::{broadcast, Mutex};
use tokio::task::JoinHandle;
use x32_lib::safety::ConsoleSnapshot;

const MAX_EVENT_FILE_BYTES: u64 = 16 * 1024 * 1024;

/// Serializable OSC argument stored in an event recording.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum RecordedOscArg {
    /// 32-bit signed integer argument.
    Int(i32),
    /// 32-bit floating-point argument.
    Float(f32),
    /// UTF-8 string argument.
    String(String),
}

impl RecordedOscArg {
    /// Converts from [`OscArg`], skipping raw binary meter blobs.
    pub fn from_osc(arg: &OscArg) -> Option<Self> {
        match arg {
            OscArg::Int(i) => Some(Self::Int(*i)),
            OscArg::Float(f) => Some(Self::Float(*f)),
            OscArg::String(s) => Some(Self::String(s.clone())),
            OscArg::Blob(_) => None,
        }
    }

    /// Converts back to [`OscArg`].
    pub fn to_osc(&self) -> OscArg {
        match self {
            Self::Int(i) => OscArg::Int(*i),
            Self::Float(f) => OscArg::Float(*f),
            Self::String(s) => OscArg::String(s.clone()),
        }
    }
}

/// One JSON-Lines record inside a `.x32events.jsonl` session file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum RecordedEvent {
    /// Initial full console state snapshot captured at recording start.
    Keyframe {
        /// Snapshot entries recorded at `t = 0 ms`.
        snapshot: ConsoleSnapshot,
    },
    /// Timestamped console OSC event.
    Osc {
        /// Elapsed milliseconds since recording started.
        offset_ms: u64,
        /// OSC address path.
        path: String,
        /// Typed OSC arguments.
        args: Vec<RecordedOscArg>,
    },
    /// Named cue/song marker dropped during recording.
    Marker {
        /// Elapsed milliseconds since recording started.
        offset_ms: u64,
        /// Marker label.
        label: String,
    },
}

/// Live status summary for [`EventRecorder`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecorderStatus {
    /// `"idle"`, `"recording"`, or `"playing"`.
    pub state: String,
    /// Path to the currently active or most recent recording file.
    pub active_file: Option<String>,
    /// Number of OSC events recorded or replayed in the current/last run.
    pub event_count: usize,
    /// Number of markers recorded in the current/last run.
    pub marker_count: usize,
    /// Elapsed milliseconds in current recording or playback.
    pub elapsed_ms: u64,
    /// Whether event replay is synced with DAW Play mode.
    pub sync_replay_enabled: bool,
}

impl Default for RecorderStatus {
    fn default() -> Self {
        Self {
            state: "idle".to_string(),
            active_file: None,
            event_count: 0,
            marker_count: 0,
            elapsed_ms: 0,
            sync_replay_enabled: true,
        }
    }
}

/// Summary metadata for a saved `.x32events.jsonl` file on disk.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecordingInfo {
    /// File name.
    pub name: String,
    /// Full path on disk.
    pub path: String,
    /// File size in bytes.
    pub size_bytes: u64,
    /// Number of OSC event lines.
    pub events: usize,
    /// Number of marker lines.
    pub markers: usize,
}

struct ActiveRecording {
    path: PathBuf,
    writer: BufWriter<File>,
    started_at: Instant,
    event_count: usize,
    marker_count: usize,
}

struct ActivePlayback {
    path: PathBuf,
    started_at: Instant,
    task: JoinHandle<()>,
}

struct RecorderInner {
    recording: Option<ActiveRecording>,
    playback: Option<ActivePlayback>,
    last_file: Option<String>,
    last_event_count: usize,
    last_marker_count: usize,
    sync_replay_enabled: bool,
}

/// Thread-safe full console OSC event recorder and timed replayer.
pub struct EventRecorder {
    inner: Arc<Mutex<RecorderInner>>,
}

impl Default for EventRecorder {
    fn default() -> Self {
        Self::new()
    }
}

impl EventRecorder {
    /// Creates a new idle [`EventRecorder`].
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(RecorderInner {
                recording: None,
                playback: None,
                last_file: None,
                last_event_count: 0,
                last_marker_count: 0,
                sync_replay_enabled: true,
            })),
        }
    }

    /// Sets whether DAW Play mode automatically triggers event playback.
    pub async fn set_sync_replay_enabled(&self, enabled: bool) {
        self.inner.lock().await.sync_replay_enabled = enabled;
    }

    /// Returns the current [`RecorderStatus`].
    pub async fn status(&self) -> RecorderStatus {
        let guard = self.inner.lock().await;
        if let Some(ref rec) = guard.recording {
            return RecorderStatus {
                state: "recording".to_string(),
                active_file: Some(rec.path.to_string_lossy().to_string()),
                event_count: rec.event_count,
                marker_count: rec.marker_count,
                elapsed_ms: rec.started_at.elapsed().as_millis() as u64,
                sync_replay_enabled: guard.sync_replay_enabled,
            };
        }
        if let Some(ref pb) = guard.playback {
            if !pb.task.is_finished() {
                return RecorderStatus {
                    state: "playing".to_string(),
                    active_file: Some(pb.path.to_string_lossy().to_string()),
                    event_count: guard.last_event_count,
                    marker_count: guard.last_marker_count,
                    elapsed_ms: pb.started_at.elapsed().as_millis() as u64,
                    sync_replay_enabled: guard.sync_replay_enabled,
                };
            }
        }
        RecorderStatus {
            state: "idle".to_string(),
            active_file: guard.last_file.clone(),
            event_count: guard.last_event_count,
            marker_count: guard.last_marker_count,
            elapsed_ms: 0,
            sync_replay_enabled: guard.sync_replay_enabled,
        }
    }

    /// Starts recording a new `.x32events.jsonl` file in `dir`, writing `initial_snapshot` as Keyframe.
    pub async fn start_recording(
        &self,
        dir: &Path,
        session_label: &str,
        initial_snapshot: ConsoleSnapshot,
    ) -> Result<String> {
        let mut guard = self.inner.lock().await;
        if let Some(pb) = guard.playback.take() {
            pb.task.abort();
        }
        if let Some(mut existing) = guard.recording.take() {
            let _ = existing.writer.flush();
        }

        std::fs::create_dir_all(dir)
            .with_context(|| format!("Failed creating recordings dir '{}'", dir.display()))?;

        let clean_label: String = session_label
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        let label = if clean_label.is_empty() {
            "session".to_string()
        } else {
            clean_label
        };
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let file_path = dir.join(format!("{}_{}.x32events.jsonl", label, ts));

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&file_path)?;
        let mut writer = BufWriter::new(file);

        let keyframe = RecordedEvent::Keyframe {
            snapshot: initial_snapshot,
        };
        serde_json::to_writer(&mut writer, &keyframe)?;
        writer.write_all(b"\n")?;
        writer.flush()?;

        let path_str = file_path.to_string_lossy().to_string();
        guard.last_file = Some(path_str.clone());
        guard.last_event_count = 0;
        guard.last_marker_count = 0;
        guard.recording = Some(ActiveRecording {
            path: file_path,
            writer,
            started_at: Instant::now(),
            event_count: 0,
            marker_count: 0,
        });

        Ok(path_str)
    }

    /// Appends an incoming console OSC message to the active recording (ignoring telemetry/button paths).
    pub async fn on_incoming_osc(&self, msg: &OscMessage) {
        if should_ignore_osc_path(&msg.path) || msg.args.is_empty() {
            return;
        }
        let args: Vec<RecordedOscArg> = msg
            .args
            .iter()
            .filter_map(RecordedOscArg::from_osc)
            .collect();
        if args.is_empty() {
            return;
        }

        let mut guard = self.inner.lock().await;
        if let Some(ref mut rec) = guard.recording {
            let ev = RecordedEvent::Osc {
                offset_ms: rec.started_at.elapsed().as_millis() as u64,
                path: msg.path.clone(),
                args,
            };
            if serde_json::to_writer(&mut rec.writer, &ev).is_ok() {
                let _ = rec.writer.write_all(b"\n");
                let _ = rec.writer.flush();
                rec.event_count += 1;
            }
        }
    }

    /// Appends a named marker at the current recording offset.
    pub async fn add_marker(&self, label: &str) -> Result<usize> {
        let mut guard = self.inner.lock().await;
        let rec = guard
            .recording
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("No active event recording"))?;
        let offset_ms = rec.started_at.elapsed().as_millis() as u64;
        let ev = RecordedEvent::Marker {
            offset_ms,
            label: label.to_string(),
        };
        serde_json::to_writer(&mut rec.writer, &ev)?;
        rec.writer.write_all(b"\n")?;
        rec.writer.flush()?;
        rec.marker_count += 1;
        Ok(rec.marker_count)
    }

    /// Stops and flushes any active event recording, returning the saved file path if one was active.
    pub async fn stop_recording(&self) -> Result<Option<String>> {
        let mut guard = self.inner.lock().await;
        let Some(mut rec) = guard.recording.take() else {
            return Ok(None);
        };
        rec.writer.flush()?;
        let path_str = rec.path.to_string_lossy().to_string();
        guard.last_file = Some(path_str.clone());
        guard.last_event_count = rec.event_count;
        guard.last_marker_count = rec.marker_count;
        Ok(Some(path_str))
    }

    /// Starts timed playback of a `.x32events.jsonl` recording against `mixer`.
    pub async fn start_playback(
        &self,
        file_path: &Path,
        mixer: Arc<MixerManager>,
        apply_keyframe: bool,
        events_tx: Option<broadcast::Sender<UiEvent>>,
    ) -> Result<()> {
        let events = load_recording_events(file_path)?;

        let mut guard = self.inner.lock().await;
        if let Some(pb) = guard.playback.take() {
            pb.task.abort();
        }
        if let Some(mut rec) = guard.recording.take() {
            let _ = rec.writer.flush();
        }

        let osc_count = events
            .iter()
            .filter(|e| matches!(e, RecordedEvent::Osc { .. }))
            .count();
        let marker_count = events
            .iter()
            .filter(|e| matches!(e, RecordedEvent::Marker { .. }))
            .count();

        let path_buf = file_path.to_path_buf();
        guard.last_file = Some(path_buf.to_string_lossy().to_string());
        guard.last_event_count = osc_count;
        guard.last_marker_count = marker_count;

        let inner_ref = self.inner.clone();
        let started_at = Instant::now();
        let task = tokio::spawn(async move {
            let mut keyframe_applied = false;
            for ev in events {
                match ev {
                    RecordedEvent::Keyframe { snapshot } => {
                        if apply_keyframe && !keyframe_applied {
                            keyframe_applied = true;
                            for (p, val) in &snapshot.entries {
                                let _ = mixer.send_osc(p, vec![OscArg::from(val)]).await;
                            }
                        }
                    }
                    RecordedEvent::Osc {
                        offset_ms,
                        path,
                        args,
                    } => {
                        let target = started_at
                            + std::time::Duration::from_millis(offset_ms);
                        let now = Instant::now();
                        if target > now {
                            tokio::time::sleep(target - now).await;
                        }
                        let osc_args: Vec<OscArg> =
                            args.iter().map(RecordedOscArg::to_osc).collect();
                        let _ = mixer.send_osc(&path, osc_count_or_args(osc_args)).await;
                    }
                    RecordedEvent::Marker { offset_ms, label } => {
                        let target = started_at
                            + std::time::Duration::from_millis(offset_ms);
                        let now = Instant::now();
                        if target > now {
                            tokio::time::sleep(target - now).await;
                        }
                        if let Some(ref tx) = events_tx {
                            let _ = tx.send(UiEvent::VscStatus {
                                message: format!("Playback Marker: {}", label),
                                play_mode: true,
                            });
                        }
                    }
                }
            }
            let mut g = inner_ref.lock().await;
            g.playback = None;
        });

        guard.playback = Some(ActivePlayback {
            path: path_buf,
            started_at,
            task,
        });
        Ok(())
    }

    /// Stops any active event playback task.
    pub async fn stop_playback(&self) {
        let mut guard = self.inner.lock().await;
        if let Some(pb) = guard.playback.take() {
            pb.task.abort();
        }
    }

    /// Scans `dir` for `.x32events.jsonl` recordings sorted newest-first.
    pub fn list_recordings(dir: &Path) -> Vec<RecordingInfo> {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return Vec::new();
        };
        let mut list = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let name = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            if !name.ends_with(".x32events.jsonl") {
                continue;
            }
            let size_bytes = entry.metadata().map(|m| m.len()).unwrap_or(0);
            let (events, markers) = count_recording_lines(&path);
            list.push(RecordingInfo {
                name,
                path: path.to_string_lossy().to_string(),
                size_bytes,
                events,
                markers,
            });
        }
        list.sort_by(|a, b| b.name.cmp(&a.name));
        list
    }
}

fn osc_count_or_args(args: Vec<OscArg>) -> Vec<OscArg> {
    args
}

fn should_ignore_osc_path(path: &str) -> bool {
    path.starts_with("/meters")
        || path == "/xremote"
        || path == "/info"
        || path == "/status"
        || path == "/renew"
        || path.starts_with("/-stat/userpar/")
        || path.starts_with("/config/userctrl/")
        || path == "/-stat/userbank"
}

/// Loads all [`RecordedEvent`] entries from a `.x32events.jsonl` file (bounded to 16 MiB).
pub fn load_recording_events(path: &Path) -> Result<Vec<RecordedEvent>> {
    let file = File::open(path)
        .with_context(|| format!("Failed to open recording '{}'", path.display()))?;
    if file.metadata()?.len() > MAX_EVENT_FILE_BYTES {
        anyhow::bail!(
            "Recording '{}' exceeds {} byte safety limit",
            path.display(),
            MAX_EVENT_FILE_BYTES
        );
    }
    let reader = BufReader::new(file.take(MAX_EVENT_FILE_BYTES));
    let mut out = Vec::new();
    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(ev) = serde_json::from_str::<RecordedEvent>(trimmed) {
            out.push(ev);
        }
    }
    Ok(out)
}

fn count_recording_lines(path: &Path) -> (usize, usize) {
    let Ok(events) = load_recording_events(path) else {
        return (0, 0);
    };
    let mut osc_n = 0;
    let mut marker_n = 0;
    for ev in events {
        match ev {
            RecordedEvent::Osc { .. } => osc_n += 1,
            RecordedEvent::Marker { .. } => marker_n += 1,
            RecordedEvent::Keyframe { .. } => {}
        }
    }
    (osc_n, marker_n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_event_recorder_captures_keyframe_events_and_markers_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let rec = EventRecorder::new();

        let mut snap = ConsoleSnapshot::new();
        snap.record_float("/ch/01/mix/fader", 0.5);
        snap.record_string("/ch/01/config/name", "Kick");

        let saved_path = rec
            .start_recording(tmp.path(), "Sunday Service", snap.clone())
            .await
            .unwrap();

        // Ignored telemetry / button paths should not be recorded
        rec.on_incoming_osc(&OscMessage::new(
            "/meters/0".to_string(),
            vec![OscArg::Blob(vec![0, 0, 0, 0])],
        ))
        .await;
        rec.on_incoming_osc(&OscMessage::new(
            "/-stat/userpar/17/value".to_string(),
            vec![OscArg::Int(127)],
        ))
        .await;

        // Real console mix events should be recorded
        rec.on_incoming_osc(&OscMessage::new(
            "/ch/01/mix/fader".to_string(),
            vec![OscArg::Float(0.75)],
        ))
        .await;
        rec.add_marker("Chorus 1").await.unwrap();

        let status = rec.status().await;
        assert_eq!(status.state, "recording");
        assert_eq!(status.event_count, 1);
        assert_eq!(status.marker_count, 1);

        let stopped = rec.stop_recording().await.unwrap();
        assert_eq!(stopped.as_deref(), Some(saved_path.as_str()));

        let loaded = load_recording_events(Path::new(&saved_path)).unwrap();
        assert_eq!(loaded.len(), 3);
        assert!(matches!(&loaded[0], RecordedEvent::Keyframe { snapshot } if snapshot == &snap));
        assert!(
            matches!(&loaded[1], RecordedEvent::Osc { path, args, .. } if path == "/ch/01/mix/fader" && args == &vec![RecordedOscArg::Float(0.75)])
        );
        assert!(
            matches!(&loaded[2], RecordedEvent::Marker { label, .. } if label == "Chorus 1")
        );

        let list = EventRecorder::list_recordings(tmp.path());
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].events, 1);
        assert_eq!(list[0].markers, 1);
    }
}
