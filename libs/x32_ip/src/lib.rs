//! IPv4 dotted notation validation module, porting `validateIP4Dotted.c`.
//!
//! Provides a simple validation for IP4 type addresses expecting a string
//! in the form 123.123.123.123, allowing up to 3 digits per component and
//! values between 0 and 255.

/// Simple validation of IP4 type IP address.
///
/// Expects an IP address in the form `123.123.123.123`
/// Returns `true` if OK, `false` otherwise.
pub fn validate_ip4_dotted(s: &str) -> bool {
    let len = s.len();
    if !(7..=15).contains(&len) {
        return false;
    }

    let mut parts_count = 0;

    // Use an iterator to avoid Vec allocation for splitting
    for part in s.split('.') {
        parts_count += 1;
        let trimmed = part.trim();

        if trimmed.is_empty() {
            return false;
        }

        let digits = trimmed.chars().take_while(|c| c.is_ascii_digit()).count();
        if digits == 0 || digits > 3 {
            return false;
        }

        match trimmed.parse::<u32>() {
            Ok(val) if val <= 255 => {}
            _ => return false,
        }
    }

    parts_count == 4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_ips() {
        assert!(validate_ip4_dotted("1.2.3.4"));
        assert!(validate_ip4_dotted("1.2.3.4  "));
        assert!(validate_ip4_dotted("255.255.255.255"));
        assert!(validate_ip4_dotted("0.0.0.0"));
        assert!(validate_ip4_dotted("192.168.1.1"));
        assert!(validate_ip4_dotted("001.002.003.004"));
        assert!(validate_ip4_dotted("1.2. 3.4"));
        assert!(validate_ip4_dotted(" 1.2.3.4"));
    }

    #[test]
    fn test_invalid_ips() {
        assert!(!validate_ip4_dotted("-1.2.3.4"));
        assert!(!validate_ip4_dotted("1.2.3.456"));
        assert!(!validate_ip4_dotted("1.2.3.4.5"));
        assert!(!validate_ip4_dotted("1.2.3.4a"));
        assert!(!validate_ip4_dotted("1234.1.1.1"));
        assert!(!validate_ip4_dotted("256.1.1.1"));
        assert!(!validate_ip4_dotted("1.2.3"));
        assert!(!validate_ip4_dotted(""));
        assert!(!validate_ip4_dotted("1.2.3.4.5.6"));
        // Length bounds checks (7 to 15)
        assert!(!validate_ip4_dotted("1.2.3")); // Length < 7
        assert!(!validate_ip4_dotted("111.222.333.4444")); // Length > 15
    }
}
