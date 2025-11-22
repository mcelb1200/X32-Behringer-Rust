use anyhow::{anyhow, Context, Result};
use clap::Parser;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write, BufReader, BufWriter};
use std::sync::{Arc, Mutex};
use tokio::net::UdpSocket;
use tokio::time::{self, Duration, Instant};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt}; // C code uses system endianness (usually Little on x86)
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "192.168.0.64")]
    ip: String,
    #[arg(short, long, default_value = "X32ReplayFile.bin")]
    file: String,
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Mode {
    Idle,
    Recording,
    Playing,
    Paused,
}

struct AppState {
    mode: Mode,
    file_path: String,
    start_time: Option<Instant>,
    last_play_time: Option<Duration>, // Relative time in file
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.connect(format!("{}:10023", args.ip)).await?;
    let socket = Arc::new(socket);

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
    let mut line = String::new();
    loop {
        line.clear();
        if stdin.read_line(&mut line).is_err() { break; }
        let cmd = line.trim();

        let mut s = state.lock().unwrap();
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

async fn run_logic(state: Arc<Mutex<AppState>>, socket: Arc<UdpSocket>, default_file: String) {
    let mut buf = [0u8; 2048];
    let mut last_xremote = Instant::now();
    let mut file_writer: Option<BufWriter<File>> = None;
    let mut file_reader: Option<BufReader<File>> = None;

    // Subscribe
    let _ = socket.send(b"/info\0\0\0,"); // Simple manual packet or use osc_lib

    loop {
        let mode = { state.lock().unwrap().mode };

        match mode {
            Mode::Recording => {
                // Ensure file is open
                if file_writer.is_none() {
                    match File::create(&default_file) {
                        Ok(f) => file_writer = Some(BufWriter::new(f)),
                        Err(e) => { eprintln!("Failed to create file: {}", e); continue; }
                    }
                }

                // Send /xremote keepalive
                if last_xremote.elapsed() > Duration::from_secs(9) {
                    let _ = socket.send(b"/xremote\0\0\0\0,"); // Padding needed? osc_lib better.
                    last_xremote = Instant::now();
                }

                // Recv with timeout
                if let Ok(Ok(len)) = time::timeout(Duration::from_millis(100), socket.recv(&mut buf)).await {
                    // Write timestamp + len + data
                    if let Some(w) = &mut file_writer {
                        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                        let _ = w.write_u64::<LittleEndian>(now.as_secs());
                        let _ = w.write_u32::<LittleEndian>(now.subsec_micros());
                        let _ = w.write_u32::<LittleEndian>(len as u32);
                        let _ = w.write_all(&buf[..len]);
                        let _ = w.flush();
                    }
                }
            }
            Mode::Playing => {
                // Ensure reader open
                if file_reader.is_none() {
                     match File::open(&default_file) {
                        Ok(f) => {
                            file_reader = Some(BufReader::new(f));
                            let mut s = state.lock().unwrap();
                            s.start_time = None; // Reset timing
                        }
                        Err(e) => { eprintln!("Failed to open file: {}", e); continue; }
                    }
                }

                if let Some(r) = &mut file_reader {
                    match r.read_u64::<LittleEndian>() {
                        Ok(sec) => {
                            let usec = r.read_u32::<LittleEndian>().unwrap_or(0);
                            let len = r.read_u32::<LittleEndian>().unwrap_or(0);

                            if len > 0 && len < 2048 {
                                let mut data = vec![0u8; len as usize];
                                if r.read_exact(&mut data).is_ok() {
                                    // Timing Logic
                                    let packet_time = Duration::from_secs(sec) + Duration::from_micros(usec as u64);

                                    let sleep_dur = {
                                        let mut s = state.lock().unwrap();
                                        if s.start_time.is_none() {
                                            // First packet defines t0
                                            s.start_time = Some(Instant::now());
                                            s.last_play_time = Some(packet_time);
                                        }

                                        if let (Some(start), Some(first_packet_time)) = (s.start_time, s.last_play_time) {
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
                            let mut s = state.lock().unwrap();
                            s.mode = Mode::Idle;
                            s.start_time = None;
                            file_reader = None;
                        }
                    }
                }
            }
            Mode::Idle | Mode::Paused => {
                file_writer = None;
                file_reader = None;
                time::sleep(Duration::from_millis(100)).await;
            }
        }
    }
}
