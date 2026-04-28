//! `x32_punch_control` is a command-line tool for managing DAW punch IN/OUT down mixing updates.
//!
//! This is a Rust rewrite of the original `X32PunchControl.c` Windows GUI app.
//! It connects to the X32 via UDP, parses OSC messages, manages `.xpc` (punch control) files,
//! and handles logic to merge, catch-up, or catch-back fader/parameter updates based on
//! User Bank button presses and timecode states.
//!
//! # Credits
//! *   **Original concept and work on the C library:** Patrick-Gilles Maillot
//! *   **Rust implementation by:** [User]

use anyhow::Result;
use clap::Parser;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tokio::time::{self, Duration, Instant};

mod config;
mod format;
mod state;

use config::Config;
use state::AppState;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// X32 IP Address
    #[arg(short, long)]
    ip: Option<String>,

    /// Path to config file (default: .X32PunchControl.ini)
    #[arg(long, default_value = ".X32PunchControl.ini")]
    config: String,

    /// Punch control file to read/write (.xpc)
    #[arg(short, long)]
    file: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let mut config = Config::load(&args.config).unwrap_or_default();
    if let Some(ip) = args.ip {
        config.xip_str = ip;
    }

    println!("X32PunchControl - Rust Rewrite");
    println!("Connecting to X32 at {}", config.xip_str);

    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    let x32_addr = format!("{}:10023", config.xip_str);
    socket.connect(&x32_addr).await?;
    let socket = Arc::new(socket);

    // Initial connection subscription
    socket.send(b"/xremote").await?;

    let state = Arc::new(Mutex::new(AppState::default()));

    // Background task to handle time-based playback/merge
    let bg_state = state.clone();
    let bg_sock = socket.clone();
    let config_clone = config.clone();
    let bg_file = args.file.clone();

    tokio::spawn(async move {
        run_logic(bg_state, bg_sock, config_clone, bg_file).await;
    });

    let mut buf = [0u8; 2048];
    let mut last_xremote = Instant::now();

    loop {
        tokio::select! {
            // Keepalive
            _ = time::sleep_until(last_xremote + Duration::from_secs(9)) => {
                let _ = socket.send(b"/xremote").await;
                last_xremote = Instant::now();
            }
            res = socket.recv(&mut buf) => {
                if let Ok(len) = res {
                    let data = &buf[..len];

                    // Parse User Bank inputs (buttons mapped to play/stop/rew/etc)
                    // ⚡ Bolt: Use byte slice operations instead of String::from_utf8_lossy to avoid allocations.
                    if data.starts_with(b"/-stat/userpar/") && data.len() >= 17 {
                        let mut lock = state.lock().await;

                        if let Ok(bnum) = std::str::from_utf8(&data[15..17]).unwrap_or("").parse::<u32>() {
                        // In c_origin: bnum = ((int)Xbank - 65) * 8 + i + 1;
                        // where Xbank is 'A', 'B', 'C'. A=65. So if Xbank='A', bnum is 1..8.
                        let bank_idx = (config.xbank as u32).saturating_sub(65);
                        let base_id = bank_idx * 8;

                        // we ignore "press in" by checking value, but here we just toggle simply based on receiving OSC
                        if bnum > base_id && bnum <= base_id + 8 {
                            let btn = bnum - base_id;
                            match btn {
                                1 => {
                                    // REW
                                    println!("REW requested");
                                    lock.t_rew = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default() + Duration::from_secs(1);
                                },
                                2 => {
                                    // PLAY
                                    lock.xplay = true;
                                    lock.xpause = false;
                                    println!("PLAY requested");
                                },
                                3 => {
                                    // PAUSE
                                    lock.xpause = !lock.xpause;
                                    println!("PAUSE requested ({})", lock.xpause);
                                },
                                4 => {
                                    // FF
                                    println!("FF requested");
                                    lock.t_ff = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default() + Duration::from_secs(1);
                                },
                                5 => {
                                    // PUNCH IN/OUT
                                    lock.xpunch = !lock.xpunch;
                                    println!("PUNCH requested ({})", lock.xpunch);
                                },
                                6 => {
                                    // MERGE
                                    lock.xmerge = !lock.xmerge;
                                    println!("MERGE requested ({})", lock.xmerge);
                                },
                                7 => {
                                    // STOP
                                    lock.xplay = false;
                                    lock.xpause = false;
                                    lock.xpunch = false;
                                    lock.xrecord = false;
                                    println!("STOP requested");
                                },
                                    8 => {
                                        // RECORD
                                        lock.xrecord = true;
                                        println!("RECORD requested");
                                    },
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

use format::{PunchReader, PunchRecord, PunchWriter};
use tokio::fs::File;

async fn run_logic(
    state: Arc<Mutex<AppState>>,
    socket: Arc<UdpSocket>,
    _config: Config,
    file_path: Option<String>,
) {
    let mut interval = time::interval(Duration::from_millis(50));

    let mut reader: Option<PunchReader> = None;
    let mut writer: Option<PunchWriter> = None;
    let mut current_record: Option<PunchRecord> = None;

    if let Some(ref path) = file_path {
        if let Ok(f) = File::open(path).await {
            reader = Some(PunchReader::new(f));
        }
        let out_path = format!("{}_xpc", path);
        if let Ok(f) = File::create(&out_path).await {
            writer = Some(PunchWriter::new(f));
        }
    }

    loop {
        interval.tick().await;

        let mut s = state.lock().await;

        // If file dataready flag is false, try to read the next record
        #[allow(clippy::collapsible_if)]
        if !s.xfiledataready && s.xreadfile {
            if let Some(ref mut r) = reader {
                match r.read_record().await {
                    Ok(Some(record)) => {
                        s.dt_read = record.time;
                        current_record = Some(record);
                        s.xfiledataready = true;
                    }
                    Ok(None) | Err(_) => {
                        s.xreadfile = false;
                        s.xfiledataready = false;
                        current_record = None;
                    }
                }
            }
        }

        if s.xplay && !s.xpause {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default();
            if s.t_play.is_zero() {
                s.t_play = now;
            }
            s.dt_play = now.saturating_sub(s.t_play);

            #[allow(clippy::collapsible_if)]
            if s.xfiledataready {
                if s.dt_play > s.dt_read {
                    if let Some(record) = current_record.take() {
                        let mut should_send = true;
                        if s.xmerge {
                            if s.xmergefaders {
                                // ⚡ Bolt: Search for "fader" byte pattern using windows() directly on slice
                                // to avoid String::from_utf8_lossy allocation.
                                if record.data.windows(5).any(|w| w == b"fader") {
                                    should_send = false;
                                }
                            } else {
                                should_send = false; // "prevent all writing from the file if Xmergefaders = 0"
                            }
                        }

                        if should_send {
                            let _ = socket.send(&record.data).await;
                        }

                        // Always write to the new file, following C logic XWriteAndSend()
                        if let Some(ref mut w) = writer {
                            let _ = w.write_record(&record).await;
                        }
                    }
                    s.xfiledataready = false;
                }
            }
        } else if !s.xplay {
            s.t_play = Duration::ZERO;
            s.dt_play = Duration::ZERO;
        }

        // Drop the lock to avoid starvation
        drop(s);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_state_default() {
        let state = AppState::default();
        assert_eq!(state.xplay, false);
        assert_eq!(state.xpause, false);
        assert_eq!(state.xmerge, true);
    }
}
