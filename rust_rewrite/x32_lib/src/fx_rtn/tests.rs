
use super::data::*;

#[test]
fn test_xfxrtn01_data() {
    assert_eq!(XFXRTN01[0].command, "/fxrtn");
    assert_eq!(XFXRTN01[1].command, "/fxrtn/01");
    assert_eq!(XFXRTN01[2].command, "/fxrtn/01/config");
    assert_eq!(XFXRTN01[3].command, "/fxrtn/01/config/name");
}
