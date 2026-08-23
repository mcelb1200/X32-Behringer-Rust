//! Parses text strings into OSC messages, porting `Xcparse.c` from the original C codebase.
//!
//! This module provides the `xcparse` function which takes a formatted input string
//! and encodes it into an OSC byte buffer using `x32_xsprint`.

use x32_xsprint::{xsprint, XsprintValue};

/// Parses an input string (e.g., "/ch/01/mix/fader ,f 0.75") and encodes it into the provided
/// buffer as an OSC message.
///
/// Strings can be unquoted or quoted with `"` or `'`.
/// Format characters supported: `s` (string), `i` (integer), `f` (float).
///
/// Returns the number of bytes written to the buffer (i.e. the new length of `buf`).
pub fn xcparse(buf: &mut Vec<u8>, input: &str) -> usize {
    let mut chars = input.chars().peekable();
    let mut input_line = String::new();

    // First part: extract the address until a comma or space is found.
    while let Some(&c) = chars.peek() {
        if c == ' ' {
            // C code Replaces spaces by 0 before the comma, essentially ending the string.
            // We just consume the space and stop address parsing if we find a comma next,
            // or we just consider the address done.
            break;
        } else if c == ',' {
            break;
        }
        input_line.push(c);
        chars.next();
    }

    // Skip spaces between address and comma
    while let Some(&' ') = chars.peek() {
        chars.next();
    }

    // If no comma found, we just append the address as a string and return.
    if chars.peek() != Some(&',') {
        return xsprint(buf, 's', XsprintValue::String(&input_line));
    }

    // Found comma, consume it
    chars.next();

    let mut formatters = String::new();
    // Parse formatters (s, i, f)
    while let Some(&c) = chars.peek() {
        if c == 's' || c == 'i' || c == 'f' {
            formatters.push(c);
            chars.next();
        } else {
            break;
        }
    }

    // In C code, the address is written first.
    xsprint(buf, 's', XsprintValue::String(&input_line));

    // In C code, the format string is written next (including the leading comma).
    let mut format_str = String::from(",");
    format_str.push_str(&formatters);
    xsprint(buf, 's', XsprintValue::String(&format_str));

    // Now process each formatter
    for format_char in formatters.chars() {
        // Skip spaces before the value
        while let Some(&' ') = chars.peek() {
            chars.next();
        }

        match format_char {
            's' => {
                let mut txt = String::new();
                if let Some(&quote_char) = chars.peek() {
                    if quote_char == '"' || quote_char == '\'' {
                        chars.next(); // consume opening quote
                        while let Some(c) = chars.next() {
                            if c == quote_char {
                                break;
                            }
                            txt.push(c);
                        }
                    } else {
                        // Unquoted string, read until space
                        while let Some(&c) = chars.peek() {
                            if c == ' ' {
                                break;
                            }
                            txt.push(c);
                            chars.next();
                        }
                    }
                }
                xsprint(buf, 's', XsprintValue::String(&txt));
            }
            'i' => {
                let mut int_str = String::new();
                // Read until space or end
                while let Some(&c) = chars.peek() {
                    if c == ' ' {
                        break;
                    }
                    int_str.push(c);
                    chars.next();
                }
                let val: i32 = int_str.parse().unwrap_or(0);
                xsprint(buf, 'i', XsprintValue::Int(val));

                // C code skips spaces after
                while let Some(&' ') = chars.peek() {
                    chars.next();
                }
            }
            'f' => {
                let mut float_str = String::new();
                // Read until space or end
                while let Some(&c) = chars.peek() {
                    if c == ' ' {
                        break;
                    }
                    float_str.push(c);
                    chars.next();
                }
                let val: f32 = float_str.parse().unwrap_or(0.0);
                xsprint(buf, 'f', XsprintValue::Float(val));

                // C code skips spaces after
                while let Some(&' ') = chars.peek() {
                    chars.next();
                }
            }
            _ => {}
        }
    }

    // Actually the C code returns the running index 'i'.
    // `xsprint` returns the new length of the buffer.
    // So `buf.len()` at the end is the returned value in the C code,
    // assuming 'i' started at 0, which it does.
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
    fn test_xcparse_integer() {
        let mut buf = Vec::new();
        let len = xcparse(&mut buf, "/config/routing/IN/01-08 ,i 0");

        let mut expected = b"/config/routing/IN/01-08\0\0\0\0".to_vec();
        expected.extend_from_slice(b",i\0\0");
        expected.extend_from_slice(&0i32.to_be_bytes());

        assert_eq!(len, expected.len());
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_xcparse_unquoted_string() {
        let mut buf = Vec::new();
        let len = xcparse(&mut buf, "/ch/01/config/name ,s Vocals");

        let mut expected = b"/ch/01/config/name\0\0".to_vec();
        expected.extend_from_slice(b",s\0\0");
        expected.extend_from_slice(b"Vocals\0\0");

        assert_eq!(len, expected.len());
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_xcparse_quoted_string() {
        let mut buf = Vec::new();
        let len = xcparse(&mut buf, "/ch/01/config/name ,s \"Lead Vocals\"");

        let mut expected = b"/ch/01/config/name\0\0".to_vec();
        expected.extend_from_slice(b",s\0\0");
        expected.extend_from_slice(b"Lead Vocals\0");

        assert_eq!(len, expected.len());
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_xcparse_multiple_args() {
        let mut buf = Vec::new();
        let len = xcparse(&mut buf, "/some/path ,sif \"hello\" 42 3.14");

        let mut expected = b"/some/path\0\0".to_vec();
        expected.extend_from_slice(b",sif\0\0\0\0");
        expected.extend_from_slice(b"hello\0\0\0");
        expected.extend_from_slice(&42i32.to_be_bytes());
        expected.extend_from_slice(&3.14f32.to_be_bytes());

        assert_eq!(len, expected.len());
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_xcparse_no_args() {
        let mut buf = Vec::new();
        let len = xcparse(&mut buf, "/info");

        let expected = b"/info\0\0\0".to_vec();

        assert_eq!(len, expected.len());
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_xcparse_malformed_input() {
        // Just ensure it doesn't panic on missing values for formatters
        let mut buf = Vec::new();
        let len = xcparse(&mut buf, "/ch/01 ,f");

        let mut expected = b"/ch/01\0\0".to_vec();
        expected.extend_from_slice(b",f\0\0");
        expected.extend_from_slice(&0.0f32.to_be_bytes()); // Expect default 0.0

        assert_eq!(len, expected.len());
        assert_eq!(buf, expected);
    }
}
