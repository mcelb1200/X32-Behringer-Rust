//! Provides functions to validate IP addresses in a dotted format.
//! Port of validateIP4Dotted.c

use std::str::FromStr;
use std::net::Ipv4Addr;

/// Validates an IPv4 address string in dotted decimal notation.
///
/// Returns true if it's a valid IPv4 string like "192.168.1.100",
/// false otherwise.
pub fn validate_ipv4_dotted(s: &str) -> bool {
    // Note: C implementation didn't trim whitespace, so " 192..." would fail the length/content check.
    // However, Ipv4Addr::from_str handles some weird cases, we should stick closer to the original c behavior
    // that enforces exactly "XXX.XXX.XXX.XXX".

    if s.len() < 7 || s.len() > 15 {
        return false;
    }

    // Check if any character is not a digit or a dot
    for c in s.chars() {
        if !c.is_ascii_digit() && c != '.' {
            return false;
        }
    }

    // Use standard library parser for the actual value validation
    Ipv4Addr::from_str(s).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_ips() {
        assert!(validate_ipv4_dotted("192.168.1.1"));
        assert!(validate_ipv4_dotted("0.0.0.0"));
        assert!(validate_ipv4_dotted("255.255.255.255"));
        assert!(validate_ipv4_dotted("10.0.0.1"));
        assert!(validate_ipv4_dotted("127.0.0.1"));
    }

    #[test]
    fn test_invalid_ips() {
        assert!(!validate_ipv4_dotted(""));
        assert!(!validate_ipv4_dotted("1.2.3"));
        assert!(!validate_ipv4_dotted("1.2.3.4.5"));
        assert!(!validate_ipv4_dotted("256.0.0.1"));
        assert!(!validate_ipv4_dotted("192.168.1.256"));
        assert!(!validate_ipv4_dotted("abc.def.ghi.jkl"));
        assert!(!validate_ipv4_dotted("192.168.1.-1"));
        assert!(!validate_ipv4_dotted("192.168.1.1 "));
        assert!(!validate_ipv4_dotted(" 192.168.1.1"));
    }
}
