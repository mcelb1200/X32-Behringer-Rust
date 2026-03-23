mod config;
mod rpn;

use anyhow::{Context, Result};
use clap::Parser;
use midir::{Ignore, MidiInput};
use osc_lib::{OscArg, OscMessage};
use rpn::RpnCalculator;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::time::{self, Duration, Instant};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// IP address of the X32 mixer
    #[arg(short, long, default_value = "")]
    ip: String,

    /// Path to the .m2o configuration file
    #[arg(short, long)]
    file: String,

    /// Name or partial name of the MIDI input port to use
    #[arg(short, long, default_value = "")]
    midi_in: String,
}

/// Helper function to evaluate an RPN expression from a template and write to a buffer
fn execute_template(
    template: &str,
    mparam: &[f64; 3],
    calculator: &mut RpnCalculator,
) -> Result<Vec<u8>> {
    let mut path = String::new();
    // OPTIMIZATION: Most OSC messages have <8 arguments. Pre-allocating capacity
    // prevents reallocation in this hot path triggered by high-frequency MIDI events.
    let mut type_tags = Vec::with_capacity(8);
    let mut args = Vec::with_capacity(8);

    let mut in_types = false;
    let mut in_expr = false;
    // Pre-allocate to prevent growth during loop construction
    let mut current_expr = String::with_capacity(32);

    // OPTIMIZATION: Avoid allocating an intermediate Vec<&str> using collect().
    // Instead, iterate directly over split_whitespace().
    let mut arg_idx = 0;

    for (i, part) in template.split_whitespace().enumerate() {
        if i == 0 {
            path = part.to_string();
            continue;
        }

        if !in_types && part.starts_with(',') {
            in_types = true;
            for tag in part.chars().skip(1) {
                type_tags.push(tag);
            }
            continue;
        }

        if let Some(stripped) = part.strip_prefix('[') {
            in_expr = true;
            current_expr = stripped.to_string();
            if current_expr.ends_with(']') {
                in_expr = false;
                current_expr.pop(); // remove ']'

                let val = calculator.evaluate(&current_expr, mparam)?;

                let current_tag = if arg_idx < type_tags.len() {
                    type_tags[arg_idx]
                } else {
                    'f'
                };

                if current_tag == 'i' {
                    args.push(OscArg::Int(val as i32));
                } else {
                    args.push(OscArg::Float(val as f32));
                }
                arg_idx += 1;
                current_expr.clear();
            }
        } else if in_expr {
            current_expr.push(' ');
            #[allow(clippy::manual_strip)]
            if part.ends_with(']') {
                in_expr = false;
                current_expr.push_str(&part[..part.len() - 1]);

                let val = calculator.evaluate(&current_expr, mparam)?;

                let current_tag = if arg_idx < type_tags.len() {
                    type_tags[arg_idx]
                } else {
                    'f'
                };

                if current_tag == 'i' {
                    args.push(OscArg::Int(val as i32));
                } else {
                    args.push(OscArg::Float(val as f32));
                }
                arg_idx += 1;
                current_expr.clear();
            } else {
                current_expr.push_str(part);
            }
        } else {
            // Not in an expression, just a normal argument
            // Determine type and parse
            let current_tag = if arg_idx < type_tags.len() {
                type_tags[arg_idx]
            } else {
                'f'
            };

            if current_tag == 'i' {
                if let Ok(v) = part.parse::<i32>() {
                    args.push(OscArg::Int(v));
                } else {
                    args.push(OscArg::Int(0));
                }
            } else if current_tag == 's' {
                args.push(OscArg::String(part.to_string()));
            } else {
                if let Ok(v) = part.parse::<f32>() {
                    args.push(OscArg::Float(v));
                } else {
                    args.push(OscArg::Float(0.0));
                }
            }
            arg_idx += 1;
        }
    }

    let msg = OscMessage::new(path, args);
    msg.to_bytes()
        .map_err(|e| anyhow::anyhow!("Failed to serialize OscMessage: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_template() {
        let mut calc = RpnCalculator::new();
        let mparam = [1.0, 12.0, 63.5];

        let template = "/ch/01/mix/fader ,f [$2 127.0 /]";
        let out = execute_template(template, &mparam, &mut calc).unwrap();

        let msg = OscMessage::from_bytes(&out).unwrap();
        assert_eq!(msg.path, "/ch/01/mix/fader");
        assert_eq!(msg.args.len(), 1);
        if let OscArg::Float(f) = msg.args[0] {
            assert_eq!(f, 0.5); // 63.5 / 127 = 0.5
        } else {
            panic!("Expected Float argument");
        }
    }

    #[test]
    fn test_execute_template_int() {
        let mut calc = RpnCalculator::new();
        let mparam = [1.0, 12.0, 1.0];

        let template = "/ch/01/mix/on ,i [$2 0 >]";
        let out = execute_template(template, &mparam, &mut calc).unwrap();

        let msg = OscMessage::from_bytes(&out).unwrap();
        assert_eq!(msg.path, "/ch/01/mix/on");
        assert_eq!(msg.args.len(), 1);
        if let OscArg::Int(i) = msg.args[0] {
            assert_eq!(i, 1);
        } else {
            panic!("Expected Int argument");
        }
    }

    #[test]
    fn test_execute_template_mixed() {
        let mut calc = RpnCalculator::new();
        let mparam = [1.0, 12.0, 64.0];

        // test mixing RPN and standard arguments
        let template = "/ch/01/config/name ,sif MyChannel [$1 2 *] [$2 128.0 /]";
        let out = execute_template(template, &mparam, &mut calc).unwrap();

        let msg = OscMessage::from_bytes(&out).unwrap();
        assert_eq!(msg.path, "/ch/01/config/name");
        assert_eq!(msg.args.len(), 3);

        assert_eq!(msg.args[0], OscArg::String("MyChannel".to_string()));

        if let OscArg::Int(i) = msg.args[1] {
            assert_eq!(i, 24);
        } else {
            panic!("Expected Int argument");
        }

        if let OscArg::Float(f) = msg.args[2] {
            assert_eq!(f, 0.5);
        } else {
            panic!("Expected Float argument");
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let rules = config::parse_file(&args.file).context("Failed to load .m2o rules file")?;
    println!("Loaded {} rules from {}", rules.len(), args.file);

    let config = config::Config::load(".X32Midi2OSC.ini").unwrap_or_default();

    let ip = if args.ip.is_empty() {
        config.ip_str.clone()
    } else {
        args.ip.clone()
    };

    let x32_addr = format!("{}:10023", ip);
    let socket: std::net::UdpSocket = std::net::UdpSocket::bind("0.0.0.0:0")?;
    socket.set_nonblocking(true)?;
    let socket = UdpSocket::from_std(socket)?;
    socket.connect(&x32_addr).await?;
    let socket = Arc::new(socket);

    // Initial keepalive
    socket.send(b"/xremote\0\0\0\0").await?;

    let mut midi_in = MidiInput::new("x32_midi2osc")?;
    midi_in.ignore(Ignore::None);

    let in_ports = midi_in.ports();
    let mut selected_port = None;

    if in_ports.is_empty() {
        println!("No MIDI input ports available.");
        return Ok(());
    }

    // Try finding by explicit name match or index from config if no name provided
    for (i, port) in in_ports.iter().enumerate() {
        let name = midi_in.port_name(port)?;
        if (!args.midi_in.is_empty() && name.to_lowercase().contains(&args.midi_in.to_lowercase()))
            || (args.midi_in.is_empty() && (i + 1) as i32 == config.midi_in_port)
            || (args.midi_in.is_empty() && config.midi_in_port == 0 && i == 0)
        // fallback to 0th
        {
            selected_port = Some(port.clone());
            println!("Selecting MIDI Input: {}", name);
            break;
        }
    }

    let in_port = match selected_port {
        Some(p) => p,
        None => {
            println!("Could not find matching MIDI input port.");
            return Ok(());
        }
    };

    let rules_clone = Arc::new(rules);
    let socket_clone = socket.clone();

    // Use a multi-producer, single-consumer channel to send matching OSC payloads
    // from the synchronous midi thread to the async tokio thread
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();

    let _conn_in = match midi_in.connect(
        &in_port,
        "x32_midi2osc_in",
        move |_stamp, message: &[u8], _| {
            if message.len() < 2 {
                return;
            }

            let status = message[0];
            if (status & 0x80) == 0 {
                return;
            }

            let mmc = (status & 0xF0) as i32;
            let mch = ((status & 0x0F) + 1) as i32;

            let md1 = message[1] as i32;
            let md2 = if message.len() > 2 {
                message[2] as i32
            } else {
                0
            };

            let mut mparam = [0.0; 3];
            mparam[0] = mch as f64;
            mparam[1] = md1 as f64;

            match (status & 0x70) >> 4 {
                0x00 | 0x01 | 0x02 | 0x03 | 0x06 | 0x07 => {
                    mparam[2] = md2 as f64;
                }
                0x04 | 0x05 => {
                    mparam[2] = 0x80 as f64; // C code uses 0x80 for these
                }
                _ => {}
            }

            let mut calculator = RpnCalculator::new();
            let command_id = ((md1 << 8) | (mmc | ((mch - 1) & 0xF))) as u32;

            for rule in rules_clone.iter() {
                if command_id == rule.get_match_key() {
                    if (mparam[2] as i32) & 0x80 != 0 {
                        mparam[2] = (rule.data2 & 0x7F) as f64;
                    }

                    if let Ok(payload) =
                        execute_template(&rule.osc_command, &mparam, &mut calculator)
                    {
                        let _ = tx.send(payload);
                    }
                }
            }
        },
        (),
    ) {
        Ok(c) => c,
        Err(e) => return Err(anyhow::anyhow!("Error connecting to MIDI port: {}", e)),
    };

    let mut last_xremote = Instant::now();

    loop {
        tokio::select! {
            _ = time::sleep_until(last_xremote + Duration::from_secs(9)) => {
                let _ = socket_clone.send(b"/xremote\0\0\0\0").await;
                last_xremote = Instant::now();
            }
            recv_result = rx.recv() => {
                if let Some(payload) = recv_result {
                    let _ = socket_clone.send(&payload).await;
                }
            }
        }
    }
}
