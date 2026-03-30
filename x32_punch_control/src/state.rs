use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum Mode {
    Idle,
    Playing,
    Paused,
    Recording,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct AppState {
    pub mode: Mode,
    pub xplay: bool,
    pub xpunch: bool,
    pub xrecord: bool,
    pub xmerge: bool,
    pub xpause: bool,

    // timeval structs translated
    pub dt_read: Duration,
    pub dt_play: Duration,
    pub t_now: Duration,
    pub t_play: Duration,
    pub t_pause: Duration,
    pub t_rew: Duration,
    pub t_ff: Duration,

    pub xmergefaders: bool,
    pub xreadfile: bool,
    pub xfiledataready: bool,

    pub xmconnected: bool,
    pub xmtcon: bool,

    // Timecode variables
    pub xmidihr: u32,
    pub xmidimn: u32,
    pub xmidiss: u32,
    pub xmidifr: u32,
    pub xfrrate: u32,
    pub xcatchdelay: i32,
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
            xcatchdelay: 10,
        }
    }
}
