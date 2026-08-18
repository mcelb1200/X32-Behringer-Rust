//! Parses text strings into OSC messages (port of Xcparse.c).
//! Takes a string like `/ch/01/mix/fader ,f 0.5` and returns an OscMessage.

use osc_lib::{OscArg, OscMessage};

/// Parses a string into an OscMessage.
/// The input format is typically `<path> ,<format> <arg1> <arg2> ...`
/// Note that if there are no arguments, it's just `<path>`.
pub fn xcparse(input: &str) -> Option<OscMessage> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }

    // Split on first comma
    let (path_part, rest) = if let Some(idx) = input.find(',') {
        (&input[..idx], &input[idx..])
    } else {
        // No comma means just a path
        return Some(OscMessage {
            path: input.split_whitespace().next()?.to_string(),
            args: vec![],
        });
    };

    let path = path_part.split_whitespace().next()?.to_string();

    // Parse formatters
    let mut parts = rest.split_whitespace();
    let format_str = parts.next()?;

    // The format string includes the comma, e.g. ",f" or ",sif"
    if !format_str.starts_with(',') {
        return None;
    }

    let formatters = &format_str[1..];
    let mut args = Vec::new();

    // The rest of the string contains arguments
    // To handle quoted strings correctly, we need a slightly more complex parser
    // than just split_whitespace.

    let arg_str = rest[format_str.len()..].trim_start();
    let mut chars = arg_str.char_indices().peekable();

    for f in formatters.chars() {
        // Skip leading whitespace
        while let Some(&(_, c)) = chars.peek() {
            if c.is_whitespace() {
                chars.next();
            } else {
                break;
            }
        }

        if chars.peek().is_none() {
            break; // Ran out of arguments
        }

        match f {
            's' => {
                let &(_start_idx, first_char) = chars.peek().unwrap();
                let mut string_val = String::new();

                if first_char == '"' || first_char == '\'' {
                    // Quoted string
                    let quote_char = first_char;
                    chars.next(); // Consume opening quote

                    while let Some((_, c)) = chars.next() {
                        if c == quote_char {
                            break;
                        }
                        string_val.push(c);
                    }
                } else {
                    // Unquoted string (stops at whitespace)
                    while let Some((_, c)) = chars.next() {
                        if c.is_whitespace() {
                            break;
                        }
                        string_val.push(c);
                    }
                }
                args.push(OscArg::String(string_val));
            }
            'i' => {
                let mut int_str = String::new();
                while let Some(&(_, c)) = chars.peek() {
                    if c.is_whitespace() {
                        break;
                    }
                    int_str.push(c);
                    chars.next();
                }
                if let Ok(val) = int_str.parse::<i32>() {
                    args.push(OscArg::Int(val));
                } else {
                    // Fallback to 0 if parsing fails
                    args.push(OscArg::Int(0));
                }
            }
            'f' => {
                let mut float_str = String::new();
                while let Some(&(_, c)) = chars.peek() {
                    if c.is_whitespace() {
                        break;
                    }
                    float_str.push(c);
                    chars.next();
                }
                if let Ok(val) = float_str.parse::<f32>() {
                    args.push(OscArg::Float(val));
                } else {
                    // Fallback to 0.0 if parsing fails
                    args.push(OscArg::Float(0.0));
                }
            }
            _ => {
                // Unknown formatter
            }
        }
    }

    Some(OscMessage { path, args })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xcparse_simple_path() {
        let msg = xcparse("/info").unwrap();
        assert_eq!(msg.path, "/info");
        assert!(msg.args.is_empty());
    }

    #[test]
    fn test_xcparse_path_with_whitespace() {
        let msg = xcparse("  /info  ").unwrap();
        assert_eq!(msg.path, "/info");
        assert!(msg.args.is_empty());
    }

    #[test]
    fn test_xcparse_float() {
        let msg = xcparse("/ch/01/mix/fader ,f 0.75").unwrap();
        assert_eq!(msg.path, "/ch/01/mix/fader");
        assert_eq!(msg.args.len(), 1);
        if let OscArg::Float(f) = msg.args[0] {
            assert_eq!(f, 0.75);
        } else {
            panic!("Expected float");
        }
    }

    #[test]
    fn test_xcparse_int() {
        let msg = xcparse("/ch/01/mix/on ,i 1").unwrap();
        assert_eq!(msg.path, "/ch/01/mix/on");
        assert_eq!(msg.args.len(), 1);
        if let OscArg::Int(i) = msg.args[0] {
            assert_eq!(i, 1);
        } else {
            panic!("Expected int");
        }
    }

    #[test]
    fn test_xcparse_string() {
        let msg = xcparse("/ch/01/config/name ,s \"Lead Vox\"").unwrap();
        assert_eq!(msg.path, "/ch/01/config/name");
        assert_eq!(msg.args.len(), 1);
        if let OscArg::String(ref s) = msg.args[0] {
            assert_eq!(s, "Lead Vox");
        } else {
            panic!("Expected string");
        }
    }

    #[test]
    fn test_xcparse_unquoted_string() {
        let msg = xcparse("/ch/01/config/name ,s LeadVox").unwrap();
        assert_eq!(msg.path, "/ch/01/config/name");
        assert_eq!(msg.args.len(), 1);
        if let OscArg::String(ref s) = msg.args[0] {
            assert_eq!(s, "LeadVox");
        } else {
            panic!("Expected string");
        }
    }

    #[test]
    fn test_xcparse_multiple_args() {
        let msg = xcparse("/some/path ,sif \"test\" 42 3.14").unwrap();
        assert_eq!(msg.path, "/some/path");
        assert_eq!(msg.args.len(), 3);

        if let OscArg::String(ref s) = msg.args[0] {
            assert_eq!(s, "test");
        } else {
            panic!("Expected string");
        }

        if let OscArg::Int(i) = msg.args[1] {
            assert_eq!(i, 42);
        } else {
            panic!("Expected int");
        }

        if let OscArg::Float(f) = msg.args[2] {
            assert_eq!(f, 3.14);
        } else {
            panic!("Expected float");
        }
    }
}
