//! `x32_punch_control` is a utility to manage DAW punch IN/OUT down mixing updates for the Behringer X32.
//! This is a Rust rewrite of the original C program by Patrick-Gilles Maillot.

use anyhow::Result;
use clap::Parser;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::net::UdpSocket;
use std::thread;
use std::time::{Duration, Instant};
use x32_lib::{create_socket, common};
use osc_lib::{OscMessage, OscArg};
use portmidi::{PortMidi, InputPort, MidiMessage};

/// A utility to manage DAW punch IN/OUT down mixing updates for the Behringer X32.
/// This is a Rust rewrite of the original C program by Patrick-Gilles Maillot.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// IP address of the X32 mixer
    #[arg(short, long)]
    ip: String,

    /// Input file to read automation data from (*.xpc)
    #[arg(short = 'f', long)]
    input_file: Option<String>,

    /// Output file to write automation data to (*.xpc)
    #[arg(short, long)]
    output_file: String,

    /// Scene number to load on the X32
    #[arg(short, long)]
    scene: Option<u8>,

    /// MIDI input port number
    #[arg(long)]
    midi_in: Option<usize>,

    /// MIDI output port number
    #[arg(long)]
    midi_out: Option<usize>,

    /// Enable and specify which user bank to use for hardware controls (A, B, or C)
    #[arg(long)]
    use_bank: Option<char>,

    /// Use MIDI Time Code for synchronization
    #[arg(long)]
    mtc: bool,

    /// Record button implies Play
    #[arg(long)]
    record_implies_play: bool,

    /// When merging, only protect faders from being overwritten by file data
    #[arg(long)]
    merge_faders_only: bool,

    /// Delay in milliseconds used during catch-up/catch-back operations
    #[arg(long, default_value_t = 10)]
    catch_delay: u64,
}

/// Represents the current state of the application.
#[derive(Debug, Default)]
struct AppState {
    is_playing: bool,
    is_recording: bool,
    is_punch_in: bool,
    is_paused: bool,
    is_merge_on: bool,
    start_time: Option<Instant>,
    pause_time: Option<Instant>,
    mtc_time: Duration,
    mtc_hours: u8,
    mtc_mins: u8,
    mtc_secs: u8,
    mtc_frames: u8,
    mtc_frame_rate_idx: u8,
}

/// Represents a single record in the `.xpc` automation file.
#[derive(Debug)]
struct PunchControlRecord {
    timestamp: Duration,
    data: Vec<u8>,
}

impl PunchControlRecord {
    /// Reads a single `PunchControlRecord` from a reader.
    fn from_reader(reader: &mut impl Read) -> Result<Option<Self>> {
        let mut secs_buf = [0u8; 8];
        let mut nanos_buf = [0u8; 4];
        let mut len_buf = [0u8; 4];

        if reader.read_exact(&mut secs_buf).is_err() {
            return Ok(None);
        }
        if reader.read_exact(&mut nanos_buf).is_err() {
            return Ok(None);
        }
        if reader.read_exact(&mut len_buf).is_err() {
            return Ok(None);
        }

        let secs = u64::from_be_bytes(secs_buf);
        let nanos = u32::from_be_bytes(nanos_buf);
        let len = u32::from_be_bytes(len_buf) as usize;

        let mut data = vec![0u8; len];
        reader.read_exact(&mut data)?;

        Ok(Some(PunchControlRecord {
            timestamp: Duration::new(secs, nanos),
            data,
        }))
    }

    /// Writes the `PunchControlRecord` to a writer.
    fn write(&self, writer: &mut impl Write) -> Result<()> {
        writer.write_all(&self.timestamp.as_secs().to_be_bytes())?;
        writer.write_all(&self.timestamp.subsec_nanos().to_be_bytes())?;
        writer.write_all(&(self.data.len() as u32).to_be_bytes())?;
        writer.write_all(&self.data)?;
        Ok(())
    }
}

/// Initializes the user control buttons on the X32 for the specified bank.
fn init_user_controls(socket: &UdpSocket, bank: char) -> Result<()> {
    let bank_id = (bank as u8) - b'A';
    for i in 0..8 {
        let button_id = bank_id * 8 + i + 5;
        let msg = OscMessage::new(
            format!("/config/userctrl/{}/btn/{}", bank, button_id),
            vec![OscArg::String(format!("MN1600{}", i))],
        );
        socket.send(&msg.to_bytes()?)?;
    }
    let msg = OscMessage::new(
        format!("/config/userctrl/{}/color", bank),
        vec![OscArg::Int(common::Color::White as i32)],
    );
    socket.send(&msg.to_bytes()?)?;

    let msg = OscMessage::new(
        "/-stat/userbank".to_string(),
        vec![OscArg::Int(bank_id as i32)],
    );
    socket.send(&msg.to_bytes()?)?;

    Ok(())
}

/// Handles incoming MIDI messages and updates the application state.
fn handle_midi_message(message: MidiMessage, state: &mut AppState) {
    match message.status {
        0xF1 => { // MTC Quarter Frame
            let data = message.data1;
            let msg_type = (data >> 4) & 0x07;
            let value = data & 0x0F;

            match msg_type {
                0 => state.mtc_frames = (state.mtc_frames & 0xF0) | value,
                1 => state.mtc_frames = (state.mtc_frames & 0x0F) | (value << 4),
                2 => state.mtc_secs = (state.mtc_secs & 0xF0) | value,
                3 => state.mtc_secs = (state.mtc_secs & 0x0F) | (value << 4),
                4 => state.mtc_mins = (state.mtc_mins & 0xF0) | value,
                5 => state.mtc_mins = (state.mtc_mins & 0x0F) | (value << 4),
                6 => state.mtc_hours = (state.mtc_hours & 0xF0) | value,
                7 => {
                    state.mtc_hours = (state.mtc_hours & 0x0F) | ((value & 0x01) << 4);
                    state.mtc_frame_rate_idx = (value >> 1) & 0x03;
                }
                _ => {}
            }

            let frame_rates_micros = [41666, 40000, 33366, 33333];
            let micros_per_frame = frame_rates_micros[state.mtc_frame_rate_idx as usize];
            let total_secs = state.mtc_hours as u64 * 3600 +
                             state.mtc_mins as u64 * 60 +
                             state.mtc_secs as u64;
            let total_micros = state.mtc_frames as u64 * micros_per_frame;

            state.mtc_time = Duration::from_secs(total_secs) + Duration::from_micros(total_micros);
        }
        0xFA => { // Start
            state.is_playing = true;
            state.start_time = Some(Instant::now());
            println!("MIDI Start received");
        }
        0xFB => { // Continue
            state.is_paused = false;
            if let (Some(start), Some(pause)) = (state.start_time, state.pause_time) {
                let pause_duration = pause.elapsed();
                state.start_time = Some(start + pause_duration);
                state.pause_time = None;
            }
            println!("MIDI Continue received");
        }
        0xFC => { // Stop
            state.is_playing = false;
            state.is_paused = false;
            state.is_recording = false;
            state.is_punch_in = false;
            println!("MIDI Stop received");
        }
        _ => {}
    }
}


fn main() -> Result<()> {
    let args = Args::parse();
    println!("X32 Punch Control starting with args: {:?}", args);

    let socket = create_socket(&args.ip, 10)?;
    println!("Connected to X32 at {}", args.ip);

    if let Some(bank) = args.use_bank {
        init_user_controls(&socket, bank)?;
        println!("Initialized user controls on bank {}", bank);
    }

    let mut input_file = if let Some(path) = &args.input_file {
        Some(File::open(path)?)
    } else {
        None
    };

    let mut output_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&args.output_file)?;

    println!("Input file: {:?}, Output file: {}", args.input_file, args.output_file);

    let pm = PortMidi::new()?;
    let mut in_port: Option<InputPort> = if let Some(port_id) = args.midi_in {
        let devices = pm.devices()?;
        let device = devices.get(port_id).ok_or_else(|| anyhow::anyhow!("Invalid MIDI Input port"))?;
        println!("Connecting to MIDI input: {}", device.name());
        Some(pm.input_port(device.clone(), 1024)?)
    } else {
        None
    };

    let mut state = AppState::default();
    state.is_merge_on = true;

    let mut next_record: Option<PunchControlRecord> = None;
    if let Some(ref mut f) = input_file {
        next_record = PunchControlRecord::from_reader(f)?;
    }

    // Main loop
    loop {
        if let Some(ref mut port) = in_port {
            if let Ok(Some(events)) = port.read_n(1024) {
                for event in events {
                    handle_midi_message(event.message, &mut state);
                }
            }
        }

        let elapsed = if args.mtc {
            state.mtc_time
        } else if let Some(start_time) = state.start_time {
            if let Some(pause_time) = state.pause_time {
                pause_time.duration_since(start_time)
            } else {
                start_time.elapsed()
            }
        } else {
            Duration::from_secs(0)
        };

        // Playback from input file
        if state.is_playing && !state.is_paused {
             if let Some(record) = &next_record {
                if elapsed >= record.timestamp {
                    let mut play_record = true;
                    if state.is_merge_on {
                        if args.merge_faders_only {
                            let msg = OscMessage::from_bytes(&record.data)?;
                            if msg.path.ends_with("/fader") {
                                play_record = false;
                            }
                        } else {
                            play_record = false;
                        }
                    }

                    if play_record {
                        socket.send(&record.data)?;
                    }

                    if let Some(ref mut f) = input_file {
                        next_record = PunchControlRecord::from_reader(f)?;
                    } else {
                        next_record = None;
                    }
                }
            }
        }

        // Recording from X32
        let mut buf = [0u8; 1024];
        if let Ok(len) = socket.recv(&mut buf) {
            let data = &buf[..len];
            let msg = OscMessage::from_bytes(data)?;

            let automatable = msg.path.starts_with("/ch/") ||
                              msg.path.starts_with("/auxin/") ||
                              msg.path.starts_with("/bus/") ||
                              msg.path.starts_with("/mtx/") ||
                              msg.path.starts_with("/dca/");

            if automatable {
                if let Some(bank) = args.use_bank {
                    if msg.path.starts_with("/-stat/userpar/") {
                        // This is a user control message, handle it
                        let parts: Vec<&str> = msg.path.split('/').collect();
                        if let (Some(bank_char), Some(btn_str)) = (parts.get(3), parts.get(4)) {
                            if bank_char.len() == 1 && bank_char.chars().next().unwrap() == bank {
                                if let Ok(btn_id) = btn_str.parse::<u8>() {
                                    if let Some(OscArg::Int(val)) = msg.args.get(0) {
                                        if *val == 1 {
                                            match btn_id {
                                                5 => { // REW
                                                    println!("REWIND");
                                                    state.start_time = Some(Instant::now());
                                                    if let Some(path) = &args.input_file {
                                                        input_file = Some(File::open(path)?);
                                                        if let Some(ref mut f) = input_file {
                                                            next_record = PunchControlRecord::from_reader(f)?;
                                                        }
                                                    }
                                                }
                                                6 => { // PLAY
                                                    state.is_playing = !state.is_playing;
                                                    if state.is_playing {
                                                        state.start_time = Some(Instant::now());
                                                    }
                                                }
                                                7 => { // PAUSE
                                                    state.is_paused = !state.is_paused;
                                                    if state.is_paused {
                                                        state.pause_time = Some(Instant::now());
                                                    } else {
                                                        if let (Some(start), Some(pause)) = (state.start_time, state.pause_time) {
                                                            let pause_duration = pause.elapsed();
                                                            state.start_time = Some(start + pause_duration);
                                                            state.pause_time = None;
                                                        }
                                                    }
                                                }
                                                8 => { // FF
                                                    println!("FAST FORWARD");
                                                    if let Some(start) = state.start_time {
                                                        state.start_time = Some(start - Duration::from_secs(10));
                                                    }
                                                }
                                                9 => { // PUNCH IN
                                                    state.is_punch_in = !state.is_punch_in;
                                                }
                                                10 => { // MERGE
                                                    state.is_merge_on = !state.is_merge_on;
                                                }
                                                11 => { // STOP
                                                    state.is_playing = false;
                                                    state.is_recording = false;
                                                    state.is_punch_in = false;
                                                    state.start_time = None;
                                                }
                                                12 => { // RECORD
                                                    state.is_recording = !state.is_recording;
                                                    if args.record_implies_play {
                                                        state.is_playing = true;
                                                        if state.start_time.is_none() {
                                                            state.start_time = Some(Instant::now());
                                                        }
                                                    }
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if state.is_recording && (state.is_punch_in || !args.mtc) {
                    let record = PunchControlRecord {
                        timestamp: elapsed,
                        data: data.to_vec(),
                    };
                    record.write(&mut output_file)?;
                }
            }
        }

        // Send /xremote to keep the connection alive
        socket.send(b"/xremote\x00\x00\x00\x00")?;

        thread::sleep(Duration::from_millis(10));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_midi_message() {
        let mut state = AppState::default();

        // Test Start
        handle_midi_message(MidiMessage { status: 0xFA, data1: 0, data2: 0, data3: 0 }, &mut state);
        assert!(state.is_playing);
        assert!(state.start_time.is_some());

        // Test Stop
        handle_midi_message(MidiMessage { status: 0xFC, data1: 0, data2: 0, data3: 0 }, &mut state);
        assert!(!state.is_playing);

        // Test Continue
        state.is_paused = true;
        handle_midi_message(MidiMessage { status: 0xFB, data1: 0, data2: 0, data3: 0 }, &mut state);
        assert!(!state.is_paused);
    }
}
