use x32_xsprint::{xsprint, XsprintValue};

/// Parses a string formatted text input into a byte buffer for OSC.
/// Mimics the behavior of Xcparse.c.
/// It splits at the first comma to handle OSC paths and formats,
/// then parses subsequent text, ints, or floats according to the formatters.
pub fn xcparse(buf: &mut Vec<u8>, input: &str) -> usize {
    let mut input_line: Vec<char> = input.chars().collect();
    let input_len = input_line.len();
    let mut start_comma = None;

    // Jump to next comma, replace spaces by 0 (null) prior to comma
    for i in 0..input_len {
        if input_line[i] == ' ' {
            input_line[i] = '\0';
        }
        if input_line[i] == ',' {
            start_comma = Some(i);
            break;
        }
    }

    let initial_len = buf.len();

    // Prepare first command block
    // the first string (OSC path) ends at start_comma or end of string if no comma
    let path_end = start_comma.unwrap_or(input_len);
    let path_str: String = input_line[0..path_end]
        .iter()
        .take_while(|&&c| c != '\0')
        .collect();

    xsprint(buf, 's', XsprintValue::String(&path_str));

    if let Some(mut s_comma) = start_comma {
        // Look for end of formatters (s, i, f)
        let mut k = s_comma + 1;
        while k < input_len && (input_line[k] == 's' || input_line[k] == 'i' || input_line[k] == 'f') {
            k += 1;
        }

        // Prepare command formatters' block
        // In C, input_line[k] = 0. We slice from s_comma to k.
        let formatters_str: String = input_line[s_comma..k].iter().collect();
        xsprint(buf, 's', XsprintValue::String(&formatters_str));

        s_comma += 1; // skip the formatters' leading comma
        let end_comma = k;

        while s_comma < end_comma {
            // ignore spaces before new block
            k += 1;
            while k < input_len && input_line[k] == ' ' {
                k += 1;
            }

            if s_comma >= input_line.len() {
                break; // Safety bounds check
            }
            let ccase = input_line[s_comma];
            s_comma += 1;

            match ccase {
                's' => {
                    let mut txt_start = k;
                    if k < input_len && (input_line[k] == '"' || input_line[k] == '\'') {
                        let ccend = input_line[k];
                        k += 1;
                        txt_start += 1;
                        while k < input_len && input_line[k] != ccend {
                            k += 1;
                        }
                    } else {
                        while k < input_len && input_line[k] != ' ' {
                            k += 1;
                        }
                    }

                    let safe_start = std::cmp::min(txt_start, input_len);
                    let safe_end = std::cmp::max(safe_start, std::cmp::min(k, input_len));
                    let s_val: String = input_line[safe_start..safe_end].iter().collect();
                    xsprint(buf, 's', XsprintValue::String(&s_val));
                }
                'i' => {
                    let start_k = std::cmp::min(k, input_len);
                    while k < input_len && input_line[k] != ' ' {
                        k += 1;
                    }
                    let int_str: String = input_line[start_k..std::cmp::min(k, input_len)].iter().collect();
                    if let Ok(val) = int_str.parse::<i32>() {
                        xsprint(buf, 'i', XsprintValue::Int(val));
                    }
                }
                'f' => {
                    let start_k = std::cmp::min(k, input_len);
                    while k < input_len && input_line[k] != ' ' {
                        k += 1;
                    }
                    let float_str: String = input_line[start_k..std::cmp::min(k, input_len)].iter().collect();
                    if let Ok(val) = float_str.parse::<f32>() {
                        xsprint(buf, 'f', XsprintValue::Float(val));
                    }
                }
                _ => {}
            }
        }
    }

    buf.len() - initial_len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xcparse_simple_int() {
        let mut buf = Vec::new();
        let len = xcparse(&mut buf, "/ch/01/mix/on ,i 1");

        let mut expected = b"/ch/01/mix/on\0\0\0".to_vec();
        expected.extend_from_slice(b",i\0\0");
        expected.extend_from_slice(&1i32.to_be_bytes());

        assert_eq!(len, expected.len());
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_xcparse_simple_float() {
        let mut buf = Vec::new();
        let len = xcparse(&mut buf, "/ch/01/mix/fader ,f 0.75");

        let mut expected = b"/ch/01/mix/fader\0\0\0\0".to_vec();
        expected.extend_from_slice(b",f\0\0");
        expected.extend_from_slice(&0.75f32.to_be_bytes());

        assert_eq!(len, expected.len());
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_xcparse_simple_string() {
        let mut buf = Vec::new();
        let len = xcparse(&mut buf, "/ch/01/config/name ,s \"Kick\"");

        let mut expected = b"/ch/01/config/name\0\0".to_vec();
        expected.extend_from_slice(b",s\0\0");
        expected.extend_from_slice(b"Kick\0\0\0\0");

        assert_eq!(len, expected.len());
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_xcparse_single_quotes_string() {
        let mut buf = Vec::new();
        let len = xcparse(&mut buf, "/ch/01/config/name ,s 'Snare'");

        let mut expected = b"/ch/01/config/name\0\0".to_vec();
        expected.extend_from_slice(b",s\0\0");
        expected.extend_from_slice(b"Snare\0\0\0");

        assert_eq!(len, expected.len());
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_xcparse_multiple_args() {
        let mut buf = Vec::new();
        let len = xcparse(&mut buf, "/test ,ifs 42 0.5 \"hello\"");

        let mut expected = b"/test\0\0\0".to_vec();
        expected.extend_from_slice(b",ifs\0\0\0\0");
        expected.extend_from_slice(&42i32.to_be_bytes());
        expected.extend_from_slice(&0.5f32.to_be_bytes());
        expected.extend_from_slice(b"hello\0\0\0");

        assert_eq!(len, expected.len());
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_xcparse_no_args() {
        let mut buf = Vec::new();
        let len = xcparse(&mut buf, "/test");

        let expected = b"/test\0\0\0".to_vec();

        assert_eq!(len, expected.len());
        assert_eq!(buf, expected);
    }

    #[test]
    fn test_xcparse_missing_args() {
        let mut buf = Vec::new();
        let len = xcparse(&mut buf, "/test ,if 42");

        let mut expected = b"/test\0\0\0".to_vec();
        expected.extend_from_slice(b",if\0");
        expected.extend_from_slice(&42i32.to_be_bytes());

        assert_eq!(len, expected.len());
        assert_eq!(buf, expected);
    }
}
