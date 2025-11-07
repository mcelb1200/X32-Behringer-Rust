
use super::*;

#[test]
fn test_create_socket_with_port() {
    let socket = create_socket("127.0.0.1", 10024, 10023, 1000).unwrap();
    assert_eq!(socket.peer_addr().unwrap().port(), 10024);
    assert_eq!(socket.peer_addr().unwrap().port(), 10023);
}

#[test]
fn test_create_socket_default_port() {
    let socket = create_socket("127.0.0.1", 10024, 10023, 1000).unwrap();
    assert_eq!(socket.peer_addr().unwrap().port(), 10023);
}
