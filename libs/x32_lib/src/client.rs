use crate::error::Result;
use crate::transport::MixerTransport;
use osc_lib::{OscArg, OscError, OscMessage};
use std::sync::Arc;
use tokio::sync::{broadcast, watch};
use tokio::time::{self, Duration};

/// A client for interacting with a Behringer X32/Midas M32 mixer.
pub struct MixerClient {
    transport: Arc<dyn MixerTransport>,
    msg_tx: broadcast::Sender<OscMessage>,
    heartbeat_tx: watch::Sender<bool>,
    _receiver_handle: tokio::task::JoinHandle<()>,
    _heartbeat_handle: tokio::task::JoinHandle<()>,
}

impl MixerClient {
    /// Queries the `/node` command for a given path and returns the response string.
    pub async fn query_node(&self, node_path: &str) -> Result<String> {
        let mut rx = self.msg_tx.subscribe();
        self.send_message("/node", vec![OscArg::String(node_path.to_string())]).await?;

        let timeout_dur = Duration::from_secs(2);
        let start = std::time::Instant::now();

        while start.elapsed() < timeout_dur {
            match time::timeout(timeout_dur - start.elapsed(), rx.recv()).await {
                Ok(Ok(msg)) => {
                    if msg.path == "/node" || msg.path == "node" {
                        if let Some(OscArg::String(response_str)) = msg.args.first() {
                            return Ok(response_str.clone());
                        }
                    }
                }
                _ => continue,
            }
        }
        Err(OscError::ParseError("Query /node timeout".to_string()).into())
    }

    /// Creates a new `MixerClient` instance using the provided transport.
    pub fn new(transport: Arc<dyn MixerTransport>, heartbeat: bool) -> Self {
        let (msg_tx, _) = broadcast::channel(100);
        let msg_tx_clone = msg_tx.clone();
        let transport_clone = transport.clone();

        // Background receiver task
        let _receiver_handle = tokio::spawn(async move {
            while let Ok(msg) = transport_clone.recv().await {
                let _ = msg_tx_clone.send(msg);
            }
        });

        let (heartbeat_tx, heartbeat_rx) = watch::channel(heartbeat);
        let transport_heartbeat = transport.clone();
        let _heartbeat_handle = tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(9));
            let heartbeat_msg = OscMessage::new("/xremote".to_string(), vec![]);
            loop {
                interval.tick().await;
                if *heartbeat_rx.borrow()
                    && transport_heartbeat
                        .send(heartbeat_msg.clone())
                        .await
                        .is_err()
                {
                    break;
                }
            }
        });

        Self {
            transport,
            msg_tx,
            heartbeat_tx,
            _receiver_handle,
            _heartbeat_handle,
        }
    }

    /// Connects to a mixer at the given IP address (OSC over Ethernet or tunneled AES50).
    pub async fn connect(ip: &str, heartbeat: bool) -> Result<Self> {
        let transport = Arc::new(crate::transport::udp::UdpTransport::connect(ip).await?);
        Ok(Self::new(transport, heartbeat))
    }

    /// Connects to a mixer via USB MIDI Sysex.
    pub fn connect_midi(port_name: &str, heartbeat: bool) -> Result<Self> {
        let transport = Arc::new(crate::transport::midi::MidiTransport::connect(port_name)?);
        Ok(Self::new(transport, heartbeat))
    }

    /// Probes the mixer connection by sending a /info message.
    pub async fn probe(&self) -> bool {
        let mut rx = self.msg_tx.subscribe();
        if self.send_message("/info", vec![]).await.is_err() {
            return false;
        }
        let timeout_dur = Duration::from_millis(250);
        let start = std::time::Instant::now();
        while start.elapsed() < timeout_dur {
            match time::timeout(timeout_dur - start.elapsed(), rx.recv()).await {
                Ok(Ok(msg)) => {
                    if msg.path == "/info" {
                        return true;
                    }
                }
                _ => break,
            }
        }
        false
    }

    /// Connects automatically using fallback hierarchy.
    pub async fn connect_auto(
        primary_ip: &str,
        aes50_ip: &str,
        usb_port: &str,
        heartbeat: bool,
    ) -> Result<(Self, String)> {
        // 1. Primary IP
        if !primary_ip.is_empty() {
            println!("Probing Tier 1: Ethernet OSC at {}...", primary_ip);
            if let Ok(client) = Self::connect(primary_ip, heartbeat).await {
                if client.probe().await {
                    println!("Tier 1 Ethernet connected.");
                    return Ok((client, "osc".to_string()));
                }
            }
        }

        // 2. AES50 IP
        if !aes50_ip.is_empty() {
            println!("Probing Tier 2: AES50 Tunneled OSC at {}...", aes50_ip);
            if let Ok(client) = Self::connect(aes50_ip, heartbeat).await {
                if client.probe().await {
                    println!("Tier 2 AES50 connected.");
                    return Ok((client, "aes50".to_string()));
                }
            }
        }

        // 3. USB MIDI
        println!("Probing Tier 3: USB MIDI (Port: {})...", usb_port);
        let client = Self::connect_midi(usb_port, heartbeat)?;
        if client.probe().await {
            println!("Tier 3 USB MIDI connected.");
            return Ok((client, "usb".to_string()));
        }

        Err(crate::error::X32Error::Custom(
            "All connection attempts failed".to_string(),
        ))
    }

    /// Connects to the mixer using the specified transport configuration.
    pub async fn connect_with_transport(
        primary_ip: &str,
        aes50_ip: &str,
        usb_port: &str,
        transport: &str,
        heartbeat: bool,
    ) -> Result<(Self, String)> {
        match transport {
            "auto" => Self::connect_auto(primary_ip, aes50_ip, usb_port, heartbeat).await,
            "osc" | "aes50" => {
                let target_ip = if transport == "aes50" && !aes50_ip.is_empty() {
                    aes50_ip
                } else {
                    primary_ip
                };
                let full_ip = if target_ip.contains(':') {
                    target_ip.to_string()
                } else {
                    format!("{}:10023", target_ip)
                };
                let client = Self::connect(&full_ip, heartbeat).await?;
                Ok((client, transport.to_string()))
            }
            "usb" => {
                let client = Self::connect_midi(usb_port, heartbeat)?;
                Ok((client, "usb".to_string()))
            }
            other => Err(crate::error::X32Error::Custom(format!(
                "Unknown transport type: {}",
                other
            ))),
        }
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
        self.transport.send(msg).await?;
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
                        return msg.args.first().cloned().ok_or_else(|| {
                            OscError::ParseError("Empty response".to_string()).into()
                        });
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
