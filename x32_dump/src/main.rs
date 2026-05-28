use anyhow::{Context, Result};
use byteorder::ReadBytesExt;
use clap::Parser;
use std::io::{self, Cursor, Read, Write};
use std::fs::File;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Debug mode (print raw bytes before parsing)
    #[arg(short, long)]
    debug: bool,

    /// Input file path. If not provided, reads from standard input.
    #[arg(short, long)]
    file: Option<String>,
}

fn dump_osc_buffer(buf: &[u8], debug: bool, out: &mut impl Write) -> Result<()> {
    let mut i = 0;
    let len = buf.len();
    let mut comma = 0;
    let mut dtc = 0;
    let mut data;

    while i < len {
        let mut c = buf[i];
        if c < 32 || c == 127 || c == 255 {
            c = b'~'; // Manage unprintable chars
        }

        if debug {
            write!(out, " {}", c as char)?;
        } else {
            write!(out, "{}", c as char)?;
        }

        if c == b',' {
            comma = i;
            dtc = 1;
        }

        if dtc == 1 && buf[i] == 0 {
            data = (i + 4) & !3; // Align to 4 bytes
            for idx in (i + 1)..data {
                if idx < len {
                    if debug {
                        write!(out, " ~")?;
                    } else {
                        write!(out, "~")?;
                    }
                }
            }

            dtc = 0;
            let l = data;
            comma += 1;
            while comma < l && data < len {
                let tag = buf[comma];
                match tag {
                    b's' => {
                        let mut string_len = 0;
                        while data + string_len < len && buf[data + string_len] != 0 {
                            string_len += 1;
                        }
                        let k = (string_len + 4) & !3;
                        for _ in 0..k {
                            if data < len {
                                let mut ch = buf[data];
                                data += 1;
                                if ch < 32 || ch == 127 || ch == 255 {
                                    ch = b'~';
                                }
                                if debug {
                                    write!(out, " {}", ch as char)?;
                                } else {
                                    write!(out, "{}", ch as char)?;
                                }
                            }
                        }
                    }
                    b'i' => {
                        if data + 4 <= len {
                            let mut cursor = Cursor::new(&buf[data..data + 4]);
                            // `Xdump.c` behavior for int: `for (k = 4; k > 0; endian.c1[--k] = buf[data++]);`
                            // In a union littlebig, writing bytes backwards to c1 creates a big-endian read.
                            let val = cursor.read_i32::<byteorder::BigEndian>().unwrap_or(0);
                            write!(out, "[{:6}]", val)?;
                            data += 4;
                        } else {
                            data = len;
                        }
                    }
                    b'f' => {
                        if data + 4 <= len {
                            let mut cursor = Cursor::new(&buf[data..data + 4]);
                            let val = cursor.read_f32::<byteorder::BigEndian>().unwrap_or(0.0);
                            let abs_val = val.abs();
                            if abs_val < 10.0 {
                                write!(out, "[{:06.4}]", val)?;
                            } else if abs_val < 100.0 {
                                write!(out, "[{:06.3}]", val)?;
                            } else if abs_val < 1000.0 {
                                write!(out, "[{:06.2}]", val)?;
                            } else if abs_val < 10000.0 {
                                write!(out, "[{:06.1}]", val)?;
                            } else {
                                write!(out, "[{}]", val)?;
                            }
                            data += 4;
                        } else {
                            data = len;
                        }
                    }
                    b'b' => {
                        if data + 4 <= len {
                            let mut cursor = Cursor::new(&buf[data..data + 4]);
                            // `endian.i1` starts with bytes read backwards, so BigEndian.
                            let n = cursor.read_i32::<byteorder::BigEndian>().unwrap_or(0) as usize;
                            data += 4;

                            if data + 4 <= len {
                                let mut inner_cursor = Cursor::new(&buf[data..data + 4]);
                                // Next 4 bytes read forwards into `c1[k++]`. So LittleEndian.
                                let n_elements = inner_cursor.read_i32::<byteorder::LittleEndian>().unwrap_or(0) as usize;
                                data += 4;

                                if n == n_elements {
                                    write!(out, "{:3} chrs: ", n)?;
                                    for _ in 0..n {
                                        if data < len {
                                            write!(out, "{} ", buf[data] as char)?;
                                            data += 1;
                                        }
                                    }
                                } else {
                                    // Need to look at the start of buf for /meters/15 or /meters/16
                                    let is_meters_15 = buf.starts_with(b"/meters/15");
                                    let is_meters_16 = buf.starts_with(b"/meters/16");

                                    if is_meters_15 {
                                        let elements = n_elements * 2;
                                        write!(out, "{:3} rta: \n", elements)?;
                                        for j in 0..elements {
                                            if data + 2 <= len {
                                                let mut cur2 = Cursor::new(&buf[data..data + 2]);
                                                let val = cur2.read_i16::<byteorder::LittleEndian>().unwrap_or(0);
                                                let f1 = (val as f32) / 256.0;
                                                write!(out, "[{}] {:07.2} ", j, f1)?;
                                                data += 2;
                                            } else {
                                                break;
                                            }
                                        }
                                    } else if is_meters_16 {
                                        let elements = n_elements * 2;
                                        write!(out, "M/16: {} shorts\n", elements)?;
                                        for j in 0..elements {
                                            if data + 2 <= len {
                                                let mut cur2 = Cursor::new(&buf[data..data + 2]);
                                                let val = cur2.read_i16::<byteorder::LittleEndian>().unwrap_or(0);
                                                if j < 32 {
                                                    let f1 = (val as f32) / 32767.0;
                                                    write!(out, "[{}: G {:07.2}] ", j, f1)?;
                                                } else if j < 64 {
                                                    let f1 = (val as f32) / 32767.0;
                                                    write!(out, "[{}: C {:07.2}] ", j, f1)?;
                                                } else if j < 80 {
                                                    let f1 = (val as f32) / 32767.0;
                                                    write!(out, "[{}: B {:07.2}] ", j, f1)?;
                                                } else if j < 86 {
                                                    let f1 = (val as f32) / 32767.0;
                                                    write!(out, "[{}: M {:07.2}] ", j, f1)?;
                                                } else if j == 86 {
                                                    let f1 = (val as f32) / 32767.0;
                                                    write!(out, "[{}:LR {:07.2}] ", j, f1)?;
                                                } else if j == 87 {
                                                    let f1 = (val as f32) / 32767.0;
                                                    write!(out, "[{}:MC {:07.2}] ", j, f1)?;
                                                } else {
                                                    let f1 = (val as f32) / 256.0;
                                                    write!(out, "[{}: A {:07.2}] ", j, f1)?;
                                                }
                                                data += 2;
                                            } else {
                                                break;
                                            }
                                        }
                                        write!(out, "\n")?;
                                    } else {
                                        let elements = n_elements;
                                        write!(out, "{:3} flts: ", elements)?;
                                        for _ in 0..elements {
                                            if data + 4 <= len {
                                                let mut cur2 = Cursor::new(&buf[data..data + 4]);
                                                let val = cur2.read_f32::<byteorder::LittleEndian>().unwrap_or(0.0);
                                                write!(out, "{:06.2} ", val)?;
                                                data += 4;
                                            } else {
                                                break;
                                            }
                                        }
                                    }
                                }
                            } else {
                                data = len;
                            }
                        } else {
                            data = len;
                        }
                    }
                    _ => {}
                }
                comma += 1;
            }
            i = data.saturating_sub(1);
        }
        i += 1;
    }
    write!(out, "\n")?;
    Ok(())
}

fn process_buffer(buf: &[u8], debug: bool) -> Result<()> {
    let mut out = io::stdout();
    dump_osc_buffer(buf, debug, &mut out)?;
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut buf = Vec::new();
    if let Some(path) = &args.file {
        let file = File::open(path).context("Failed to open file")?;
        // Protect against unbounded reads
        let metadata = file.metadata()?;
        if metadata.len() > 1024 * 1024 * 10 { // 10 MB limit for raw dumps
             return Err(anyhow::anyhow!("File too large"));
        }
        file.take(1024 * 1024 * 10 + 1).read_to_end(&mut buf)?;
        if buf.len() > 1024 * 1024 * 10 {
            return Err(anyhow::anyhow!("File too large"));
        }
    } else {
        // Read from stdin
        io::stdin().read_to_end(&mut buf).context("Failed to read from standard input")?;
    }

    process_buffer(&buf, args.debug)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_meters_15() {
        let path = b"/meters/15\0\0";
        let tags = b",b\0\0";
        let mut blob = Vec::new();
        let blob_size: i32 = 8;
        blob.extend_from_slice(&blob_size.to_be_bytes());

        let n_elements: i32 = 1;
        blob.extend_from_slice(&n_elements.to_le_bytes());

        let val1: i16 = 256;
        let val2: i16 = -256;
        blob.extend_from_slice(&val1.to_le_bytes());
        blob.extend_from_slice(&val2.to_le_bytes());

        let mut buf = Vec::new();
        buf.extend_from_slice(path);
        buf.extend_from_slice(tags);
        buf.extend_from_slice(&blob);

        let mut out = Vec::new();
        dump_osc_buffer(&buf, false, &mut out).unwrap();

        let out_str = String::from_utf8_lossy(&out).into_owned();
        assert_eq!(out_str, "/meters/15~~,b~~  2 rta: \n[0] 0001.00 [1] -001.00 \n");
    }

    #[test]
    fn test_format_meters_16() {
        let path = b"/meters/16\0\0";
        let tags = b",b\0\0";
        let mut blob = Vec::new();

        let blob_size: i32 = 8;
        blob.extend_from_slice(&blob_size.to_be_bytes());

        let n_elements: i32 = 1;
        blob.extend_from_slice(&n_elements.to_le_bytes());

        let val1: i16 = 32767;
        let val2: i16 = -32767;
        blob.extend_from_slice(&val1.to_le_bytes());
        blob.extend_from_slice(&val2.to_le_bytes());

        let mut buf = Vec::new();
        buf.extend_from_slice(path);
        buf.extend_from_slice(tags);
        buf.extend_from_slice(&blob);

        let mut out = Vec::new();
        dump_osc_buffer(&buf, false, &mut out).unwrap();

        let out_str = String::from_utf8_lossy(&out).into_owned();
        assert_eq!(out_str, "/meters/16~~,b~~M/16: 2 shorts\n[0: G 0001.00] [1: G -001.00] \n\n");
    }
}
