use anyhow::Result;
use byteorder::{LittleEndian, ReadBytesExt};
use clap::Parser;
use osc_lib::{OscArg, OscMessage};
use std::fs::File;
use std::io::{self, Read};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The file to read from. If missing, reads from STDIN.
    #[arg(value_name = "FILE")]
    file: Option<String>,
}

use std::fmt::Write;

fn dump_osc_message(msg: &OscMessage, out: &mut String) {
    out.clear();
    // format like Xdump.c:
    let _ = write!(
        out,
        "{}, {:4} B: ",
        msg.path,
        OscMessage::serialize_to_bytes(&msg.path, &msg.args)
            .unwrap_or_default()
            .len()
    );

    // We emulate Xdump.c printing logic:
    // It prints comma
    out.push(',');
    for arg in &msg.args {
        match arg {
            OscArg::Int(_) => out.push('i'),
            OscArg::Float(_) => out.push('f'),
            OscArg::String(_) => out.push('s'),
            OscArg::Blob(_) => out.push('b'),
        }
    }

    for arg in &msg.args {
        match arg {
            OscArg::Int(val) => {
                let _ = write!(out, " [{:6}]", val);
            }
            OscArg::Float(val) => {
                let f = *val;
                if f < 10.0 {
                    let _ = write!(out, " [{:06.4}]", f);
                } else if f < 100.0 {
                    let _ = write!(out, " [{:06.3}]", f);
                } else if f < 1000.0 {
                    let _ = write!(out, " [{:06.2}]", f);
                } else {
                    let _ = write!(out, " [{:06.1}]", f);
                }
            }
            OscArg::String(val) => {
                let _ = write!(out, " {}", val);
            }
            OscArg::Blob(blob) => {
                // Read from blob based on path
                if msg.path.starts_with("/meters/15") {
                    let mut cursor = std::io::Cursor::new(blob);
                    if let Ok(num_elements) = cursor.read_i32::<LittleEndian>() {
                        let n = num_elements * 2;
                        let _ = writeln!(out, " {} rta: ", n);
                        for j in 0..n {
                            if let Ok(s) = cursor.read_i16::<LittleEndian>() {
                                let f = (s as f32) / 256.0;
                                let _ = write!(out, "[{}] {:07.2} ", j, f);
                            } else {
                                break;
                            }
                        }
                    }
                } else if msg.path.starts_with("/meters/16") {
                    let mut cursor = std::io::Cursor::new(blob);
                    if let Ok(num_elements) = cursor.read_i32::<LittleEndian>() {
                        let n = num_elements * 2;
                        let _ = writeln!(out, " M/16: {} shorts", n);
                        for j in 0..(n - 8) {
                            if let Ok(s) = cursor.read_i16::<LittleEndian>() {
                                let f = (s as f32) / 32767.0;
                                if j < 32 {
                                    let _ = write!(out, "[{}: G {:07.2}] ", j, f);
                                } else if j < 64 {
                                    let _ = write!(out, "[{}: C {:07.2}] ", j, f);
                                } else if j < 80 {
                                    let _ = write!(out, "[{}: B {:07.2}] ", j, f);
                                } else if j < 86 {
                                    let _ = write!(out, "[{}: M {:07.2}] ", j, f);
                                } else if j == 86 {
                                    let _ = write!(out, "[{}:LR {:07.2}] ", j, f);
                                } else if j == 87 {
                                    let _ = write!(out, "[{}:MC {:07.2}] ", j, f);
                                }
                            } else {
                                break;
                            }
                        }
                        for j in (n - 8)..n {
                            if let Ok(s) = cursor.read_i16::<LittleEndian>() {
                                let f = (s as f32) / 256.0;
                                let _ = write!(out, "[{}: A {:07.2}] ", j, f);
                            } else {
                                break;
                            }
                        }
                        out.push('\n');
                    }
                } else {
                    let mut cursor = std::io::Cursor::new(blob);
                    if let Ok(num_elements) = cursor.read_i32::<LittleEndian>() {
                        let _ = write!(out, " {} flts: ", num_elements);
                        for _ in 0..num_elements {
                            if let Ok(f) = cursor.read_f32::<LittleEndian>() {
                                let _ = write!(out, "{:06.2} ", f);
                            } else {
                                break;
                            }
                        }
                    } else {
                        let _ = write!(out, " {} chrs: ", blob.len());
                        for b in blob {
                            out.push(*b as char);
                            out.push(' ');
                        }
                    }
                }
            }
        }
    }
    println!("{}", out);
}

fn process_stream<R: Read>(reader: R) -> Result<()> {
    let mut buffer = Vec::new();
    let limit = 10 * 1024 * 1024; // 10MB limit to prevent OOM DoS
    reader.take(limit + 1).read_to_end(&mut buffer)?;

    if buffer.len() > limit as usize {
        anyhow::bail!("Input exceeds maximum allowed size (10MB)");
    }

    // ⚡ Bolt: Pre-allocate string to avoid reallocations on every message dump iteration
    let mut out = String::with_capacity(1024);
    let mut offset = 0;
    while offset < buffer.len() {
        if let Ok(msg) = OscMessage::from_bytes(&buffer[offset..]) {
            let consumed = match OscMessage::serialize_to_bytes(&msg.path, &msg.args) {
                Ok(b) => b.len(),
                Err(_) => {
                    offset += 4;
                    continue;
                }
            };

            dump_osc_message(&msg, &mut out);

            offset += consumed;

            while offset < buffer.len() && buffer[offset] == 0 {
                offset += 1;
            }
            let rem = offset % 4;
            if rem != 0 {
                offset += 4 - rem;
            }
        } else {
            offset += 4;
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    if let Some(file_path) = args.file {
        let file = File::open(file_path)?;
        process_stream(file)?;
    } else {
        let stdin = io::stdin();
        process_stream(stdin.lock())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_generic_osc() {
        let msg = OscMessage {
            path: "/ch/01/mix/fader".to_string(),
            args: vec![OscArg::Float(0.75)],
        };
        let mut out = String::new();
        dump_osc_message(&msg, &mut out);
    }

    #[test]
    fn test_parse_meters_15() {
        let path = "/meters/15";
        let mut raw_bytes = Vec::new();

        let num_shorts: i32 = 2;
        raw_bytes.extend_from_slice(&num_shorts.to_le_bytes());
        let s1: i16 = 256; // -> 1.0
        let s2: i16 = -512; // -> -2.0
        raw_bytes.extend_from_slice(&s1.to_le_bytes());
        raw_bytes.extend_from_slice(&s2.to_le_bytes());

        let msg = OscMessage {
            path: path.to_string(),
            args: vec![OscArg::Blob(raw_bytes)],
        };
        let mut out = String::new();
        dump_osc_message(&msg, &mut out);
    }

    #[test]
    fn test_parse_meters_16() {
        let path = "/meters/16";
        let mut raw_bytes = Vec::new();
        let num_shorts: i32 = 4; // meaning n = 8.
        raw_bytes.extend_from_slice(&num_shorts.to_le_bytes());
        for _ in 0..8 {
            raw_bytes.extend_from_slice(&0i16.to_le_bytes());
        }

        let msg = OscMessage {
            path: path.to_string(),
            args: vec![OscArg::Blob(raw_bytes)],
        };
        let mut out = String::new();
        dump_osc_message(&msg, &mut out);
    }
}
