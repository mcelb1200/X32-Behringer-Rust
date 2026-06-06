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
        let full_ip = if (ip.contains(':') && !ip.starts_with('[')) || ip.contains("]:") {
            ip.to_string()
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
