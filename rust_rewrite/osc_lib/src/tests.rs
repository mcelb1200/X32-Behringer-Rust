
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
