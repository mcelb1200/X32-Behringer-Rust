//! Parses and formats OSC buffers into formatted string logs, mirroring `X32logf.c`.

use std::fmt::Write;

/// Converts an OSC buffer into a formatted log string.
///
/// Mirrors the logic in `X32logf.c`.
///
/// # Arguments
/// * `header` - The prefix string to attach (e.g. "Rx").
/// * `buf` - The OSC message buffer as bytes.
///
/// # Returns
/// A formatted string representing the output of `X32logf()`.
pub fn x32_logf(header: &str, buf: &[u8]) -> String {
    let len = buf.len();
    let mut out = String::with_capacity(len * 2 + 32);

    let _ = write!(&mut out, "{}, {:4} B: ", header, len);

    let mut i = 0;
    let mut comma = 0;
    let mut dtc = 0;
    let mut data;

    while i < len {
        let mut c = buf[i];
        if c < 32 || c == 127 || c == 255 {
            c = b'~';
        }
        out.push(c as char);

        if c == b',' {
            comma = i;
            dtc = 1;
        }

        if dtc == 1 && buf[i] == 0 {
            data = (i + 4) & !3;

            // Print ~ for the remaining bytes in the alignment block
            for d in (i + 1)..data {
                if d < len {
                    out.push('~');
                }
            }

            dtc = 0;
            let l = data;

            comma += 1;
            while comma < l && data < len {
                match buf[comma] {
                    b's' => {
                        let mut str_len = 0;
                        while data + str_len < len && buf[data + str_len] != 0 {
                            str_len += 1;
                        }
                        let k = (str_len + 4) & !3;

                        for _ in 0..k {
                            if data < len {
                                let mut sc = buf[data];
                                if sc < 32 || sc == 127 || sc == 255 {
                                    sc = b'~';
                                }
                                out.push(sc as char);
                                data += 1;
                            }
                        }
                    }
                    b'i' => {
                        if data + 4 <= len {
                            let mut bytes = [0u8; 4];
                            bytes.copy_from_slice(&buf[data..data + 4]);
                            let val = i32::from_be_bytes(bytes);
                            let _ = write!(&mut out, "[{:6}]", val);
                            data += 4;
                        } else {
                            data = len;
                        }
                    }
                    b'f' => {
                        if data + 4 <= len {
                            let mut bytes = [0u8; 4];
                            bytes.copy_from_slice(&buf[data..data + 4]);
                            let val = f32::from_be_bytes(bytes);
                            if val < 10.0 {
                                let _ = write!(&mut out, "[{:06.4}]", val);
                            } else if val < 100.0 {
                                let _ = write!(&mut out, "[{:06.3}]", val);
                            } else if val < 1000.0 {
                                let _ = write!(&mut out, "[{:06.2}]", val);
                            } else if val < 10000.0 {
                                let _ = write!(&mut out, "[{:06.1}]", val);
                            } else {
                                let _ = write!(&mut out, "[{}]", val);
                            }
                            data += 4;
                        } else {
                            data = len;
                        }
                    }
                    b'b' => {
                        if data + 4 <= len {
                            let mut bytes = [0u8; 4];
                            bytes.copy_from_slice(&buf[data..data + 4]);
                            let n = i32::from_be_bytes(bytes);
                            data += 4;

                            if data + 4 <= len {
                                let mut type_bytes = [0u8; 4];
                                type_bytes.copy_from_slice(&buf[data..data + 4]);
                                let type_val = i32::from_le_bytes(type_bytes);
                                data += 4;

                                if n == type_val {
                                    let _ = write!(&mut out, "{} chrs: ", n);
                                    for _ in 0..n {
                                        if data < len {
                                            out.push(buf[data] as char);
                                            out.push(' ');
                                            data += 1;
                                        }
                                    }
                                } else {
                                    let _ = write!(&mut out, "{} flts: ", type_val);
                                    for _ in 0..type_val {
                                        if data + 4 <= len {
                                            let mut f_bytes = [0u8; 4];
                                            f_bytes.copy_from_slice(&buf[data..data + 4]);
                                            let f_val = f32::from_le_bytes(f_bytes);
                                            let _ = write!(&mut out, "{:06.2} ", f_val);
                                            data += 4;
                                        } else {
                                            data = len;
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

    out.push_str("\r\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logf_format_output() {
        let header = "Rx";
        let mut buf = Vec::new();
        buf.extend_from_slice(b"/ch/01/mix/fader\0\0\0\0");
        buf.extend_from_slice(b",f\0\0");
        let val: f32 = 0.75;
        buf.extend_from_slice(&val.to_be_bytes());

        let out = x32_logf(header, &buf);

        assert_eq!(out, "Rx,   28 B: /ch/01/mix/fader~~~~,f~~[0.7500]\r\n");
    }

    #[test]
    fn test_logf_int_format() {
        let header = "Tx";
        let mut buf = Vec::new();
        buf.extend_from_slice(b"/ch/02/mix/on\0\0\0");
        buf.extend_from_slice(b",i\0\0");
        let val: i32 = 1;
        buf.extend_from_slice(&val.to_be_bytes());

        let out = x32_logf(header, &buf);
        assert_eq!(out, "Tx,   24 B: /ch/02/mix/on~~~,i~~[     1]\r\n");
    }

    #[test]
    fn test_logf_string_format() {
        let header = "R";
        let mut buf = Vec::new();
        buf.extend_from_slice(b"/node\0\0\0");
        buf.extend_from_slice(b",s\0\0");
        buf.extend_from_slice(b"hello\0\0\0");

        let out = x32_logf(header, &buf);
        assert_eq!(out, "R,   20 B: /node~~~,s~~hello~~~\r\n");
    }

    #[test]
    fn test_logf_blob_string() {
        let header = "B";
        let mut buf = Vec::new();
        buf.extend_from_slice(b"/meters/0\0\0\0");
        buf.extend_from_slice(b",b\0\0");

        // Blobs with n == type_val (string blob)
        // Length of blob data = 8
        buf.extend_from_slice(&8i32.to_be_bytes());
        // type_val LE = 8
        buf.extend_from_slice(&8i32.to_le_bytes());
        buf.extend_from_slice(b"abcd"); // C code says for i=0..n, buf[data++]

        let out = x32_logf(header, &buf);
        assert_eq!(out, "B,   28 B: /meters/0~~~,b~~8 chrs: a b c d \r\n");
    }

    #[test]
    fn test_logf_blob_floats() {
        let header = "F";
        let mut buf = Vec::new();
        buf.extend_from_slice(b"/meters/1\0\0\0");
        buf.extend_from_slice(b",b\0\0");

        // Length of blob data = 8
        buf.extend_from_slice(&8i32.to_be_bytes());
        // type_val LE = 1 (1 float)
        buf.extend_from_slice(&1i32.to_le_bytes());
        // 1 float in LE
        let f: f32 = 1.23;
        buf.extend_from_slice(&f.to_le_bytes());

        let out = x32_logf(header, &buf);
        assert_eq!(out, "F,   28 B: /meters/1~~~,b~~1 flts: 001.23 \r\n");
    }
}
