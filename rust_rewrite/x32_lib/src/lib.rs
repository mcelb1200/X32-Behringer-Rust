
use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;
use std::io;

pub fn create_socket(ip: &str, timeout_ms: u64) -> io::Result<UdpSocket> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_read_timeout(Some(Duration::from_millis(timeout_ms)))?;

    let x32_addr: SocketAddr = format!("{}:10023", ip).parse()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    socket.connect(x32_addr)?;

    Ok(socket)
}
