use std::net::UdpSocket;
use x32_lib::create_socket;

#[test]
fn test_create_socket_ipv4() {
    let server = UdpSocket::bind("127.0.0.1:0").unwrap();
    let server_addr = server.local_addr().unwrap();

    let client = create_socket(&server_addr.to_string(), 100).unwrap();
    client.send(b"test").unwrap();

    let mut buf = [0; 10];
    let (len, from) = server.recv_from(&mut buf).unwrap();

    assert_eq!(len, 4);
    assert_eq!(&buf[..len], b"test");
    assert_eq!(from, client.local_addr().unwrap());
}

#[test]
fn test_create_socket_ipv6() {
    let server = match UdpSocket::bind("[::1]:0") {
        Ok(socket) => socket,
        Err(e) => {
            if e.raw_os_error() == Some(97) {
                // EAFNOSUPPORT on Linux
                println!("IPv6 not supported, skipping test.");
                return;
            }
            panic!("Failed to create IPv6 socket: {}", e);
        }
    };
    let server_addr = server.local_addr().unwrap();

    let client = create_socket(&server_addr.to_string(), 100).unwrap();
    client.send(b"test").unwrap();

    let mut buf = [0; 10];
    let (len, from) = server.recv_from(&mut buf).unwrap();

    assert_eq!(len, 4);
    assert_eq!(&buf[..len], b"test");
    assert_eq!(from, client.local_addr().unwrap());
}
