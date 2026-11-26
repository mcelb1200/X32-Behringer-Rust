use crate::network::ChannelState;
use std::time::Instant;
use x32_lib::automix::SpectrumAutomixer;
use x32_lib::safety::ConsoleSnapshot;
use x32_lib::streaming::{StreamMonitor, StreamTelemetry};
use x32_lib::MixerModel;

/// Active operational alert displayed in the Volunteer Mode TUI.
pub enum Alert {
    /// A channel meter exceeded the clipping threshold.
    Clipping {
        /// Index into `AppState::channels`.
        channel_idx: usize,
        /// Measured level in dBFS.
        level_db: f32,
    },
    /// A channel fader exceeded the configured maximum limit.
    FaderLimit {
        /// Index into `AppState::channels`.
        channel_idx: usize,
        /// Current fader level in dB.
        fader_db: f32,
        /// Configured fader limit in dB.
        limit: f32,
    },
}

/// Shared application state for the Volunteer Mode TUI.
pub struct AppState {
    /// Connected mixer model.
    pub model: MixerModel,
    /// Monitored channels and DCAs.
    pub channels: Vec<ChannelState>,
    /// Currently selected channel index.
    pub selected_idx: usize,
    /// Main LR fader position (`0.0..=1.0`).
    pub main_fader: f32,
    /// Whether the main LR bus is muted.
    pub main_muted: bool,
    /// Indices of channels with active alerts.
    pub alerts: Vec<usize>,
    /// Detailed active alert entries.
    pub active_alerts: Vec<Alert>,
    /// Overall system health status.
    pub status: Status,
    /// Status banner message.
    pub message: String,
    /// Optional maximum fader limit in dB.
    pub max_fader_limit_db: Option<f32>,
    /// Timestamp when the panic confirmation prompt was armed.
    pub confirm_panic_at: Option<Instant>,
    /// Timestamp of the last received OSC packet from the mixer.
    pub last_rx_at: Option<Instant>,
    /// Application startup timestamp.
    pub started_at: Instant,
    /// Most recent console snapshot for undo/restore.
    pub last_snapshot: Option<ConsoleSnapshot>,
    /// Whether mixbus automix is currently enabled.
    pub automix_enabled: bool,
    /// Whether RTA-assisted automix is enabled.
    pub rta_assisted: bool,
    /// Optional spectrum automixer instance.
    pub automixer: Option<SpectrumAutomixer>,
    /// Optional livestream monitor instance.
    pub stream_monitor: Option<StreamMonitor>,
    /// Latest livestream telemetry reading.
    pub stream_telemetry: Option<StreamTelemetry>,
    /// Current stream bus fader position (`0.0..=1.0`).
    pub stream_fader: f32,
    /// Optional soundcheck status message.
    pub check_status_msg: Option<String>,
}

/// Overall health status indicator for the Volunteer Mode dashboard.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Status {
    /// All monitored channels are within normal operating bounds.
    Ok,
    /// A non-critical condition requires attention.
    Caution,
    /// A critical alert (such as clipping or offline console) is active.
    Problem,
}

impl AppState {
    /// Creates a new `AppState` for the given monitored channels.
    pub fn new(channels: Vec<ChannelState>) -> Self {
        Self {
            model: MixerModel::X32,
            channels,
            selected_idx: 0,
            main_fader: 0.0,
            main_muted: false,
            alerts: vec![],
            active_alerts: vec![],
            status: Status::Ok,
            message: "Starting up...".to_string(),
            max_fader_limit_db: None,
            confirm_panic_at: None,
            last_rx_at: None,
            started_at: Instant::now(),
            last_snapshot: None,
            automix_enabled: false,
            rta_assisted: false,
            automixer: None,
            stream_monitor: None,
            stream_telemetry: None,
            stream_fader: 0.75,
            check_status_msg: None,
        }
    }

    /// Returns true if a panic confirmation prompt is currently active (within 3 seconds).
    pub fn is_panic_armed(&self) -> bool {
        self.confirm_panic_at
            .map(|t| t.elapsed().as_secs() < 3)
            .unwrap_or(false)
    }

    /// Returns true if the console has responded recently (within 3 seconds), or still in initial grace window.
    pub fn is_online(&self) -> bool {
        match self.last_rx_at {
            Some(t) => t.elapsed().as_secs() < 3,
            None => self.started_at.elapsed().as_secs() < 3,
        }
    }
}
