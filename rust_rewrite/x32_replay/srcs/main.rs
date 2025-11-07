use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Write};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};

/// A simple record & playback service for X32
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// X32 console IP address
    #[arg(short, long, default_value = "127.0.0.1")]
    ip: String,

    /// Verbose mode
    #[arg(short, long, default_value_t = false)]
    verbose: bool,

    /// File name for recording/playing
    #[arg(short, long, default_value = "X32ReplayFile.bincode")]
    file: String,
}

/// Represents the type of event in a replay file.
#[derive(Serialize, Deserialize, Debug)]
enum EventType {
    /// An OSC message.
    Osc(Vec<u8>),
    /// A user comment.
    Comment(String),
}

/// Represents a single event in a replay file, with a timestamp.
#[derive(Serialize, Deserialize, Debug)]
struct ReplayEvent {
    /// The timestamp of the event.
    timestamp: DateTime<Utc>,
    /// The type of the event.
    event: EventType,
}

/// Represents the current state of the application.
enum AppState {
    /// The application is idle.
    Idle,
    /// The application is recording.
    Recording,
    /// The application is playing back a recording.
    Playing,
    /// The application has paused a recording.
    RecordPaused,
}

fn main() -> Result<()> {
    let args = Args::parse();
    println!("X32 Replay - v0.1.0");

    // Establish connection with the X32
    println!("Connecting to X32 at {}...", args.ip);
    let socket = Arc::new(x32_lib::create_socket(&args.ip, 10)?);
    socket.send(b"/info\x00\x00\x00")?;

    let mut buf = [0; 512];
    match socket.recv(&mut buf) {
        Ok(_) => {
            println!("Connected!");
        }
        Err(e) => {
            eprintln!("Failed to connect to X32: {}", e);
            return Ok(());
        }
    }

    let app_state = Arc::new(Mutex::new(AppState::Idle));
    let mut last_xremote = Instant::now();
    let mut file: Option<BufWriter<File>> = None;
    let mut replay_file: Option<BufReader<File>> = None;
    let mut next_event: Option<ReplayEvent> = None;
    let mut start_time = Instant::now();
    let mut first_timestamp = Utc::now();

    println!("Enter 'help' for a list of commands.");
    loop {
        let mut state = app_state.lock().unwrap();
        match *state {
            AppState::Recording => {
                if file.is_none() {
                    println!("Opening file for recording...");
                    let f = OpenOptions::new()
                        .write(true)
                        .create(true)
                        .truncate(true)
                        .open(&args.file)?;
                    file = Some(BufWriter::new(f));
                }

                if last_xremote.elapsed() > Duration::from_secs(9) {
                    println!("Sending /xremote...");
                    socket.send(b"/xremote\x00\x00\x00")?;
                    last_xremote = Instant::now();
                }

                let mut buf = [0; 512];
                if let Ok(len) = socket.recv(&mut buf) {
                    println!("Recording message: {:?}", &buf[..len]);
                    let event = ReplayEvent {
                        timestamp: Utc::now(),
                        event: EventType::Osc(buf[..len].to_vec()),
                    };
                    bincode::serialize_into(file.as_mut().unwrap(), &event)?;
                }
            }
            AppState::Playing => {
                if replay_file.is_none() {
                    println!("Opening file for playback...");
                    let f = File::open(&args.file)?;
                    replay_file = Some(BufReader::new(f));
                    start_time = Instant::now();
                    if let Ok(event) = bincode::deserialize_from(replay_file.as_mut().unwrap()) {
                         let first_event: ReplayEvent = event;
                         first_timestamp = first_event.timestamp;
                         next_event = Some(first_event);
                    }
                }

                if let Some(event) = next_event.take() {
                    let offset = event.timestamp.signed_duration_since(first_timestamp);
                    if start_time.elapsed() >= offset.to_std()? {
                        match event.event {
                            EventType::Osc(msg) => {
                                println!("Playing message: {:?}", msg);
                                socket.send(&msg)?;
                            }
                            EventType::Comment(comment) => {
                                println!("{}", comment);
                            }
                        }
                        if let Ok(next) = bincode::deserialize_from(replay_file.as_mut().unwrap()) {
                            next_event = Some(next);
                        } else {
                            println!("Playback finished.");
                            *state = AppState::Idle;
                        }
                    } else {
                        next_event = Some(event);
                    }
                }
            }
            _ => {
                if file.is_some() {
                    file.take().unwrap().flush()?;
                }
                if replay_file.is_some() {
                    replay_file = None;
                }
            }
        }

        print!("> ");
        io::stdout().flush()?;
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_ok() {
            let cmd = input.trim();
            match cmd {
                "exit" => break,
                "record on" => {
                    println!("Recording started.");
                    *state = AppState::Recording
                },
                "record off" => {
                    println!("Recording stopped.");
                    *state = AppState::Idle
                },
                "record pause" => {
                    println!("Recording paused.");
                    *state = AppState::RecordPaused
                },
                "play on" => {
                    println!("Playback started.");
                    *state = AppState::Playing
                },
                "play off" => {
                    println!("Playback stopped.");
                    *state = AppState::Idle
                },
                _ => {
                    if cmd.starts_with('#') {
                        if let AppState::Recording = *state {
                            let event = ReplayEvent {
                                timestamp: Utc::now(),
                                event: EventType::Comment(cmd.to_string()),
                            };
                            bincode::serialize_into(file.as_mut().unwrap(), &event)?;
                        }
                    } else {
                        println!("Unknown command: {}", cmd)
                    }
                }
            }
        }
    }

    println!("Exiting.");
    Ok(())
}
