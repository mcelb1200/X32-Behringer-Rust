//! Port of Xcparse.c for parsing strings into OSC messages
//!
//! This module provides a single entrypoint function to parse a string input
//! into an OSC-formatted byte buffer.

use x32_xsprint::{XsprintValue, xsprint};

/// Parses an input string (e.g. `/ch/01/mix/fader ,f 0.75`) into an OSC byte buffer.
/// Returns the length of the written buffer.
pub fn xcparse(buf: &mut Vec<u8>, input: &str) -> usize {
    let mut chars: Vec<char> = input.chars().collect();
    let len = chars.len();

    let mut start_comma = None;

    // Find first comma and replace spaces before it with null terminators (in C it was a 0)
    // Actually in Rust we can just parse the string up to the comma and split spaces.
    for i in 0..len {
        if chars[i] == ' ' {
            chars[i] = '\0';
        }
        if chars[i] == ',' {
            start_comma = Some(i);
            break;
        }
    }

    // Write the path. Everything up to the first space or comma is the path.
    // In C, replacing spaces by '\0' effectively truncated the string at the first space.
    let path_end = start_comma.unwrap_or(len);
    let path_str: String = chars[0..path_end]
        .iter()
        .take_while(|&&c| c != '\0')
        .collect();

    xsprint(buf, 's', XsprintValue::String(&path_str));

    if let Some(mut comma_idx) = start_comma {
        // Look for end of formatters (s, i, f)
        let mut k = comma_idx + 1;
        while k < len && (chars[k] == 's' || chars[k] == 'i' || chars[k] == 'f') {
            k += 1;
        }

        // Prepare formatters string (e.g. ",f" or ",si")
        let formatters: String = chars[comma_idx..k].iter().collect();
        xsprint(buf, 's', XsprintValue::String(&formatters));

        comma_idx += 1; // skip leading comma
        let end_comma = k;

        while comma_idx < end_comma {
            // skip initial space (C code explicitly did k += 1 and skipped spaces)
            if k < len {
                k += 1;
            }
            while k < len && chars[k] == ' ' {
                k += 1;
            }

            let ccase = chars[comma_idx];
            comma_idx += 1;

            match ccase {
                's' => {
                    let mut txt_start = k;
                    if k < len && (chars[k] == '"' || chars[k] == '\'') {
                        let ccend = chars[k];
                        k += 1;
                        txt_start += 1;
                        while k < len && chars[k] != ccend {
                            k += 1;
                        }
                    } else {
                        while k < len && chars[k] != ' ' {
                            k += 1;
                        }
                    }

                    // Extract the string value
                    let k_safe = std::cmp::min(k, len);
                    let txt_start_safe = std::cmp::min(txt_start, k_safe);
                    let val: String = chars[txt_start_safe..k_safe].iter().collect();
                    xsprint(buf, 's', XsprintValue::String(&val));
                }
                'i' => {
                    let val_start = k;
                    while k < len && chars[k] != ' ' {
                        k += 1;
                    }
                    let k_safe = std::cmp::min(k, len);
                    let val_start_safe = std::cmp::min(val_start, k_safe);
                    let val_str: String = chars[val_start_safe..k_safe].iter().collect();
                    let val_int = val_str.parse::<i32>().unwrap_or(0);
                    xsprint(buf, 'i', XsprintValue::Int(val_int));
                }
                'f' => {
                    let val_start = k;
                    while k < len && chars[k] != ' ' {
                        k += 1;
                    }
                    let k_safe = std::cmp::min(k, len);
                    let val_start_safe = std::cmp::min(val_start, k_safe);
                    let val_str: String = chars[val_start_safe..k_safe].iter().collect();
                    let val_float = val_str.parse::<f32>().unwrap_or(0.0);
                    xsprint(buf, 'f', XsprintValue::Float(val_float));
                }
                _ => {}
            }
        }
    }

    buf.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xcparse_path_only() {
        let mut buf = Vec::new();
        xcparse(&mut buf, "/ch/01/mix/fader");
        let expected = b"/ch/01/mix/fader\0\0\0\0";
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_xcparse_float() {
        let mut buf = Vec::new();
        xcparse(&mut buf, "/ch/01/mix/fader ,f 0.75");
        let mut expected = b"/ch/01/mix/fader\0\0\0\0".to_vec();
        expected.extend_from_slice(b",f\0\0");
        expected.extend_from_slice(&0.75f32.to_be_bytes());
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_xcparse_int() {
        let mut buf = Vec::new();
        xcparse(&mut buf, "/ch/01/mix/on ,i 1");
        let mut expected = b"/ch/01/mix/on\0\0\0".to_vec();
        expected.extend_from_slice(b",i\0\0");
        expected.extend_from_slice(&1i32.to_be_bytes());
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_xcparse_string() {
        let mut buf = Vec::new();
        xcparse(&mut buf, "/info ,s test");
        let mut expected = b"/info\0\0\0".to_vec();
        expected.extend_from_slice(b",s\0\0");
        expected.extend_from_slice(b"test\0\0\0\0");
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_xcparse_string_quoted() {
        let mut buf = Vec::new();
        xcparse(&mut buf, "/info ,s \"test space\"");
        let mut expected = b"/info\0\0\0".to_vec();
        expected.extend_from_slice(b",s\0\0");
        expected.extend_from_slice(b"test space\0\0");
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_xcparse_string_single_quoted() {
        let mut buf = Vec::new();
        xcparse(&mut buf, "/info ,s 'test space'");
        let mut expected = b"/info\0\0\0".to_vec();
        expected.extend_from_slice(b",s\0\0");
        expected.extend_from_slice(b"test space\0\0");
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_xcparse_multiple() {
        let mut buf = Vec::new();
        xcparse(&mut buf, "/test ,ifs 1 0.5 \"hello\"");
        let mut expected = b"/test\0\0\0".to_vec();
        expected.extend_from_slice(b",ifs\0\0\0\0");
        expected.extend_from_slice(&1i32.to_be_bytes());
        expected.extend_from_slice(&0.5f32.to_be_bytes());
        expected.extend_from_slice(b"hello\0\0\0");
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_xcparse_space_truncation() {
        let mut buf = Vec::new();
        // C logic truncated path at the first space if it wasn't a comma
        xcparse(&mut buf, "/test path_should_truncate ,i 1");
        let mut expected = b"/test\0\0\0".to_vec();
        expected.extend_from_slice(b",i\0\0");
        expected.extend_from_slice(&1i32.to_be_bytes());
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_xcparse_extra_spaces() {
        let mut buf = Vec::new();
        xcparse(&mut buf, "/test   ,f    0.75");
        let mut expected = b"/test\0\0\0".to_vec();
        expected.extend_from_slice(b",f\0\0");
        expected.extend_from_slice(&0.75f32.to_be_bytes());
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_xcparse_truncated_input() {
        let mut buf = Vec::new();
        xcparse(&mut buf, "/test ,i");
        let mut expected = b"/test\0\0\0".to_vec();
        expected.extend_from_slice(b",i\0\0");
        expected.extend_from_slice(&0i32.to_be_bytes());
        assert_eq!(buf, expected);
    }
}
