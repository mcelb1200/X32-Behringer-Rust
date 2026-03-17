use anyhow::{Result, anyhow};
use clap::Parser;
use midir::{Ignore, MidiInput};
use osc_lib::{OscArg, OscMessage};
use std::sync::Arc;
use std::sync::Mutex;

mod config;
mod rpn_calc;

use config::parse_file;
use rpn_calc::RpnCalculator;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// IP address of the X32 mixer.
    #[arg(short, long)]
    ip: String,

    /// Path to the .m2o command file.
    #[arg(short, long)]
    file: String,

    /// MIDI input port number (optional). If not specified, the first available port will be used.
    #[arg(short, long)]
    midi_in_port: Option<usize>,

    /// Print debug information for incoming MIDI messages.
    #[arg(short, long)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    println!("X32Midi2OSC - Rust Rewrite");

    // Connect to X32
    println!("Connecting to X32 at {}...", args.ip);
    let x32_socket = x32_lib::create_socket(&args.ip, 1000)?;
    println!("X32 Connected!");

    // Parse command file
    let commands = parse_file(&args.file)?;
    println!("Successfully parsed {} commands.", commands.len());

    let rpn_calc = Arc::new(Mutex::new(RpnCalculator::new()));

    // Create MIDI Input connection
    let mut midi_in = MidiInput::new("X32Midi2OSC Input")?;
    midi_in.ignore(Ignore::None);

    let in_ports = midi_in.ports();
    let in_port = match args.midi_in_port {
        Some(p) => in_ports
            .get(p)
            .ok_or_else(|| anyhow!("Invalid MIDI input port number: {}", p))?,
        None => in_ports
            .get(0)
            .ok_or_else(|| anyhow!("No MIDI input ports available"))?,
    };

    println!("Listening on MIDI input: {}", midi_in.port_name(in_port)?);

    let x32_socket = Arc::new(x32_socket);
    let debug = args.debug;

    let commands_arc = Arc::new(commands);

    // Run MIDI listener
    let _conn_in = midi_in
        .connect(
            in_port,
            "x32_midi2osc-read",
            move |_stamp, message, _| {
                // MIDI message parsing
                // message is typically 3 bytes: status, data1, data2
                // For some messages (like Program Change), it might be 2 bytes

                if message.is_empty() {
                    return;
                }
                let status = message[0];
                let data1 = if message.len() > 1 { message[1] } else { 0 };
                let data2 = if message.len() > 2 { message[2] } else { 0 };

                if debug {
                    println!("MIDI in: {:02X} {:02X} {:02X}", status, data1, data2);
                }

                // Only process channel messages (0x80 to 0xEF)
                if status >= 0x80 && status < 0xF0 {
                    let cmd_type = status & 0xF0;
                    let channel = (status & 0x0F) + 1;

                    // Match the C code logic for 'dwParam1'
                    // dwParam1 is a 32-bit int, containing status, data1, data2
                    // We'll reconstruct the match key:

                    // Check if it's a Program Change or Channel Aftertouch (0xC0 or 0xD0)
                    let actual_data2 = if cmd_type == 0xC0 || cmd_type == 0xD0 {
                        0x80 // Magic value used in C code for these commands
                    } else {
                        data2
                    };

                    let match_key = ((data1 as u32) << 8) | (status as u32);

                    let mparam = [channel as f64, data1 as f64, actual_data2 as f64];

                    for cmd in commands_arc.iter() {
                        if match_key == cmd.get_match_key() {
                            let osc_str = &cmd.osc_command;

                            // Execute OSC generation in a blocking context because we are in a sync callback
                            // It's a bit tricky but since the callback is sync, we can use try_lock
                            if let Ok(mut calc) = rpn_calc.try_lock() {
                                match process_osc_string(osc_str, &mut calc, &mparam) {
                                    Ok(msg) => {
                                        if debug {
                                            println!("Sending OSC: {:?}", msg);
                                        }
                                        match msg.to_bytes() {
                                            Ok(bytes) => {
                                                if let Err(e) = x32_socket.send(&bytes) {
                                                    eprintln!("Failed to send OSC message: {}", e);
                                                }
                                            }
                                            Err(e) => eprintln!("Failed to serialize OSC: {}", e),
                                        }
                                    }
                                    Err(e) => eprintln!("Error generating OSC: {}", e),
                                }
                            }

                            break;
                        }
                    }
                }
            },
            (),
        )
        .map_err(|e| anyhow!("Failed to connect to MIDI input port: {}", e))?;

    println!("Press Ctrl+C to exit.");

    // Keep the main thread alive, send xremote periodically
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(9));
    let x32_socket_main = x32_lib::create_socket(&args.ip, 1000)?;
    let xremote_msg = b"/xremote\0\0\0\0";
    loop {
        interval.tick().await;
        if let Err(e) = x32_socket_main.send(xremote_msg) {
            eprintln!("Failed to send /xremote: {}", e);
        }
    }
}

fn process_osc_string(
    osc_str: &str,
    calc: &mut RpnCalculator,
    mparam: &[f64; 3],
) -> Result<OscMessage> {
    let mut parts = osc_str.split_whitespace();
    let path = parts
        .next()
        .ok_or_else(|| anyhow!("Empty OSC command"))?
        .to_string();

    let mut args = Vec::new();
    let mut type_tags = String::new();

    let mut next_part = parts.next();
    if let Some(t) = next_part {
        if t.starts_with(',') {
            type_tags = t[1..].to_string();
            next_part = parts.next();
        }
    }

    let mut remaining = String::new();
    if let Some(n) = next_part {
        remaining.push_str(n);
        for p in parts {
            remaining.push(' ');
            remaining.push_str(p);
        }
    }

    let mut exprs = Vec::new();
    let mut in_bracket = false;
    let mut current_expr = String::new();
    let mut current_const = String::new();

    for c in remaining.chars() {
        if c == '[' {
            in_bracket = true;
            current_expr.clear();

            let trimmed = current_const.trim();
            if !trimmed.is_empty() {
                for token in trimmed.split_whitespace() {
                    exprs.push(token.to_string());
                }
            }
            current_const.clear();
        } else if c == ']' {
            in_bracket = false;
            exprs.push(format!("[{}]", current_expr));
        } else if in_bracket {
            current_expr.push(c);
        } else {
            current_const.push(c);
        }
    }

    let trimmed = current_const.trim();
    if !trimmed.is_empty() {
        for token in trimmed.split_whitespace() {
            exprs.push(token.to_string());
        }
    }

    let mut expr_idx = 0;
    for t in type_tags.chars() {
        if expr_idx >= exprs.len() {
            return Err(anyhow!("Not enough arguments for type tags"));
        }
        let expr = &exprs[expr_idx];

        let val = if expr.starts_with('[') && expr.ends_with(']') {
            let inner = &expr[1..expr.len() - 1];
            calc.calculate(inner, mparam)?
        } else {
            expr.parse::<f64>()
                .map_err(|e| anyhow!("Failed to parse constant: {}", e))?
        };

        match t {
            'i' => args.push(OscArg::Int(val as i32)),
            'f' => args.push(OscArg::Float(val as f32)),
            's' => args.push(OscArg::String(val.to_string())),
            _ => return Err(anyhow!("Unsupported OSC type tag: {}", t)),
        }
        expr_idx += 1;
    }

    Ok(OscMessage { path, args })
}
