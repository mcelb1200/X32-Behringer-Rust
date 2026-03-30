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
                    let s_buf = String::from_utf8_lossy(data);

                    // Parse User Bank inputs (buttons mapped to play/stop/rew/etc)
                    if s_buf.starts_with("/-stat/userpar/") && s_buf.len() >= 17 {
                        let mut lock = state.lock().await;

                        if let Ok(bnum) = s_buf[15..17].parse::<u32>() {
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

            // Only update dt_play if not in catch-up/back (which override dt_play for tests)
            if s.t_rew.is_zero() && s.t_ff.is_zero() {
                s.dt_play = now.saturating_sub(s.t_play);
            }

            if s.xfiledataready {
                if s.dt_play > s.dt_read {
                    if let Some(record) = current_record.take() {
                        let mut should_send = true;
                        if s.xmerge {
                            if s.xmergefaders {
                                let data_str = String::from_utf8_lossy(&record.data);
                                if data_str.contains("fader") {
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

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();

        // We will execute long-running catch-up/catch-back outside the lock
        let mut do_rew = false;
        let mut do_ff = false;
        let xcatchdelay = s.xcatchdelay.max(0) as u64;

        if !s.t_rew.is_zero() && now > s.t_rew {
            do_rew = true;
            s.t_rew = Duration::ZERO;
        } else if !s.t_ff.is_zero() && now > s.t_ff {
            do_ff = true;
            s.t_ff = Duration::ZERO;
        }

        // Drop the lock to avoid starvation during sleep/IO
        drop(s);

        if do_rew {
            println!("Executing catch-back logic (REW)...");

            // Re-open reading file and newly created file up to this point
            // For simplicity, we just reset the reader to the start and replay
            if let Some(ref path) = file_path {
                // To avoid deadlocks we need a quick way to read/write state without holding it indefinitely
                if let Ok(f) = File::open(path).await {
                    reader = Some(PunchReader::new(f));
                    let mut s = state.lock().await;
                    s.xreadfile = true;
                    s.dt_read = Duration::ZERO;
                }

                let out_path = format!("{}_xpc", path);
                let backup_path = format!("{}_xpc_backup", path);

                // Rename current writer output
                writer = None; // drop current writer
                let rename_res = tokio::fs::rename(&out_path, &backup_path).await;
                println!("Rename res: {:?}", rename_res);

                if let Ok(f) = File::create(&out_path).await {
                    writer = Some(PunchWriter::new(f));
                }

                if let Ok(backup_f) = File::open(&backup_path).await {
                    let mut backup_reader = PunchReader::new(backup_f);

                    if let Some(ref mut r) = reader {
                        loop {
                            let (xreadfile, dt_play, dt_read) = {
                                let s = state.lock().await;
                                (s.xreadfile, s.dt_play, s.dt_read)
                            };

                            // The C code uses: while (Xreadfile  && (timercmp(&dt_play, &dt_read, >)))
                            // but in catch-back dt_read starts at 0 and goes up.
                            if !xreadfile || dt_play <= dt_read {
                                break;
                            }

                            // Read from both original and backup (ex-writer)
                            // The C logic reads both but uses backup data to write
                            let backup_time;
                            match backup_reader.read_record().await {
                                Ok(Some(b_record)) => {
                                    backup_time = b_record.time;
                                    let _ = socket.send(&b_record.data).await;
                                    if let Some(ref mut w) = writer {
                                        let _ = w.write_record(&b_record).await;
                                    }
                                    if xcatchdelay > 0 {
                                        time::sleep(Duration::from_millis(xcatchdelay)).await;
                                    }
                                }
                                _ => {
                                    let mut s = state.lock().await;
                                    s.xreadfile = false;
                                    break;
                                }
                            }

                            let mut current_dt_read = {
                                let s = state.lock().await;
                                s.dt_read
                            };

                            // Advance the original reader to match the backup_time
                            // This ensures dt_read catches up to the current replayed time
                            if current_dt_read <= backup_time {
                                if let Ok(Some(record)) = r.read_record().await {
                                    let mut s = state.lock().await;
                                    s.dt_read = record.time;
                                    current_dt_read = record.time;
                                }
                            }

                            if backup_time >= dt_play || current_dt_read >= dt_play {
                                break;
                            }
                        }
                    }
                }
                let _ = tokio::fs::remove_file(&backup_path).await;
            }
            let mut s = state.lock().await;
            s.xfiledataready = false;
            current_record = None;
        }

        if do_ff {
            println!("Executing catch-up logic (FF)...");
            // Implement XCatchUpProc behavior: Read and send rapidly until catching up
            if let Some(ref mut r) = reader {
                loop {
                    let (xreadfile, dt_play, dt_read) = {
                        let s = state.lock().await;
                        (s.xreadfile, s.dt_play, s.dt_read)
                    };

                    // The C code uses: while (Xreadfile  && (timercmp(&dt_play, &dt_read, >)))
                    if !xreadfile || dt_play <= dt_read {
                        break;
                    }

                    if let Ok(Some(record)) = r.read_record().await {
                        {
                            let mut s = state.lock().await;
                            s.dt_read = record.time;
                        }
                        let _ = socket.send(&record.data).await;
                        if let Some(ref mut w) = writer {
                            let _ = w.write_record(&record).await;
                        }
                        // Introduce a small delay to help fader moves
                        if xcatchdelay > 0 {
                            time::sleep(Duration::from_millis(xcatchdelay)).await;
                        }
                    } else {
                        let mut s = state.lock().await;
                        s.xreadfile = false;
                        s.xfiledataready = false;
                        current_record = None;
                        break;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
