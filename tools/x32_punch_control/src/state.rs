//! Application state for `x32_punch_control`.

use std::time::Duration;

/// Playback and recording mode for the punch controller.
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum Mode {
    /// Controller is idle.
    Idle,
    /// Playback is active.
    Playing,
    /// Playback is paused.
    Paused,
    /// Recording is active.
    Recording,
}

/// Shared runtime state for `x32_punch_control`.
#[derive(Debug)]
#[allow(dead_code)]
pub struct AppState {
    /// Current operating mode.
    pub mode: Mode,
    /// Whether playback is active.
    pub xplay: bool,
    /// Whether punch-in is active.
    pub xpunch: bool,
    /// Whether recording is active.
    pub xrecord: bool,
    /// Whether parameter merging is enabled.
    pub xmerge: bool,
    /// Whether playback is paused.
    pub xpause: bool,

    // timeval structs translated
    /// Timestamp of the last file read.
    pub dt_read: Duration,
    /// Timestamp of the current playback position.
    pub dt_play: Duration,
    /// Current wall-clock time.
    pub t_now: Duration,
    /// Playback start time.
    pub t_play: Duration,
    /// Pause start time.
    pub t_pause: Duration,
    /// Rewind target time.
    pub t_rew: Duration,
    /// Fast-forward target time.
    pub t_ff: Duration,

    /// Whether faders are currently being merged.
    pub xmergefaders: bool,
    /// Whether a punch file is being read.
    pub xreadfile: bool,
    /// Whether file data is ready for playback.
    pub xfiledataready: bool,

    /// Whether MIDI is connected.
    pub xmconnected: bool,
    /// Whether MIDI Time Code (MTC) is enabled.
    pub xmtcon: bool,

    // Timecode variables
    /// MIDI timecode hours.
    pub xmidihr: u32,
    /// MIDI timecode minutes.
    pub xmidimn: u32,
    /// MIDI timecode seconds.
    pub xmidiss: u32,
    /// MIDI timecode frames.
    pub xmidifr: u32,
    /// MIDI timecode frame rate.
    pub xfrrate: u32,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            mode: Mode::Idle,
            xplay: false,
            xpunch: false,
            xrecord: false,
            xmerge: true, // "by default and in normal operation, Merging is enabled (Xmerge = 0)" ? In C, xmerge=0 is merge on. We'll set xmerge true = merge ON.
            xpause: false,
            dt_read: Duration::ZERO,
            dt_play: Duration::ZERO,
            t_now: Duration::ZERO,
            t_play: Duration::ZERO,
            t_pause: Duration::ZERO,
            t_rew: Duration::ZERO,
            t_ff: Duration::ZERO,
            xmergefaders: false,
            xreadfile: false,
            xfiledataready: false,
            xmconnected: false,
            xmtcon: false,
            xmidihr: 0,
            xmidimn: 0,
            xmidiss: 0,
            xmidifr: 0,
            xfrrate: 0,
        }
    }
}
