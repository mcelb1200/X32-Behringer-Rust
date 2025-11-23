use super::*;

#[path = "tests/common.rs"]
mod common;
#[path = "tests/error.rs"]
mod error;

#[test]
fn test_create_socket_with_port() {
    let socket = create_socket("127.0.0.1:10024", 1000).unwrap();
    assert_eq!(socket.peer_addr().unwrap().port(), 10024);
}

#[test]
fn test_create_socket_default_port() {
    let socket = create_socket("127.0.0.1", 1000).unwrap();
    assert_eq!(socket.peer_addr().unwrap().port(), 10023);
}
