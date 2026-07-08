//! `x32_commander` is a bridge application that listens for specific OSC commands from
//! an X32 mixer and triggers other OSC commands in response.
//!
//! It reads a configuration file (default: `X32Commander.txt`) which maps incoming
//! OSC paths to outgoing OSC commands. This allows for complex automation and logic
//! that is not natively supported by the mixer, such as controlling one channel based
//! on the state of another, or creating macro-like functionality.
//!
//! # Credits
//!
//! *   **Original concept and work on the C library:** Patrick-Gilles Maillot
//! *   **Additional concepts by:** mcelb1200
//! *   **Rust implementation by:** mcelb1200

use clap::Parser;
use std::fs::File;
use std::io::{self, BufRead, Read};
use std::str::FromStr;
use tokio::net::UdpSocket;

use midir::MidiOutput;
use osc_lib::{OscArg, OscMessage};
pub mod rpn;
use rpn::RpnCalculator;
use x32_lib::MixerClient;

/// A Rust implementation of the X32Commander utility.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path to the command file.
    #[arg(short, long, default_value = "X32Commander.txt")]
    pub file: String,

    /// IP address of the X32 mixer.
    #[arg(short, long)]
    pub ip: String,

    /// OSC output address and port (e.g., 127.0.0.1:9000).
    #[arg(short, long)]
    pub output: Option<String>,

    /// MIDI output port name.
    #[arg(short = 'm', long)]
    pub midi_out: Option<String>,
}

/// Represents the type of command to trigger.
#[derive(Debug, Clone, PartialEq)]
enum CommandType {
    /// An Open Sound Control (OSC) command.
    Osc,
    /// A MIDI command.
    Midi,
}

/// Represents a parsed command mapping from the configuration file.
#[derive(Debug, Clone, PartialEq)]
struct Command {
    /// The type of the outgoing command (OSC or MIDI).
    command_type: CommandType,
    /// The OSC path to listen for on the input.
    incoming_address: String,
    /// The command string to execute when the trigger is received.
    outgoing_command: String,
}

/// Parses the command configuration file.
fn parse_command_file(path: &str) -> io::Result<Vec<Command>> {
    let file = File::open(path)?;

    if file.metadata()?.len() > 1024 * 1024 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Configuration file too large (max 1MB)",
        ));
    }

    let mut file = file.take(1024 * 1024);
    let mut reader = io::BufReader::new(&mut file);
    let mut commands = Vec::new();

    loop {
        let mut byte_buf = Vec::new();
        match reader.by_ref().take(4096).read_until(b'\n', &mut byte_buf) {
            Ok(0) => {
                if file.limit() == 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Configuration file exceeded the maximum length of 1MB during processing",
                    ));
                }
                break;
            }
            Err(e) => return Err(e),
            Ok(len) => {
                if len == 4096 && !byte_buf.ends_with(b"\n") {
                    let mut discard = Vec::with_capacity(1024);
                    loop {
                        discard.clear();
                        match reader.by_ref().take(1024).read_until(b'\n', &mut discard) {
                            Ok(0) | Err(_) => break,
                            Ok(_) => {
                                if discard.ends_with(b"\n") {
                                    break;
                                }
                            }
                        }
                    }
                    eprintln!("Input line too long, discarded.");
                    continue;
                }
            }
        }

        let line_str = match std::str::from_utf8(&byte_buf) {
            Ok(s) => s,
            Err(_) => {
                eprintln!("Invalid UTF-8 sequence in input, discarded.");
                continue;
            }
        };

        let line = line_str.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((trigger, action)) = line.split_once('|') {
            let trigger = trigger.trim();
            let action = action.trim();

            if trigger.len() < 5 || &trigger[1..4] != "~~~" {
                continue;
            }

            let command_type = match trigger.chars().next() {
                Some('O') => CommandType::Osc,
                Some('M') => CommandType::Midi,
                _ => continue,
            };

            let incoming_address = trigger[4..].trim().to_string();

            commands.push(Command {
                command_type,
                incoming_address,
                outgoing_command: action.to_string(),
            });
        }
    }
    Ok(commands)
}

fn parse_midi_hex(hex_str: &str) -> std::result::Result<Vec<u8>, anyhow::Error> {
    // ⚡ Bolt: Manually parse hex bytes instead of using `u8::from_str_radix`.
    // This avoids overhead of slice creation, generic parsing, and utf8 checks,
    // which improves configuration loading speed for a better UX.
    let mut result = Vec::new();
    let bytes = hex_str.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }

        let mut val = 0u8;
        let mut count = 0;
        while i < bytes.len() && !bytes[i].is_ascii_whitespace() {
            let b = bytes[i];
            let v = match b {
                b'0'..=b'9' => b - b'0',
                b'a'..=b'f' => b - b'a' + 10,
                b'A'..=b'F' => b - b'A' + 10,
                _ => {
                    return Err(anyhow::anyhow!(
                        "Invalid hex character in byte '{}'",
                        b as char
                    ));
                }
            };
            val = (val << 4) | v;
            count += 1;
            i += 1;
        }
        if count > 2 {
            return Err(anyhow::anyhow!(
                "Invalid hex byte length: expected at most 2 hex chars per byte"
            ));
        }
        result.push(val);
    }
    Ok(result)
}

#[derive(Debug, Clone)]
pub enum CommandParam {
    Float(f64),
    String(String),
}

impl CommandParam {
    pub fn as_f64(&self) -> f64 {
        match self {
            CommandParam::Float(f) => *f,
            CommandParam::String(_) => 0.0,
        }
    }
    pub fn as_str(&self) -> &str {
        match self {
            CommandParam::Float(_) => "",
            CommandParam::String(s) => s.as_str(),
        }
    }
}

fn generate_rpn_nrpn(
    is_nrpn: bool,
    channel: u8,
    param_msb: u8,
    param_lsb: u8,
    val_msb: u8,
    val_lsb: Option<u8>,
) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(12);
    let cc_param_msb = if is_nrpn { 99 } else { 101 };
    let cc_param_lsb = if is_nrpn { 98 } else { 100 };

    // Select Parameter
    bytes.push(0xB0 | (channel & 0x0F));
    bytes.push(cc_param_msb);
    bytes.push(param_msb & 0x7F);

    bytes.push(0xB0 | (channel & 0x0F));
    bytes.push(cc_param_lsb);
    bytes.push(param_lsb & 0x7F);

    // Data Entry MSB
    bytes.push(0xB0 | (channel & 0x0F));
    bytes.push(6);
    bytes.push(val_msb & 0x7F);

    // Data Entry LSB (optional)
    if let Some(lsb) = val_lsb {
        bytes.push(0xB0 | (channel & 0x0F));
        bytes.push(38);
        bytes.push(lsb & 0x7F);
    }

    bytes
}

fn expand_template(
    template: &str,
    mparam: &[CommandParam],
    calc: &mut RpnCalculator,
) -> Result<String, anyhow::Error> {
    let mut result = String::with_capacity(template.len());
    let mut chars = template.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '[' {
            let mut expr = String::new();
            while let Some(&nc) = chars.peek() {
                if nc == ']' {
                    chars.next(); // consume ']'
                    break;
                }
                expr.push(chars.next().unwrap());
            }

            if let Some(stripped) = expr.strip_prefix("$s") {
                if let Ok(idx) = stripped.parse::<usize>() {
                    if idx < mparam.len() {
                        result.push_str(mparam[idx].as_str());
                        continue;
                    }
                }
            }

            let val = calc.calculate(&expr, mparam)?;

            // Determine formatting based on the template structure heuristically.
            // If the template looks like a hex string block (e.g. MIDI), we format as hex.
            // Otherwise, we format as float (e.g., OSC argument).
            let is_midi =
                template.len() > 5 && (template.starts_with("F0") || template.starts_with("B0"));
            if is_midi {
                use std::fmt::Write;
                write!(&mut result, "{:02X}", val as u8)?;
            } else {
                use std::fmt::Write;
                write!(&mut result, "{}", val)?;
            }
        } else {
            result.push(c);
        }
    }

    Ok(result)
}

pub async fn run(args: Args) -> anyhow::Result<()> {
    let commands = parse_command_file(&args.file)
        .map_err(|e| anyhow::anyhow!("Failed to parse command file: {}", e))?;
    println!("Successfully parsed {} commands.", commands.len());

    println!("Connecting to X32 at {}...", args.ip);
    let client = MixerClient::connect(&args.ip, true).await?;

    let out_socket = if let Some(ref output_addr) = args.output {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.connect(output_addr).await?;
        println!("OSC output enabled to: {}", output_addr);
        Some(socket)
    } else {
        None
    };

    let mut midi_conn = None;
    if let Some(ref midi_port_name) = args.midi_out {
        let midi_out = MidiOutput::new("x32_commander")?;
        let ports = midi_out.ports();
        let mut found_port = None;
        for port in &ports {
            if let Ok(name) = midi_out.port_name(port) {
                if name.to_lowercase().contains(&midi_port_name.to_lowercase()) {
                    found_port = Some(port.clone());
                    break;
                }
            }
        }

        if let Some(port) = found_port {
            println!(
                "Connecting to MIDI output: {}",
                midi_out.port_name(&port).unwrap_or_default()
            );
            let conn = midi_out
                .connect(&port, "x32_commander_out")
                .map_err(|e| anyhow::anyhow!("Failed to connect to MIDI port: {}", e))?;
            midi_conn = Some(conn);
        }
    }

    let mut rx = client.subscribe();
    println!("Listening for OSC messages...");
    while let Ok(incoming_msg) = rx.recv().await {
        for command in &commands {
            if command.incoming_address == incoming_msg.path {
                let mut mparam = Vec::with_capacity(incoming_msg.args.len());
                for arg in &incoming_msg.args {
                    match arg {
                        OscArg::Int(i) => mparam.push(CommandParam::Float(*i as f64)),
                        OscArg::Float(f) => mparam.push(CommandParam::Float(*f as f64)),
                        OscArg::String(s) => mparam.push(CommandParam::String(s.clone())),
                        _ => mparam.push(CommandParam::Float(0.0)),
                    }
                }
                let mut calc = RpnCalculator::new();

                match command.command_type {
                    CommandType::Osc => {
                        println!(
                            "Match found for: {}. Triggering: {}",
                            incoming_msg.path, command.outgoing_command
                        );
                        match expand_template(&command.outgoing_command, &mparam, &mut calc) {
                            Ok(expanded) => {
                                if let Ok(outgoing_msg) = OscMessage::from_str(&expanded) {
                                    if let Some(ref target_socket) = out_socket {
                                        if let Ok(bytes) = outgoing_msg.to_bytes() {
                                            let _ = target_socket.send(&bytes).await;
                                        }
                                    } else {
                                        let _ = client
                                            .send_message(&outgoing_msg.path, outgoing_msg.args)
                                            .await;
                                    }
                                }
                            }
                            Err(e) => eprintln!("Failed to expand template: {}", e),
                        }
                    }
                    CommandType::Midi => {
                        println!(
                            "Match found for: {}. Triggering MIDI: {}",
                            incoming_msg.path, command.outgoing_command
                        );
                        if command.outgoing_command.starts_with("NRPN ")
                            || command.outgoing_command.starts_with("RPN ")
                        {
                            let is_nrpn = command.outgoing_command.starts_with("NRPN ");

                            let slice_start = if is_nrpn { 5 } else { 4 };
                            match expand_template(
                                &command.outgoing_command[slice_start..],
                                &mparam,
                                &mut calc,
                            ) {
                                Ok(expanded) => {
                                    let mut parts = expanded.split_whitespace();
                                    let p0 = parts.next();
                                    let p1 = parts.next();
                                    let p2 = parts.next();
                                    let p3 = parts.next();
                                    let p4 = parts.next();

                                    if let (
                                        Some(channel_str),
                                        Some(param_msb_str),
                                        Some(param_lsb_str),
                                        Some(val_msb_str),
                                    ) = (p0, p1, p2, p3)
                                    {
                                        let channel = channel_str.parse::<u8>().unwrap_or(0);
                                        let param_msb = param_msb_str.parse::<u8>().unwrap_or(0);
                                        let param_lsb = param_lsb_str.parse::<u8>().unwrap_or(0);
                                        let val_msb = val_msb_str.parse::<u8>().unwrap_or(0);
                                        let val_lsb = p4.and_then(|s| s.parse::<u8>().ok());

                                        let bytes = generate_rpn_nrpn(
                                            is_nrpn, channel, param_msb, param_lsb, val_msb,
                                            val_lsb,
                                        );
                                        if let Some(ref mut conn) = midi_conn {
                                            let _ = conn.send(&bytes);
                                        }
                                    } else {
                                        eprintln!(
                                            "Invalid RPN/NRPN format. Expected: channel param_msb param_lsb val_msb [val_lsb]"
                                        );
                                    }
                                }
                                Err(e) => eprintln!("Failed to expand template: {}", e),
                            }
                        } else {
                            match expand_template(&command.outgoing_command, &mparam, &mut calc) {
                                Ok(expanded) => {
                                    if let Ok(bytes) = parse_midi_hex(&expanded) {
                                        if let Some(ref mut conn) = midi_conn {
                                            let _ = conn.send(&bytes);
                                        }
                                    }
                                }
                                Err(e) => eprintln!("Failed to expand template: {}", e),
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn create_test_file(content: &str) -> anyhow::Result<(tempfile::NamedTempFile, String)> {
        let mut file = tempfile::NamedTempFile::new()?;
        writeln!(file, "{}", content)?;
        let path = file
            .path()
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Failed to convert temp file path to string"))?
            .to_string();
        Ok((file, path))
    }

    #[test]
    fn test_parse_valid_osc_command() -> anyhow::Result<()> {
        let content = "O~~~/ch/01/mix/fader|/ch/02/mix/fader ,f 0.5";
        let (_file, path) = create_test_file(content)?;
        let commands = parse_command_file(&path)?;
        assert_eq!(commands.len(), 1);
        assert_eq!(
            commands[0],
            Command {
                command_type: CommandType::Osc,
                incoming_address: "/ch/01/mix/fader".to_string(),
                outgoing_command: "/ch/02/mix/fader ,f 0.5".to_string(),
            }
        );
        Ok(())
    }

    #[test]
    fn test_parse_valid_midi_command() -> anyhow::Result<()> {
        let content = "M~~~/ch/01/mix/fader|F0 00 20 32 32 2F 63 68 2F 30 31 2F 6D 69 78 2F 66 61 64 65 72 20 2C 66 20 30 2E 35 F7";
        let (_file, path) = create_test_file(content)?;
        let commands = parse_command_file(&path)?;
        assert_eq!(commands.len(), 1);
        assert_eq!(
            commands[0],
            Command {
                command_type: CommandType::Midi,
                incoming_address: "/ch/01/mix/fader".to_string(),
                outgoing_command: "F0 00 20 32 32 2F 63 68 2F 30 31 2F 6D 69 78 2F 66 61 64 65 72 20 2C 66 20 30 2E 35 F7".to_string(),
            }
        );
        Ok(())
    }

    #[test]
    fn test_parse_empty_lines_and_comments() -> anyhow::Result<()> {
        let content = "
# This is a comment
O~~~/ch/01/mix/fader|/ch/02/mix/fader ,f 0.5

M~~~/ch/02/mix/fader|/ch/03/mix/fader ,f 0.75
";
        let (_file, path) = create_test_file(content)?;
        let commands = parse_command_file(&path)?;
        assert_eq!(commands.len(), 2);
        Ok(())
    }

    #[test]
    fn test_parse_invalid_format() -> anyhow::Result<()> {
        let content = "invalid line";
        let (_file, path) = create_test_file(content)?;
        let commands = parse_command_file(&path)?;
        assert_eq!(commands.len(), 0);
        Ok(())
    }

    #[test]
    fn test_parse_midi_hex() -> anyhow::Result<()> {
        let hex_str = "F0 00 20 32 32 2F 63 68 2F 30 31 2F 6D 69 78 2F 66 61 64 65 72 20 2C 66 20 30 2E 35 F7";
        let parsed = parse_midi_hex(hex_str)?;
        assert_eq!(
            parsed,
            vec![
                0xF0, 0x00, 0x20, 0x32, 0x32, 0x2F, 0x63, 0x68, 0x2F, 0x30, 0x31, 0x2F, 0x6D, 0x69,
                0x78, 0x2F, 0x66, 0x61, 0x64, 0x65, 0x72, 0x20, 0x2C, 0x66, 0x20, 0x30, 0x2E, 0x35,
                0xF7
            ]
        );

        let invalid = "F0 00 20 32 32 XX F7";
        assert!(parse_midi_hex(invalid).is_err());
        Ok(())
    }

    #[test]
    fn test_expand_template_osc() -> anyhow::Result<()> {
        let mut calc = RpnCalculator::new();
        let mparam = vec![CommandParam::Float(0.5)];
        let template = "/ch/02/mix/fader ,f [$0 0.5 *]";
        let result = expand_template(template, &mparam, &mut calc)?;
        assert_eq!(result, "/ch/02/mix/fader ,f 0.25");

        let mparam2 = vec![CommandParam::Float(1.0), CommandParam::Float(2.0)];
        let template2 = "/ch/02/mix/fader ,f [$1 $0 +]";
        let result2 = expand_template(template2, &mparam2, &mut calc)?;
        assert_eq!(result2, "/ch/02/mix/fader ,f 3");
        Ok(())
    }

    #[test]
    fn test_expand_template_string() -> anyhow::Result<()> {
        let mut calc = RpnCalculator::new();
        let mparam = vec![CommandParam::String("Hello".to_string())];
        let template = "/ch/02/config/name ,s [$s0]";
        let result = expand_template(template, &mparam, &mut calc)?;
        assert_eq!(result, "/ch/02/config/name ,s Hello");
        Ok(())
    }

    #[test]
    fn test_generate_rpn_nrpn() {
        // Test NRPN Channel 1 (0), Param MSB 1, LSB 2, Val MSB 3, LSB 4
        let bytes = generate_rpn_nrpn(true, 0, 1, 2, 3, Some(4));
        assert_eq!(
            bytes,
            vec![0xB0, 99, 1, 0xB0, 98, 2, 0xB0, 6, 3, 0xB0, 38, 4]
        );

        // Test RPN Channel 2 (1), Param MSB 5, LSB 6, Val MSB 7, NO LSB
        let bytes = generate_rpn_nrpn(false, 1, 5, 6, 7, None);
        assert_eq!(bytes, vec![0xB1, 101, 5, 0xB1, 100, 6, 0xB1, 6, 7]);
    }

    #[test]
    fn test_expand_template_midi() -> anyhow::Result<()> {
        let mut calc = RpnCalculator::new();
        let mparam = vec![CommandParam::Float(0.5)];
        let template = "F0 00 [$0 127 *] F7";
        let result = expand_template(template, &mparam, &mut calc)?;
        assert_eq!(result, "F0 00 3F F7"); // 0.5 * 127 = 63.5 -> 63 -> 3F in hex
        Ok(())
    }

    #[test]
    fn test_expand_template_no_expression() -> anyhow::Result<()> {
        let mut calc = RpnCalculator::new();
        let mparam = vec![];
        let template = "/ch/02/mix/fader ,f 0.5";
        let result = expand_template(template, &mparam, &mut calc)?;
        assert_eq!(result, "/ch/02/mix/fader ,f 0.5");
        Ok(())
    }
}
