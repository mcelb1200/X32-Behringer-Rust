//! Parses text strings into OSC messages, porting the original `Xcparse.c`.

use x32_xsprint::{XsprintValue, xsprint};

/// Parses a command string into an OSC byte buffer.
///
/// The input format mimics the `Xcparse.c` C code:
/// `command,formats data`
/// Example: `/ch/01/mix/fader ,f 0.75`
///
/// Returns the length of the written data.
pub fn xcparse(buf: &mut Vec<u8>, input: &str) -> usize {
    buf.clear();

    // Replicating logic of Xcparse.c
    // Note: C implementation replaced spaces with \0 up to the first comma,
    // but Rust's String split is cleaner. We will match the C logic of splitting parts.

    // Find the first comma which separates the OSC address and the format string.
    let start_comma = input.find(',');

    if let Some(comma_idx) = start_comma {
        // OSC Address path before the comma
        let path = input[..comma_idx].trim();
        xsprint(buf, 's', XsprintValue::String(path));

        // Format string starts at comma
        let after_comma = &input[comma_idx + 1..];

        // Find end of formatters
        let mut k = 0;
        let mut format_chars = Vec::new();
        for (i, c) in after_comma.char_indices() {
            if c == 's' || c == 'i' || c == 'f' {
                format_chars.push(c);
                k = i + 1;
            } else {
                break;
            }
        }

        // Format string representation, prepended with ','
        let mut format_str = String::from(",");
        format_str.push_str(&after_comma[..k]);
        xsprint(buf, 's', XsprintValue::String(&format_str));

        // Parse remaining values based on format_chars
        let data_part = &after_comma[k..];

        let mut char_indices = data_part.char_indices().peekable();

        for ccase in format_chars {
            // skip leading spaces
            while let Some(&(_, c)) = char_indices.peek() {
                if c == ' ' {
                    char_indices.next();
                } else {
                    break;
                }
            }

            if char_indices.peek().is_none() {
                break; // No more data to parse for this formatter
            }

            match ccase {
                's' => {
                    let mut txt_start = None;
                    let mut is_quoted = false;
                    let mut quote_char = ' ';

                    if let Some(&(idx, c)) = char_indices.peek() {
                        if c == '"' || c == '\'' {
                            is_quoted = true;
                            quote_char = c;
                            char_indices.next(); // Consume quote
                            if let Some(&(next_idx, _)) = char_indices.peek() {
                                txt_start = Some(next_idx);
                            }
                        } else {
                            txt_start = Some(idx);
                        }
                    }

                    let mut end_idx = data_part.len();
                    if let Some(start) = txt_start {
                        while let Some(&(idx, c)) = char_indices.peek() {
                            if is_quoted {
                                if c == quote_char {
                                    end_idx = idx;
                                    char_indices.next(); // Consume closing quote
                                    break;
                                }
                            } else {
                                if c == ' ' {
                                    end_idx = idx;
                                    break;
                                }
                            }
                            char_indices.next();
                        }

                        // Bounds checking for slicing
                        if start <= end_idx && end_idx <= data_part.len() {
                            let s = &data_part[start..end_idx];
                            xsprint(buf, 's', XsprintValue::String(s));
                        }
                    }
                }
                'i' => {
                    let mut num_str = String::new();
                    while let Some(&(_, c)) = char_indices.peek() {
                        if c == ' ' {
                            break;
                        }
                        num_str.push(c);
                        char_indices.next();
                    }
                    if let Ok(val) = num_str.parse::<i32>() {
                        xsprint(buf, 'i', XsprintValue::Int(val));
                    }
                }
                'f' => {
                    let mut num_str = String::new();
                    while let Some(&(_, c)) = char_indices.peek() {
                        if c == ' ' {
                            break;
                        }
                        num_str.push(c);
                        char_indices.next();
                    }
                    if let Ok(val) = num_str.parse::<f32>() {
                        xsprint(buf, 'f', XsprintValue::Float(val));
                    }
                }
                _ => {}
            }
        }
    } else {
        // No comma found, treat whole input as string path
        let path = input.trim();
        xsprint(buf, 's', XsprintValue::String(path));
    }

    buf.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xcparse_float() {
        let mut buf = Vec::new();
        let len = xcparse(&mut buf, "/ch/01/mix/fader ,f 0.75");

        let mut expected = b"/ch/01/mix/fader\0\0\0\0".to_vec();
        expected.extend_from_slice(b",f\0\0");
        expected.extend_from_slice(&0.75f32.to_be_bytes());

        assert_eq!(len, expected.len());
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_xcparse_int() {
        let mut buf = Vec::new();
        let len = xcparse(&mut buf, "/ch/01/mix/on ,i 1");

        let mut expected = b"/ch/01/mix/on\0\0\0".to_vec();
        expected.extend_from_slice(b",i\0\0");
        expected.extend_from_slice(&1i32.to_be_bytes());

        assert_eq!(len, expected.len());
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_xcparse_string() {
        let mut buf = Vec::new();
        let len = xcparse(&mut buf, "/ch/01/config/name ,s \"Lead Vox\"");

        let mut expected = b"/ch/01/config/name\0\0".to_vec();
        expected.extend_from_slice(b",s\0\0");
        expected.extend_from_slice(b"Lead Vox\0\0\0\0");

        assert_eq!(len, expected.len());
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_xcparse_multiple() {
        let mut buf = Vec::new();
        let len = xcparse(&mut buf, "/test ,isf 42 \"hello\" 3.14");

        let mut expected = b"/test\0\0\0".to_vec();
        expected.extend_from_slice(b",isf\0\0\0\0");
        expected.extend_from_slice(&42i32.to_be_bytes());
        expected.extend_from_slice(b"hello\0\0\0");
        expected.extend_from_slice(&3.14f32.to_be_bytes());

        assert_eq!(len, expected.len());
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_xcparse_no_comma() {
        let mut buf = Vec::new();
        let len = xcparse(&mut buf, "/status");

        let expected = b"/status\0".to_vec();

        assert_eq!(len, expected.len());
        assert_eq!(buf, expected);
    }
}
