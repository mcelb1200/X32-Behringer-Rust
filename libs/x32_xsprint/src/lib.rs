//! Port of Xsprint.c for encoding OSC-formatted data into buffers
//!
//! This module provides functions to append data to a byte buffer
//! in OSC (Open Sound Control) format, which means big-endian numerical
//! encoding and 4-byte string alignment.

/// Represents a value to be formatted by xsprint.
pub enum XsprintValue<'a> {
    /// A string value.
    String(&'a str),
    /// A 32-bit float value.
    Float(f32),
    /// A 32-bit integer value.
    Int(i32),
    /// No value (used when format doesn't require a payload).
    None,
}

/// Appends the value to the buffer based on the provided format char,
/// returning the new length of the buffer.
///
/// Format options:
/// - 's': appends a string, null-terminates it, and pads to 4 bytes boundary.
/// - 'f': appends a 32-bit float in big-endian order.
/// - 'i': appends a 32-bit integer in big-endian order.
///
/// Any other format character performs no copy.
pub fn xsprint(bd: &mut Vec<u8>, format: char, bs: XsprintValue) -> usize {
    match format {
        's' => {
            if let XsprintValue::String(s) = bs {
                bd.extend_from_slice(s.as_bytes());
            }
            bd.push(0);

            // align to 4 bytes boundary
            while bd.len() % 4 != 0 {
                bd.push(0);
            }
        }
        'f' | 'i' => {
            match bs {
                XsprintValue::Float(f) => {
                    bd.extend_from_slice(&f.to_be_bytes());
                }
                XsprintValue::Int(i) => {
                    bd.extend_from_slice(&i.to_be_bytes());
                }
                _ => {} // Original C did nothing if types didn't match memory interpretation, but here we ignore gracefully
            }
        }
        _ => {}
    }
    bd.len()
}

/// Appends a text address (or parameter), followed by the format comma-string,
/// and then appends the value, mimicking an OSC message generation.
pub fn xfprint(bd: &mut Vec<u8>, text: &str, format: char, bs: XsprintValue) -> usize {
    // first copy text
    bd.extend_from_slice(text.as_bytes());
    bd.push(0);

    // align to 4 bytes boundary
    while bd.len() % 4 != 0 {
        bd.push(0);
    }

    // then set format, keeping #4 alignment
    bd.push(b',');
    bd.push(format as u8);
    bd.push(0);
    bd.push(0); // This pads it to exactly 4 bytes (',', format, 0, 0)

    // based on format, set value
    xsprint(bd, format, bs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xsprint_string() {
        let mut buf = Vec::new();
        let len = xsprint(&mut buf, 's', XsprintValue::String("hello"));
        assert_eq!(len, 8);
        assert_eq!(buf, b"hello\0\0\0");
    }

    #[test]
    fn test_xsprint_string_empty() {
        let mut buf = Vec::new();
        let len = xsprint(&mut buf, 's', XsprintValue::String(""));
        assert_eq!(len, 4);
        assert_eq!(buf, b"\0\0\0\0");
    }

    #[test]
    fn test_xsprint_string_none() {
        let mut buf = Vec::new();
        let len = xsprint(&mut buf, 's', XsprintValue::None);
        assert_eq!(len, 4);
        assert_eq!(buf, b"\0\0\0\0");
    }

    #[test]
    fn test_xsprint_float() {
        let mut buf = Vec::new();
        let len = xsprint(&mut buf, 'f', XsprintValue::Float(1.0));
        assert_eq!(len, 4);
        assert_eq!(buf, 1.0f32.to_be_bytes());
    }

    #[test]
    fn test_xsprint_int() {
        let mut buf = Vec::new();
        let len = xsprint(&mut buf, 'i', XsprintValue::Int(42));
        assert_eq!(len, 4);
        assert_eq!(buf, 42i32.to_be_bytes());
    }

    #[test]
    fn test_xsprint_invalid_format() {
        let mut buf = Vec::new();
        let len = xsprint(&mut buf, 'x', XsprintValue::Int(42));
        assert_eq!(len, 0);
        assert!(buf.is_empty());
    }

    #[test]
    fn test_xfprint() {
        let mut buf = Vec::new();
        let len = xfprint(&mut buf, "/ch/01/mix/fader", 'f', XsprintValue::Float(0.75));

        let mut expected = b"/ch/01/mix/fader\0\0\0\0".to_vec();
        expected.extend_from_slice(b",f\0\0");
        expected.extend_from_slice(&0.75f32.to_be_bytes());

        assert_eq!(len, expected.len());
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_xfprint_string() {
        let mut buf = Vec::new();
        let len = xfprint(&mut buf, "/info", 's', XsprintValue::String("test"));

        let mut expected = b"/info\0\0\0".to_vec();
        expected.extend_from_slice(b",s\0\0");
        expected.extend_from_slice(b"test\0\0\0\0");

        assert_eq!(len, expected.len());
        assert_eq!(buf, expected);
    }
}
