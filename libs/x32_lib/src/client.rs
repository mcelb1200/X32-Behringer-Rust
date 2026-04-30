use crate::error::{Result, X32Error};
use osc_lib::{OscArg, OscError, OscMessage};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::{broadcast, mpsc, Mutex, watch};
use tokio::time::{self, Duration};

const MAX_PACKET_SIZE: usize = 1024 * 1024; // 1MB Sentinel limit

/// A client for interacting with a Behringer X32/Midas M32 mixer.
pub struct MixerClient {
    socket: Arc<UdpSocket>,
    msg_tx: broadcast::Sender<OscMessage>,
    heartbeat_tx: watch::Sender<bool>,
    _receiver_handle: tokio::task::JoinHandle<()>,
    _heartbeat_handle: tokio::task::JoinHandle<()>,
}

impl MixerClient {
    /// Connects to a mixer at the given IP address.
    pub async fn connect(ip: &str, heartbeat: bool) -> Result<Self> {
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
        let socket = Arc::new(socket);

        let (msg_tx, _) = broadcast::channel(100);
        let msg_tx_clone = msg_tx.clone();
        let socket_clone = socket.clone();

        // Background receiver task
        let _receiver_handle = tokio::spawn(async move {
            let mut buf = vec![0u8; MAX_PACKET_SIZE];
            loop {
                match socket_clone.recv(&mut buf).await {
                    Ok(len) => {
                        if let Ok(msg) = OscMessage::from_bytes(&buf[..len]) {
                            let _ = msg_tx_clone.send(msg);
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        let (heartbeat_tx, mut heartbeat_rx) = watch::channel(heartbeat);
        let socket_heartbeat = socket.clone();
        let _heartbeat_handle = tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(9));
            let heartbeat_msg = OscMessage::new("/xremote".to_string(), vec![]);
            let bytes = heartbeat_msg.to_bytes().unwrap();
            loop {
                interval.tick().await;
                if *heartbeat_rx.borrow() {
                    if let Err(_) = socket_heartbeat.send(&bytes).await {
                        break;
                    }
                }
            }
        });

        Ok(Self {
            socket,
            msg_tx,
            heartbeat_tx,
            _receiver_handle,
            _heartbeat_handle,
        })
    }

    /// Starts the auto-heartbeat task.
    pub fn start_heartbeat(&self) {
        let _ = self.heartbeat_tx.send(true);
    }

    /// Stops the auto-heartbeat task.
    pub fn stop_heartbeat(&self) {
        let _ = self.heartbeat_tx.send(false);
    }

    /// Sends an OSC message to the mixer.
    pub async fn send_message(&self, path: &str, args: Vec<OscArg>) -> Result<()> {
        let msg = OscMessage::new(path.to_string(), args);
        let bytes = msg.to_bytes()?;
        self.socket.send(&bytes).await?;
        Ok(())
    }

    /// Queries a value from the mixer.
    /// 
    /// Note: This waits for the next message matching the path. 
    /// In a multi-threaded environment, this might catch a message intended for someone else
    /// if they are querying the same path, but for X32 it's generally safe.
    pub async fn query_value(&self, path: &str) -> Result<OscArg> {
        let mut rx = self.msg_tx.subscribe();
        self.send_message(path, vec![]).await?;

        // Wait for response with timeout
        let timeout = Duration::from_secs(2);
        let start = std::time::Instant::now();

        while start.elapsed() < timeout {
            match time::timeout(timeout - start.elapsed(), rx.recv()).await {
                Ok(Ok(msg)) => {
                    if msg.path == path {
                        return msg.args.first().cloned().ok_or(
                            OscError::ParseError("Empty response".to_string()).into()
                        );
                    }
                }
                _ => break,
            }
        }
        Err(OscError::ParseError("Query timeout".to_string()).into())
    }

    /// Subscribes to all incoming OSC messages.
    pub fn subscribe(&self) -> broadcast::Receiver<OscMessage> {
        self.msg_tx.subscribe()
    }
}
