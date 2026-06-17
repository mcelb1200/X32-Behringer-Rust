use crate::error::Result;
use crate::transport::MixerTransport;
use async_trait::async_trait;
use osc_lib::OscMessage;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;

/// An Ethernet UDP network transport implementation of `MixerTransport`.
pub struct UdpTransport {
    socket: Arc<UdpSocket>,
}

impl UdpTransport {
    /// Connects a new UdpTransport to the given IP address.
    pub async fn connect(ip: &str) -> Result<Self> {
        let has_port = ip.contains("]:")
            || (ip.contains(':') && ip.chars().filter(|&c| c == ':').count() == 1);

        let full_ip = if has_port {
            ip.to_string()
        } else if ip.contains(':') && !ip.starts_with('[') {
            format!("[{}]:10023", ip)
        } else {
            format!("{}:10023", ip)
        };
        let remote_addr: SocketAddr = full_ip.parse()?;

        let local_addr: SocketAddr = if remote_addr.is_ipv4() {
            "0.0.0.0:0".parse()?
        } else {
            "[::]:0".parse()?
        };

        let socket = UdpSocket::bind(local_addr).await?;
        socket.connect(remote_addr).await?;
        Ok(Self {
            socket: Arc::new(socket),
        })
    }
}

#[async_trait]
impl MixerTransport for UdpTransport {
    async fn send(&self, msg: OscMessage) -> Result<()> {
        let bytes = msg.to_bytes()?;
        self.socket.send(&bytes).await?;
        Ok(())
    }

    async fn recv(&self) -> Result<OscMessage> {
        let mut buf = vec![0u8; 1024 * 1024]; // 1MB Sentinel limit
        let len = self.socket.recv(&mut buf).await?;
        let msg = OscMessage::from_bytes(&buf[..len])?;
        Ok(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connect_invalid_ip_format() {
        let result = UdpTransport::connect("invalid_ip").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_connect_ipv4() {
        let result = UdpTransport::connect("127.0.0.1").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_connect_ipv6() {
        let result = UdpTransport::connect("::1").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_connect_ipv6_with_brackets() {
        let result = UdpTransport::connect("[::1]").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_connect_with_port() {
        let result = UdpTransport::connect("127.0.0.1:10024").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_connect_ipv6_with_port() {
        let result = UdpTransport::connect("[::1]:10024").await;
        assert!(result.is_ok());
    }
}
