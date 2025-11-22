use super::*;

#[test]
fn test_message_to_bytes_and_back() {
    let message = OscMessage {
        path: "/test".to_string(),
        args: vec![
            OscArg::Int(123),
            OscArg::Float(456.789),
            OscArg::String("hello".to_string()),
        ],
    };

    let bytes = message.to_bytes().unwrap();
    let parsed_message = OscMessage::from_bytes(&bytes).unwrap();

    assert_eq!(message, parsed_message);
}

#[test]
fn test_string_padding() {
    let mut bytes = Vec::new();
    write_osc_string(&mut bytes, "hello").unwrap();
    assert_eq!(bytes, vec![104, 101, 108, 108, 111, 0, 0, 0]);
}

#[test]
fn test_message_from_str() {
    let s = "/ch/01/mix/fader ,f 0.75";
    let message = OscMessage::from_str(s).unwrap();
    assert_eq!(message.path, "/ch/01/mix/fader");
    assert_eq!(message.args.len(), 1);
    match &message.args[0] {
        OscArg::Float(f) => assert!((f - 0.75).abs() < f32::EPSILON),
        _ => panic!("Incorrect argument type"),
    }
}

#[test]
fn test_message_from_str_with_quoted_string_no_space() {
    let s = "/ch/01/config/name ,s\"MyFader\"";
    let message = OscMessage::from_str(s).unwrap();
    assert_eq!(message.path, "/ch/01/config/name");
    assert_eq!(message.args.len(), 1);
    match &message.args[0] {
        OscArg::String(s) => assert_eq!(s, "MyFader"),
        _ => panic!("Incorrect argument type"),
    }
}

#[test]
fn test_message_with_blob_roundtrip_to_bytes() {
    let original_message = OscMessage {
        path: "/blob".to_string(),
        args: vec![OscArg::Blob(vec![0x01, 0x02, 0x03, 0x04])],
    };

    let bytes = original_message.to_bytes().unwrap();
    let roundtrip_message = OscMessage::from_bytes(&bytes).unwrap();

    assert_eq!(original_message, roundtrip_message);
}

#[test]
fn test_message_with_blob_roundtrip_to_string() {
    let original_message = OscMessage {
        path: "/blob".to_string(),
        args: vec![OscArg::Blob(vec![0x01, 0x02, 0x03, 0x04])],
    };

    let s = original_message.to_string();
    let roundtrip_message = OscMessage::from_str(&s).unwrap();

    assert_eq!(original_message, roundtrip_message);
}

#[test]
fn test_message_from_str_with_blob() {
    let s = "/blobtest ,b 0123456789abcdef";
    let message = OscMessage::from_str(s).unwrap();
    assert_eq!(message.path, "/blobtest");
    assert_eq!(message.args.len(), 1);
    match &message.args[0] {
        OscArg::Blob(b) => assert_eq!(b, &vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef]),
        _ => panic!("Incorrect argument type"),
    }
}

#[test]
fn test_message_from_str_missing_value() {
    let s = "/test ,i";
    let result = OscMessage::from_str(s);
    assert!(matches!(result, Err(OscError::ParseError(_))));
}

#[test]
fn test_message_from_str_unsupported_type() {
    let s = "/test ,x 123";
    let result = OscMessage::from_str(s);
    assert!(matches!(result, Err(OscError::UnsupportedTypeTag('x'))));
}

#[test]
fn test_message_from_str_extra_arguments() {
    let s = "/test ,i 123 456";
    let result = OscMessage::from_str(s);
    assert!(matches!(result, Err(OscError::ParseError(_))));
}

#[test]
fn test_message_to_string() {
    let message = OscMessage {
        path: "/ch/01/mix/fader".to_string(),
        args: vec![OscArg::Float(0.75)],
    };
    let s = message.to_string();
    assert_eq!(s, "/ch/01/mix/fader ,f 0.75");
}

#[test]
fn test_message_roundtrip_to_string() {
    let original_message = OscMessage {
        path: "/test/path".to_string(),
        args: vec![
            OscArg::Int(42),
            OscArg::Float(3.14159),
            OscArg::String("hello world".to_string()),
        ],
    };

    let s = original_message.to_string();
    let roundtrip_message = OscMessage::from_str(&s).unwrap();

    assert_eq!(original_message, roundtrip_message);
}

#[test]
fn test_message_from_str_with_quoted_string() {
    let s = "/ch/01/config/name ,s \"My Fader\"";
    let message = OscMessage::from_str(s).unwrap();
    assert_eq!(message.path, "/ch/01/config/name");
    assert_eq!(message.args.len(), 1);
    match &message.args[0] {
        OscArg::String(s) => assert_eq!(s, "My Fader"),
        _ => panic!("Incorrect argument type"),
    }
}

#[test]
fn test_tokenize_quoted_string_with_no_space_after() {
    let s = "/cmd ,ss \"hello\"no-space";
    let message = OscMessage::from_str(s).unwrap();
    assert_eq!(message.path, "/cmd");
    assert_eq!(message.args.len(), 2);
    match &message.args[0] {
        OscArg::String(s) => assert_eq!(s, "hello"),
        _ => panic!("Incorrect argument type"),
    }
    match &message.args[1] {
        OscArg::String(s) => assert_eq!(s, "no-space"),
        _ => panic!("Incorrect argument type"),
    }
}
