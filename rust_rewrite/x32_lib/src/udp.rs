//! # UDP Module
//!
//! Provides UDP communication functionality for interacting with X32/M32 mixers.
//!
//! This module contains the `X32Socket` struct, which encapsulates a UDP socket and provides
//! methods for sending and receiving data. It also includes a `connect` function for
//! establishing and verifying a a connection to the mixer.

use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::time::Duration;
use crate::error::{Result, X32Error};
use osc_lib::{OscMessage, OscError};

/// A UDP socket for communication with an X32/M32 console.
///
/// This struct wraps a `std::net::UdpSocket` and the server's `SocketAddr`.
///
/// # Example
///
/// ```
/// use std::time::Duration;
/// use x32_lib::udp::X32Socket;
///
/// let server_addr = "127.0.0.1:10023";
/// let socket = X32Socket::new(server_addr).unwrap();
/// ```
pub struct X32Socket {
    socket: UdpSocket,
    server: SocketAddr,
}

impl X32Socket {
    /// Creates a new `X32Socket` and binds it to a local address.
    ///
    /// The local address is chosen automatically. For IPv4, it will be "0.0.0.0:0",
    /// and for IPv6, it will be "[::]:0".
    ///
    /// # Arguments
    ///
    /// * `server` - The address of the X32/M32 console (e.g., "192.168.1.65:10023").
    pub fn new<A: ToSocketAddrs>(server: A) -> Result<Self> {
        let server_addr = server.to_socket_addrs()?.next().ok_or_else(|| X32Error::String("Invalid server address".to_string()))?;

        // Bind to a local address.
        // For IPv4, this will be "0.0.0.0:0".
        // For IPv6, this will be "[::]:0".
        let bind_addr: SocketAddr = if server_addr.is_ipv4() {
            "0.0.0.0:0".parse().unwrap()
        } else {
            "[::]:0".parse().unwrap()
        };

        let socket = UdpSocket::bind(bind_addr)?;
        Ok(X32Socket { socket, server: server_addr })
    }

    /// Sends data to the X32/M32 console.
    ///
    /// # Arguments
    ///
    /// * `buf` - The buffer containing the data to send.
    pub fn send(&self, buf: &[u8]) -> Result<usize> {
        Ok(self.socket.send_to(buf, self.server)?)
    }

    /// Receives data from the X32/M32 console.
    ///
    /// # Arguments
    ///
    /// * `buf` - The buffer to store the received data in.
    /// * `timeout` - The timeout for the receive operation.
    pub fn recv(&self, buf: &mut [u8], timeout: Duration) -> Result<usize> {
        self.socket.set_read_timeout(Some(timeout))?;
        let (len, _) = self.socket.recv_from(buf)?;
        Ok(len)
    }
}

/// Establishes and verifies a connection to an X32/M32 console.
///
/// This function creates a new `X32Socket`, sends an `/info` command to the console,
/// and waits for a valid response to confirm that the connection is live.
///
/// # Arguments
///
/// * `server` - The address of the X32/M32 console (e.g., "192.168.1.65:10023").
/// * `timeout` - The timeout for the connection attempt.
///
/// # Example
///
/// ```
/// use std::time::Duration;
/// use std::net::UdpSocket;
/// use x32_lib::udp;
///
/// // This test requires a mock server to be running on the specified address.
/// // let server_addr = "127.0.0.1:10023";
/// // let socket = udp::connect(server_addr, Duration::from_secs(1)).unwrap();
/// ```
pub fn connect<A: ToSocketAddrs>(server: A, timeout: Duration) -> Result<X32Socket> {
    let x32_socket = X32Socket::new(server)?;
    let msg = OscMessage::new("/info".to_string(), vec![]);
    let msg_bytes = msg.to_bytes()?;

    x32_socket.send(&msg_bytes)?;

    let mut buf = [0u8; 512];
    let len = x32_socket.recv(&mut buf, timeout)?;

    let response = OscMessage::from_bytes(&buf[..len])?;
    if response.path == "/info" {
        Ok(x32_socket)
    } else {
        Err(X32Error::from(OscError::ParseError("Unexpected Response".to_string())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    fn mock_server() -> UdpSocket {
        UdpSocket::bind("127.0.0.1:0").unwrap()
    }

    #[test]
    fn test_connect() {
        let server = mock_server();
        let server_addr = server.local_addr().unwrap();

        let server_thread = thread::spawn(move || {
            let mut buf = [0u8; 512];
            let (len, src) = server.recv_from(&mut buf).unwrap();
            let msg = OscMessage::from_bytes(&buf[..len]).unwrap();
            assert_eq!(msg.path, "/info");

            let response = OscMessage::new("/info".to_string(), vec![]);
            let response_bytes = response.to_bytes().unwrap();
            server.send_to(&response_bytes, src).unwrap();
        });

        let client = connect(server_addr, Duration::from_millis(500)).unwrap();
        assert_eq!(client.server, server_addr);

        server_thread.join().unwrap();
    }

    #[test]
    fn test_send_and_recv() {
        let server = mock_server();
        let server_addr = server.local_addr().unwrap();
        let client = X32Socket::new(server_addr).unwrap();

        let server_thread = thread::spawn(move || {
            let mut buf = [0u8; 512];
            let (len, src) = server.recv_from(&mut buf).unwrap();
            let received_message = String::from_utf8_lossy(&buf[..len]);
            assert_eq!(received_message, "hello");

            let response = "world";
            server.send_to(response.as_bytes(), src).unwrap();
        });

        let message = "hello";
        client.send(message.as_bytes()).unwrap();

        let mut buf = [0u8; 512];
        let len = client.recv(&mut buf, Duration::from_millis(500)).unwrap();
        let received_message = String::from_utf8_lossy(&buf[..len]);
        assert_eq!(received_message, "world");

        server_thread.join().unwrap();
    }
}
