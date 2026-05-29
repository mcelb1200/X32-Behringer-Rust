use anyhow::{Context, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use clap::Parser;
use osc_lib::{OscArg, OscMessage};
use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about = "X32 OSC diagnostic dumper", long_about = None)]
struct Args {
    /// File to read OSC data from (defaults to stdin)
    #[arg(short, long)]
    file: Option<PathBuf>,
}

fn format_osc_message(msg: &OscMessage) -> String {
    let mut out = format!("{} ,", msg.path);
    for arg in &msg.args {
        match arg {
            OscArg::Int(_) => out.push('i'),
            OscArg::Float(_) => out.push('f'),
            OscArg::String(_) => out.push('s'),
            OscArg::Blob(_) => out.push('b'),
        }
    }

    if msg.path == "/meters/15" {
        // Special formatting for /meters/15
        if let Some(OscArg::Blob(blob)) = msg.args.first() {
            let mut cursor = std::io::Cursor::new(blob);
            if let Ok(num_bytes) = cursor.read_i32::<LittleEndian>() {
                let rta_count = num_bytes * 2;
                out.push_str(&format!("~ {} rta: \n", rta_count));

                for j in 0..rta_count {
                    if let Ok(val) = cursor.read_i16::<LittleEndian>() {
                        let fval = val as f32 / 256.0;
                        out.push_str(&format!("[{}] {:07.2} ", j, fval));
                    } else {
                        break;
                    }
                }
            }
        }
    } else if msg.path == "/meters/16" {
        // Special formatting for /meters/16
        if let Some(OscArg::Blob(blob)) = msg.args.first() {
            let mut cursor = std::io::Cursor::new(blob);
            if let Ok(num_bytes) = cursor.read_i32::<LittleEndian>() {
                let count = num_bytes * 2;
                out.push_str(&format!("~ M/16: {} shorts\n", count));

                for j in 0..count {
                    if let Ok(val) = cursor.read_i16::<LittleEndian>() {
                        if j < count - 8 {
                            let fval = val as f32 / 32767.0;
                            if j < 32 {
                                out.push_str(&format!("[{}: G {:07.2}] ", j, fval));
                            } else if j < 64 {
                                out.push_str(&format!("[{}: C {:07.2}] ", j, fval));
                            } else if j < 80 {
                                out.push_str(&format!("[{}: B {:07.2}] ", j, fval));
                            } else if j < 86 {
                                out.push_str(&format!("[{}: M {:07.2}] ", j, fval));
                            } else if j == 86 {
                                out.push_str(&format!("[{}:LR {:07.2}] ", j, fval));
                            } else if j == 87 {
                                out.push_str(&format!("[{}:MC {:07.2}] ", j, fval));
                            }
                        } else {
                            let fval = val as f32 / 256.0;
                            out.push_str(&format!("[{}: A {:07.2}] ", j, fval));
                        }
                    } else {
                        break;
                    }
                }
                out.push('\n');
            }
        }
    } else {
        // Generic formatting for standard messages
        for arg in &msg.args {
            out.push('~');
            match arg {
                OscArg::Int(val) => out.push_str(&format!("[{:6}]", val)),
                OscArg::Float(val) => {
                    let v = *val;
                    if v < 10.0 {
                        out.push_str(&format!("[{:06.4}]", v));
                    } else if v < 100.0 {
                        out.push_str(&format!("[{:06.3}]", v));
                    } else if v < 1000.0 {
                        out.push_str(&format!("[{:06.2}]", v));
                    } else if v < 10000.0 {
                        out.push_str(&format!("[{:06.1}]", v));
                    } else {
                        out.push_str(&format!("[{:}]", v));
                    }
                }
                OscArg::String(val) => out.push_str(val),
                OscArg::Blob(blob) => {
                    let mut cursor = std::io::Cursor::new(blob);
                    if let Ok(num_bytes) = cursor.read_i32::<LittleEndian>() {
                        let mut is_string = false;
                        if blob.len() > 4 {
                            let val1 = &blob[4..];
                            if let Ok(s) = std::str::from_utf8(val1) {
                                // if it's purely ascii string we can consider it as string
                                if s.chars().all(|c| {
                                    c.is_ascii_alphanumeric()
                                        || c.is_ascii_whitespace()
                                        || c.is_ascii_punctuation()
                                }) {
                                    is_string = true;
                                }
                            }
                        }

                        if is_string {
                            out.push_str(&format!(" {} chrs: ", num_bytes));
                            if let Ok(s) = std::str::from_utf8(&blob[4..]) {
                                for c in s.chars() {
                                    out.push(c);
                                    out.push(' ');
                                }
                            }
                        } else {
                            // floats format
                            let count = num_bytes;
                            out.push_str(&format!(" {} flts: ", count));
                            for _ in 0..count {
                                if let Ok(fval) = cursor.read_f32::<LittleEndian>() {
                                    out.push_str(&format!("{:06.2} ", fval));
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    out
}

fn process_stream<R: Read>(mut reader: R) -> Result<()> {
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;

    let mut offset = 0;
    while offset < buffer.len() {
        let chunk = &buffer[offset..];

        // Find end of OSC message by finding the first null byte of the path, and then calculating sizes
        // But since osc_lib doesn't return the bytes consumed, let's just attempt to decode everything from the current offset
        // In most cases, `osc_lib` decodes one packet. We need to know how many bytes were consumed.
        // As a workaround for raw concatenated messages, we can just split by finding the next `/`
        // after the current packet, since OSC paths start with `/`.
        // We'll scan for the next `/` which is padded after null bytes.

        if let Ok(msg) = OscMessage::from_bytes(chunk) {
            println!("{}", format_osc_message(&msg));

            // Advance offset. How many bytes did `msg` take?
            // Re-serialize to get exact bytes consumed? `msg.to_bytes()` gives us length.
            if let Ok(serialized) = msg.to_bytes() {
                offset += serialized.len();
            } else {
                // Should not happen if parsing succeeded
                offset += chunk.len();
            }

            // Advance over any padding nulls that might have been skipped between concatenated messages
            while offset < buffer.len() && buffer[offset] == 0 {
                offset += 1;
            }
        } else {
            // If we fail to parse, print and abort
            println!("Invalid OSC packet remaining: {} bytes", chunk.len());
            break;
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    if let Some(path) = args.file {
        let file = File::open(&path).with_context(|| format!("Failed to open file: {:?}", path))?;
        process_stream(file)
    } else {
        process_stream(io::stdin())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_standard_message() {
        let msg = OscMessage {
            path: "/ch/01/mix/fader".to_string(),
            args: vec![OscArg::Float(0.75)],
        };
        let formatted = format_osc_message(&msg);
        assert_eq!(formatted, "/ch/01/mix/fader ,f~[0.7500]");
    }

    #[test]
    fn test_format_meters_15() {
        let msg = OscMessage {
            path: "/meters/15".to_string(),
            args: vec![OscArg::Blob(vec![2, 0, 0, 0, 0, 0, 0, 1])],
        };
        let formatted = format_osc_message(&msg);
        assert!(formatted.contains("/meters/15 ,b~ 4 rta:"));
        assert!(formatted.contains("[0] 0000.00"));
        assert!(formatted.contains("[1] 0001.00"));
    }

    #[test]
    fn test_format_meters_16() {
        let msg = OscMessage {
            path: "/meters/16".to_string(),
            args: vec![OscArg::Blob(vec![
                4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ])],
        };
        let formatted = format_osc_message(&msg);
        assert!(formatted.contains("/meters/16 ,b~ M/16: 8 shorts"));
    }
}
