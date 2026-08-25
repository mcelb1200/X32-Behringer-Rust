//! Port of Xcparse.c for parsing text strings into OSC messages

use x32_xsprint::{XsprintValue, xsprint};

/// Parses a string into an OSC byte buffer
pub fn xcparse(input: &str) -> Vec<u8> {
    let mut buf = Vec::new();

    // First, let's extract the OSC path.
    // In C logic:
    // It scans forward. It sets SPACE to 0 (effectively separating tokens).
    // It stops at the first COMMA.
    // The first token (before the first comma or space) is the path.

    let mut path = String::new();
    let mut formatters = String::new();
    let mut values_start_idx = 0;

    let mut has_comma = false;
    let mut in_path = true;

    // Collect path until space or comma
    for (i, c) in input.char_indices() {
        if c == ' ' && in_path {
            in_path = false;
        } else if c == ',' {
            has_comma = true;
            values_start_idx = i;
            break;
        } else if in_path {
            path.push(c);
        }
    }

    // xsprint the path
    xsprint(&mut buf, 's', XsprintValue::String(&path));

    if has_comma {
        // Collect formatters
        // formatters start after the comma and go until a character that is not 's', 'i', or 'f'

        let chars_after_comma = &input[values_start_idx + 1..];
        let mut k = 0;
        for c in chars_after_comma.chars() {
            if c == 's' || c == 'i' || c == 'f' {
                formatters.push(c);
                k += c.len_utf8();
            } else {
                break;
            }
        }

        let mut comma_formatters = String::from(",");
        comma_formatters.push_str(&formatters);

        xsprint(&mut buf, 's', XsprintValue::String(&comma_formatters));

        // Parse values based on formatters
        let mut data_str = &chars_after_comma[k..];

        for fmt in formatters.chars() {
            // skip leading spaces
            data_str = data_str.trim_start();
            if data_str.is_empty() {
                break;
            }

            match fmt {
                's' => {
                    let mut text = String::new();
                    let first_char = data_str.chars().next().unwrap();
                    if first_char == '"' || first_char == '\'' {
                        let quote = first_char;
                        data_str = &data_str[quote.len_utf8()..]; // skip quote
                        if let Some(end_idx) = data_str.find(quote) {
                            text.push_str(&data_str[..end_idx]);
                            data_str = &data_str[end_idx + quote.len_utf8()..];
                        } else {
                            // missing end quote, take rest
                            text.push_str(data_str);
                            data_str = "";
                        }
                    } else {
                        // read until space
                        if let Some(end_idx) = data_str.find(' ') {
                            text.push_str(&data_str[..end_idx]);
                            data_str = &data_str[end_idx..];
                        } else {
                            text.push_str(data_str);
                            data_str = "";
                        }
                    }
                    xsprint(&mut buf, 's', XsprintValue::String(&text));
                }
                'i' => {
                    let end_idx = data_str.find(' ').unwrap_or(data_str.len());
                    let token = &data_str[..end_idx];
                    if let Ok(val) = token.parse::<i32>() {
                        xsprint(&mut buf, 'i', XsprintValue::Int(val));
                    }
                    data_str = &data_str[end_idx..];
                }
                'f' => {
                    let end_idx = data_str.find(' ').unwrap_or(data_str.len());
                    let token = &data_str[..end_idx];
                    if let Ok(val) = token.parse::<f32>() {
                        xsprint(&mut buf, 'f', XsprintValue::Float(val));
                    }
                    data_str = &data_str[end_idx..];
                }
                _ => {}
            }
        }
    }

    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xcparse_simple_string() {
        let res = xcparse("/ch/01/mix/on ,s \"ON\"");
        let mut expected = Vec::new();
        expected.extend_from_slice(b"/ch/01/mix/on\0\0\0");
        expected.extend_from_slice(b",s\0\0");
        expected.extend_from_slice(b"ON\0\0");
        assert_eq!(res, expected);
    }

    #[test]
    fn test_xcparse_multiple_values() {
        let res = xcparse("/ch/01/mix/fader ,f 0.75");
        let mut expected = Vec::new();
        expected.extend_from_slice(b"/ch/01/mix/fader\0\0\0\0");
        expected.extend_from_slice(b",f\0\0");
        expected.extend_from_slice(&0.75f32.to_be_bytes());
        assert_eq!(res, expected);
    }

    #[test]
    fn test_xcparse_int() {
        let res = xcparse("/ch/01/mix/on ,i 1");
        let mut expected = Vec::new();
        expected.extend_from_slice(b"/ch/01/mix/on\0\0\0");
        expected.extend_from_slice(b",i\0\0");
        expected.extend_from_slice(&1i32.to_be_bytes());
        assert_eq!(res, expected);
    }

    #[test]
    fn test_xcparse_no_values() {
        let res = xcparse("/ch/01/mix/on");
        let mut expected = Vec::new();
        expected.extend_from_slice(b"/ch/01/mix/on\0\0\0");
        assert_eq!(res, expected);
    }

    #[test]
    fn test_xcparse_single_quotes() {
        let res = xcparse("/some/path ,s 'hello world'");
        let mut expected = Vec::new();
        expected.extend_from_slice(b"/some/path\0\0");
        expected.extend_from_slice(b",s\0\0");
        expected.extend_from_slice(b"hello world\0");
        assert_eq!(res, expected);
    }
}
