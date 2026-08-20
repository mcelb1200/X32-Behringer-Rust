//! Port of Xcparse.c for parsing text strings into OSC-formatted binary data.
//!
//! This module provides a function to parse a formatted text string
//! (like `/ch/01/mix/fader ,f 0.75`) and append it as a serialized
//! OSC message to a byte buffer.

use x32_xsprint::{xsprint, XsprintValue};

/// Parses a text string representation of an OSC message and appends the
/// serialized binary format to the provided byte buffer.
///
/// The input string typically takes the form:
/// `<address> [,<format_string>] [arg1 arg2 ...]`
/// where:
/// - `<address>` is the OSC path (e.g., `/ch/01/mix/fader`)
/// - `<format_string>` starts with a comma and contains characters like `s`, `i`, `f`
/// - `argN` are the arguments, separated by spaces.
///
/// If strings contain spaces, they can be enclosed in single or double quotes.
///
/// Returns the new length of the buffer.
pub fn xcparse(buf: &mut Vec<u8>, input: &str) -> usize {
    let input = input.trim();
    if input.is_empty() {
        return buf.len();
    }

    // Find the format string, which is the first word that starts with a comma.
    // The original C code looks for the first comma. Let's just find the first comma.
    let (address_part, rest) = match input.find(',') {
        Some(idx) => {
            let addr = input[..idx].trim();
            // Split off the rest, starting with the comma
            (addr, Some(input[idx..].trim()))
        }
        None => (input.trim(), None),
    };

    // First, push the address string
    xsprint(buf, 's', XsprintValue::String(address_part));

    if let Some(rest_str) = rest {
        // Find the end of the format string (the comma followed by 's', 'i', 'f')
        // We'll read the word starting with comma.
        let end_format = rest_str
            .find(' ')
            .unwrap_or(rest_str.len());
        let format_str = &rest_str[..end_format];
        let mut data_str = rest_str[end_format..].trim_start();

        // Push the format string
        xsprint(buf, 's', XsprintValue::String(format_str));

        // Skip the initial comma in format_str to iterate over types
        if format_str.starts_with(',') {
            for c in format_str[1..].chars() {
                if data_str.is_empty() {
                    break;
                }
                match c {
                    's' => {
                        // Read string. Could be quoted.
                        let (s, remaining) = parse_string(data_str);
                        xsprint(buf, 's', XsprintValue::String(&s));
                        data_str = remaining.trim_start();
                    }
                    'i' => {
                        // Read integer
                        let (i_str, remaining) = parse_word(data_str);
                        let val: i32 = i_str.parse().unwrap_or(0);
                        xsprint(buf, 'i', XsprintValue::Int(val));
                        data_str = remaining.trim_start();
                    }
                    'f' => {
                        // Read float
                        let (f_str, remaining) = parse_word(data_str);
                        let val: f32 = f_str.parse().unwrap_or(0.0);
                        xsprint(buf, 'f', XsprintValue::Float(val));
                        data_str = remaining.trim_start();
                    }
                    _ => {
                        // Unknown format character, skip
                    }
                }
            }
        }
    }

    buf.len()
}

/// Helper to parse a single string argument, respecting quotes.
fn parse_string(input: &str) -> (String, &str) {
    if input.starts_with('"') || input.starts_with('\'') {
        let quote_char = input.chars().next().unwrap();
        // find closing quote
        if let Some(end_idx) = input[1..].find(quote_char) {
            let s = &input[1..=end_idx];
            let remaining = &input[end_idx + 2..];
            (s.to_string(), remaining)
        } else {
            // unmatched quote, take till end
            let s = &input[1..];
            (s.to_string(), "")
        }
    } else {
        // take until space
        let (w, r) = parse_word(input);
        (w.to_string(), r)
    }
}

/// Helper to parse a single space-separated word.
fn parse_word(input: &str) -> (&str, &str) {
    match input.find(' ') {
        Some(idx) => (&input[..idx], &input[idx..]),
        None => (input, ""),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xcparse_address_only() {
        let mut buf = Vec::new();
        let len = xcparse(&mut buf, "/info");
        assert_eq!(len, 8);
        assert_eq!(buf, b"/info\0\0\0");
    }

    #[test]
    fn test_xcparse_with_float() {
        let mut buf = Vec::new();
        let len = xcparse(&mut buf, "/ch/01/mix/fader ,f 0.75");

        let mut expected = b"/ch/01/mix/fader\0\0\0\0".to_vec();
        expected.extend_from_slice(b",f\0\0");
        expected.extend_from_slice(&0.75f32.to_be_bytes());

        assert_eq!(len, expected.len());
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_xcparse_with_int() {
        let mut buf = Vec::new();
        let len = xcparse(&mut buf, "/config/routing ,i 1");

        // length of /config/routing is 15, so it needs one null byte to reach 16
        // making the address block padded to 16.
        let mut expected = b"/config/routing\0".to_vec();
        expected.extend_from_slice(b",i\0\0");
        expected.extend_from_slice(&1i32.to_be_bytes());

        assert_eq!(buf, expected);
        assert_eq!(len, expected.len());
    }

    #[test]
    fn test_xcparse_with_string() {
        let mut buf = Vec::new();
        let len = xcparse(&mut buf, "/ch/01/config/name ,s Kick");

        let mut expected = b"/ch/01/config/name\0\0".to_vec();
        expected.extend_from_slice(b",s\0\0");
        expected.extend_from_slice(b"Kick\0\0\0\0");

        assert_eq!(len, expected.len());
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_xcparse_with_quoted_string() {
        let mut buf = Vec::new();
        let len = xcparse(&mut buf, "/ch/01/config/name ,s \"Kick Drum\"");

        let mut expected = b"/ch/01/config/name\0\0".to_vec();
        expected.extend_from_slice(b",s\0\0");
        expected.extend_from_slice(b"Kick Drum\0\0\0");

        assert_eq!(len, expected.len());
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_xcparse_with_single_quoted_string() {
        let mut buf = Vec::new();
        let len = xcparse(&mut buf, "/ch/01/config/name ,s 'Kick Drum'");

        let mut expected = b"/ch/01/config/name\0\0".to_vec();
        expected.extend_from_slice(b",s\0\0");
        expected.extend_from_slice(b"Kick Drum\0\0\0");

        assert_eq!(len, expected.len());
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_xcparse_multiple_args() {
        let mut buf = Vec::new();
        let len = xcparse(&mut buf, "/something ,ifs 42 0.5 \"test string\"");

        let mut expected = b"/something\0\0".to_vec();
        expected.extend_from_slice(b",ifs\0\0\0\0");
        expected.extend_from_slice(&42i32.to_be_bytes());
        expected.extend_from_slice(&0.5f32.to_be_bytes());
        expected.extend_from_slice(b"test string\0");

        assert_eq!(len, expected.len());
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_xcparse_empty_string() {
        let mut buf = Vec::new();
        let len = xcparse(&mut buf, "");
        assert_eq!(len, 0);
        assert_eq!(buf, b"");
    }
}