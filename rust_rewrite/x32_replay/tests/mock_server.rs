use std::net::UdpSocket;

fn main() {
    let socket = UdpSocket::bind("127.0.0.1:10023").unwrap();
    let mut buf = [0; 512];
    loop {
        let (amt, src) = socket.recv_from(&mut buf).unwrap();
        let buf = &mut buf[..amt];
        if buf == b"/info\x00\x00\x00" {
            socket.send_to(b"/info\x00\x00\x00,s\x00\x00/some/info", src).unwrap();
        }
    }
}
