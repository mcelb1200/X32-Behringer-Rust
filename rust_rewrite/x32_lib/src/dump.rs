
use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;
use std::str;

fn align(n: usize) -> usize {
    (n + 3) & !3
}

pub fn xdump(buf: &[u8], debug: bool) -> String {
    let mut result = String::new();
    let len = buf.len();

    let path_end = buf.iter().position(|&b| b == 0).unwrap_or(len);
    let path = String::from_utf8_lossy(&buf[..path_end]);

    for c in path.chars() {
        let display_char = if c.is_ascii_graphic() { c } else { '~' };
        if debug {
            result.push(' ');
        }
        result.push(display_char);
    }

    let mut data_start = align(path_end + 1);

    if data_start > len || buf[path_end + 1] != b',' {
        result.push('\n');
        return result;
    }

    let type_tag_end = (path_end + 1..len).find(|&i| buf[i] == 0).unwrap_or(len);
    let type_tags = String::from_utf8_lossy(&buf[path_end + 2..type_tag_end]);

    for i in path_end..data_start {
        if i < len {
            let c = buf[i] as char;
            let display_char = if c.is_ascii_graphic() || c == '\0' { c } else { '~' };
             if c == '\0' {
                if debug { result.push(' '); }
                result.push('~');
            } else {
                if debug { result.push(' '); }
                result.push(display_char);
            }
        }
    }


    let mut cursor = Cursor::new(&buf[data_start..]);

    for tag in type_tags.chars() {
        match tag {
            's' => {
                let s_start = cursor.position() as usize;
                let s_end = (s_start..len - data_start).find(|&i| buf[data_start + i] == 0).unwrap_or(len - data_start);
                let s_slice = &buf[data_start + s_start..data_start + s_end];
                let s = String::from_utf8_lossy(s_slice);

                let aligned_len = align(s.len() + 1);

                for c in s.chars() {
                    let display_char = if c.is_ascii_graphic() { c } else { '~' };
                     if debug { result.push(' '); }
                    result.push(display_char);
                }

                for _ in 0..aligned_len - s.len() {
                    if debug { result.push(' ');}
                    result.push('~');
                }
                cursor.set_position((s_start + aligned_len) as u64);
            }
            'i' => {
                if let Ok(i) = cursor.read_i32::<BigEndian>() {
                    result.push_str(&format!("[{:6}]", i));
                }
            }
            'f' => {
                if let Ok(f) = cursor.read_f32::<BigEndian>() {
                    if f < 10.0 {
                        result.push_str(&format!("[{:06.4}]", f));
                    } else if f < 100.0 {
                        result.push_str(&format!("[{:06.3}]", f));
                    } else if f < 1000.0 {
                        result.push_str(&format!("[{:06.2}]", f));
                    } else {
                        result.push_str(&format!("[{:06.1}]", f));
                    }
                }
            }
            'b' => {
                 if let Ok(_size) = cursor.read_i32::<BigEndian>() {
                    // Blob handling is complex and depends on the OSC path.
                    // This is a simplified version.
                    result.push_str("[blob]");
                }
            }
            _ => {}
        }
    }

    result.push('\n');
    result
}

pub fn xfdump(header: &str, buf: &[u8], debug: bool) -> String {
    let mut result = String::new();
    if debug {
        result.push_str(&format!("{}, {:4} B: ", header, buf.len()));
        for byte in buf {
            result.push_str(&format!("{:02x}", byte));
        }
        result.push('\n');
    }

    if debug {
        result.push_str(&" ".repeat(header.len() + 10));
    } else {
        result.push_str(&format!("{}, {:4} B: ", header, buf.len()));
    }

    result.push_str(&xdump(buf, debug));
    result
}
