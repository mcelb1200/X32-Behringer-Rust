
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
fn test_message_to_string() {
    let message = OscMessage {
        path: "/ch/01/mix/fader".to_string(),
        args: vec![OscArg::Float(0.75)],
    };
    let s = message.to_string();
    assert_eq!(s, "/ch/01/mix/fader ,f 0.75");
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
