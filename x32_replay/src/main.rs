//! `x32_replay` is a command-line utility for recording and replaying OSC traffic to/from an X32 mixer.
//!
//! It can:
//! - **Record**: Capture all incoming OSC messages from the mixer to a binary file, preserving timing.
//! - **Play**: Replay a recorded file back to the mixer, respecting the original timing intervals.
//!
//! This is useful for diagnosing issues, creating regression tests, or automating repetitive tasks.
//!
//! # Credits
//!
//! *   **Original concept and work on the C library:** Patrick-Gilles Maillot
//! *   **Additional concepts by:** mcelb1200
//! *   **Rust implementation by:** mcelb1200

use anyhow::Result;
use clap::Parser;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::UdpSocket;
use tokio::time::{self, Duration, Instant};

/// Command-line arguments for `x32_replay`.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// IP address of the X32 console.
    #[arg(short, long, default_value = "192.168.0.64")]
    ip: String,
    /// File to record to or play from.
    #[arg(short, long, default_value = "X32ReplayFile.bin")]
    file: String,
    /// Enable verbose output.
    #[arg(short, long)]
    verbose: bool,
}

/// Represents the current operating mode of the application.
#[derive(Debug, PartialEq, Clone, Copy)]
enum Mode {
    /// Waiting for user input.
    Idle,
    /// Recording incoming OSC messages to file.
    Recording,
    /// Replaying messages from file to mixer.
    Playing,
    /// Playback paused.
    Paused,
}

/// Shared application state.
struct AppState {
    mode: Mode,
    #[allow(dead_code)]
    file_path: String,
    start_time: Option<Instant>,
    last_play_time: Option<Duration>, // Relative time in file
}

/// The main entry point for the application.
#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.connect(format!("{}:10023", args.ip)).await?;
    let socket = Arc::new(socket);

    if args.verbose {
        println!("Verbose mode enabled.");
    }

    println!("X32Replay connected to {}.", args.ip);
    println!("Commands: record, play, stop, pause, exit");

    let state = Arc::new(Mutex::new(AppState {
        mode: Mode::Idle,
        file_path: args.file.clone(),
        start_time: None,
        last_play_time: None,
    }));

    // Background task for logic
    let state_clone = state.clone();
    let socket_clone = socket.clone();
    let file_path = args.file.clone();

    tokio::spawn(async move {
        run_logic(state_clone, socket_clone, file_path).await;
    });

    // Stdin loop
    let stdin = std::io::stdin();
    let mut stdin_lock = stdin.lock();
    let mut line = String::new();
    loop {
        line.clear();
        use std::io::{BufRead, Read};
        if stdin_lock.by_ref().take(4096).read_line(&mut line).is_err() || line.is_empty() {
            break;
        }
        let cmd = line.trim();

        let mut s = match state.lock() {
            Ok(guard) => guard,
            Err(_) => {
                eprintln!("State mutex poisoned, exiting.");
                break;
            }
        };
        match cmd {
            "exit" => break,
            "stop" => {
                s.mode = Mode::Idle;
                println!("Stopped.");
            }
            "record" => {
                s.mode = Mode::Recording;
                println!("Recording...");
            }
            "play" => {
                s.mode = Mode::Playing;
                println!("Playing...");
            }
            "pause" => {
                s.mode = Mode::Paused;
                println!("Paused.");
            }
            _ => println!("Unknown command."),
        }
    }

    Ok(())
}

/// The core logic loop handling recording and playback.
///
/// This function runs in a background task and switches behavior based on the `AppState`.
/// - **Recording**: Captures packets from UDP, timestamps them, and writes to file.
/// - **Playing**: Reads packets from file, sleeps for the correct duration, and sends to UDP.
async fn run_logic(state: Arc<Mutex<AppState>>, socket: Arc<UdpSocket>, default_file: String) {
    let mut buf = [0u8; 2048];
    let mut last_xremote = Instant::now();
    let mut file_writer: Option<BufWriter<File>> = None;
    let mut file_reader: Option<BufReader<tokio::io::Take<File>>> = None;

    // Subscribe
    // Use proper OSC message construction or explicit bytes.
    // Assuming simple bytes are intended here for minimal overhead or legacy reasons.
    if let Err(e) = socket.send(b"/info\0\0\0,").await {
        eprintln!("Failed to send subscription: {}", e);
    }

    loop {
        let mode = match state.lock() {
            Ok(s) => s.mode,
            Err(_) => {
                eprintln!("State mutex poisoned in background task, exiting.");
                break;
            }
        };

        match mode {
            Mode::Recording => {
                // Ensure file is open
                if file_writer.is_none() {
                    match File::create(&default_file).await {
                        Ok(f) => file_writer = Some(BufWriter::new(f)),
                        Err(e) => {
                            eprintln!("Failed to create file: {}", e);
                            continue;
                        }
                    }
                }

                // Send /xremote keepalive
                if last_xremote.elapsed() > Duration::from_secs(9) {
                    if let Err(e) = socket.send(b"/xremote\0\0\0\0,").await {
                        eprintln!("Failed to send keepalive: {}", e);
                    }
                    last_xremote = Instant::now();
                }

                // Recv with timeout
                if let Ok(Ok(len)) =
                    time::timeout(Duration::from_millis(100), socket.recv(&mut buf)).await
                {
                    // Write timestamp + len + data
                    if let Some(w) = &mut file_writer {
                        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                        let _ = w.write_u64_le(now.as_secs()).await;
                        let _ = w.write_u32_le(now.subsec_micros()).await;
                        let _ = w.write_u32_le(len as u32).await;
                        let _ = w.write_all(&buf[..len]).await;
                        // OPTIMIZATION: Removed `.flush().await` in this hot loop to allow `BufWriter` to
                        // actually buffer writes, significantly reducing I/O syscall overhead during recording.
                    }
                }
            }
            Mode::Playing => {
                // Ensure reader open
                if file_reader.is_none() {
                    match File::open(&default_file).await {
                        Ok(f) => {
                            // Check file size to prevent OOM / DoS from reading huge invalid files
                            if let Ok(metadata) = f.metadata().await {
                                if metadata.len() > 10 * 1024 * 1024 {
                                    eprintln!("File too large to replay (max 10MB)");
                                    if let Ok(mut s) = state.lock() {
                                        s.mode = Mode::Idle;
                                    }
                                    continue;
                                }
                            }
                            let f = f.take(10 * 1024 * 1024);
                            file_reader = Some(BufReader::new(f));
                            if let Ok(mut s) = state.lock() {
                                s.start_time = None; // Reset timing
                            } else {
                                eprintln!("State mutex poisoned in background task, exiting.");
                                break;
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to open file: {}", e);
                            continue;
                        }
                    }
                }

                if let Some(r) = &mut file_reader {
                    match r.read_u64_le().await {
                        Ok(sec) => {
                            let usec = r.read_u32_le().await.unwrap_or(0);
                            let len = r.read_u32_le().await.unwrap_or(0);

                            if len > 0 && len < 2048 {
                                let mut data = vec![0u8; len as usize];
                                if r.read_exact(&mut data).await.is_ok() {
                                    // Timing Logic
                                    let packet_time = Duration::from_secs(sec)
                                        + Duration::from_micros(usec as u64);

                                    let sleep_dur = {
                                        let mut s = match state.lock() {
                                            Ok(guard) => guard,
                                            Err(_) => {
                                                eprintln!(
                                                    "State mutex poisoned in background task, exiting."
                                                );
                                                break;
                                            }
                                        };
                                        if s.start_time.is_none() {
                                            // First packet defines t0
                                            s.start_time = Some(Instant::now());
                                            s.last_play_time = Some(packet_time);
                                        }

                                        if let (Some(start), Some(first_packet_time)) =
                                            (s.start_time, s.last_play_time)
                                        {
                                            if packet_time > first_packet_time {
                                                let delta = packet_time - first_packet_time;
                                                let target_time = start + delta;
                                                let now = Instant::now();
                                                if target_time > now {
                                                    Some(target_time - now)
                                                } else {
                                                    None
                                                }
                                            } else {
                                                None
                                            }
                                        } else {
                                            None
                                        }
                                    };

                                    if let Some(dur) = sleep_dur {
                                        time::sleep(dur).await;
                                    }

                                    let _ = socket.send(&data).await;
                                }
                            }
                        }
                        Err(_) => {
                            println!("End of file.");
                            if let Ok(mut s) = state.lock() {
                                s.mode = Mode::Idle;
                                s.start_time = None;
                            } else {
                                eprintln!("State mutex poisoned in background task, exiting.");
                                break;
                            }
                            file_reader = None;
                        }
                    }
                }
            }
            Mode::Idle | Mode::Paused => {
                if let Some(mut w) = file_writer.take() {
                    let _ = w.flush().await;
                }
                file_reader = None;
                time::sleep(Duration::from_millis(100)).await;
            }
        }
    }
}
