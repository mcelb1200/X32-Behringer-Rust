#[cfg(test)]
mod tests {
    use crate::error::*;
    use osc_lib::OscError;
    use std::io;

    #[test]
    fn test_error_from_io() {
        let io_err = io::Error::new(io::ErrorKind::Other, "test io error");
        let err: X32Error = io_err.into();
        match err {
            X32Error::Io(_) => (),
            _ => panic!("Expected Io error variant"),
        }
    }

    #[test]
    fn test_error_from_osc() {
        let osc_err = OscError::ParseError("test osc error".to_string());
        let err: X32Error = osc_err.into();
        match err {
            X32Error::Osc(_) => (),
            _ => panic!("Expected Osc error variant"),
        }
    }

    #[test]
    fn test_error_from_string() {
        let err: X32Error = "test custom error".to_string().into();
        match err {
            X32Error::Custom(s) => assert_eq!(s, "test custom error"),
            _ => panic!("Expected Custom error variant"),
        }
    }

    #[test]
    fn test_error_display() {
        let err: X32Error = "test error".to_string().into();
        assert_eq!(format!("{}", err), "X32 error: test error");
    }
}
