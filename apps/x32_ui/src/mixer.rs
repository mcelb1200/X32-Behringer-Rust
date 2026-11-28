//! Live mixer OSC client wrapper, state cache, and control actions.

use crate::UiEvent;
use anyhow::Result;
use osc_lib::{OscArg, OscMessage};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{broadcast, Mutex, RwLock};
use tokio::task::JoinHandle;
use x32_lib::discovery::{discover_mixers, infer_model_from_info, DiscoveredMixer};
use x32_lib::meters::meter_value_at;
use x32_lib::model::resolve_target_address;
use x32_lib::safety::{smooth_fade, ConsoleSnapshot};
use x32_lib::{MixerClient, MixerModel};

const MAX_OSC_LOG_ENTRIES: usize = 120;

/// Single input channel strip state (channels 1–32).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelStrip {
    /// 1-based channel number (1..=32).
    pub index: u8,
    /// Scribble strip name.
    pub name: String,
    /// X32 color index (0..=15).
    pub color: i32,
    /// X32 icon index.
    pub icon: i32,
    /// Input source routing index (0..=64).
    pub source: i32,
    /// Normalized fader level (`0.0..=1.0`).
    pub fader: f32,
    /// Channel unmuted/on state (`true` = unmuted/on).
    pub on: bool,
    /// Normalized pan (`0.0..=1.0`, `0.5` = center).
    pub pan: f32,
    /// Whether this channel is stereo-linked with its odd/even partner.
    pub linked_stereo: bool,
    /// Latest linear meter reading (`0.0..=1.0`).
    pub meter: f32,
}

/// Simplified strip state for Aux Ins, Mix Buses, Matrices, and DCAs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleStrip {
    /// 1-based strip index.
    pub index: u8,
    /// Scribble strip name.
    pub name: String,
    /// X32 color index (0..=15).
    pub color: i32,
    /// Normalized fader level (`0.0..=1.0`).
    pub fader: f32,
    /// Unmuted/on state (`true` = unmuted/on).
    pub on: bool,
    /// Whether this strip is stereo-linked with its odd/even partner.
    #[serde(default)]
    pub linked_stereo: bool,
}

/// Complete X32 routing block state used for Routing Matrix & Virtual Soundcheck.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RoutingState {
    /// Global routing switch (`0` = REC / Live, `1` = PLAY / Virtual Soundcheck).
    pub routswitch: i32,
    /// `/config/routing/IN/{1-8,9-16,17-24,25-32}` block values.
    pub in_blocks: [i32; 4],
    /// `/config/routing/IN/AUX` value.
    pub in_aux: i32,
    /// `/config/routing/PLAY/{1-8,9-16,17-24,25-32}` block values.
    pub play_blocks: [i32; 4],
    /// `/config/routing/PLAY/AUX` value.
    pub play_aux: i32,
    /// `/config/routing/CARD/{1-8,9-16,17-24,25-32}` block values.
    pub card_blocks: [i32; 4],
    /// `/config/routing/OUT/{1-4,5-8,9-12,13-16}` block values.
    pub out_blocks: [i32; 4],
    /// `/config/routing/AES50A/{1-8..41-48}` block values.
    pub aes50a_blocks: [i32; 6],
    /// `/config/routing/AES50B/{1-8..41-48}` block values.
    pub aes50b_blocks: [i32; 6],
}

impl Default for RoutingState {
    fn default() -> Self {
        Self {
            routswitch: 0,
            in_blocks: [0, 1, 2, 3],       // AN1-8..AN25-32
            in_aux: 0,                     // AUX1-4
            play_blocks: [16, 17, 18, 19], // CARD1-8..CARD25-32
            play_aux: 12,                  // CARD1-6
            card_blocks: [0, 1, 2, 3],     // AN1-8..AN25-32
            out_blocks: [0, 1, 2, 3],
            aes50a_blocks: [0, 1, 2, 3, 20, 21],
            aes50b_blocks: [0, 1, 2, 3, 20, 21],
        }
    }
}

/// Logged OSC packet entry for the live OSC inspector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscLogEntry {
    /// Timestamp in milliseconds since Unix epoch.
    pub timestamp_ms: u64,
    /// Direction (`"in"` or `"out"`).
    pub direction: String,
    /// OSC address path.
    pub path: String,
    /// Human-readable formatted arguments.
    pub args: String,
}

/// Full cached snapshot of the connected mixer state sent to Web UI clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixerSnapshotState {
    /// Whether a client is currently attached.
    pub connected: bool,
    /// Connected target IP/address.
    pub ip: String,
    /// Active transport (`"osc"`, `"aes50"`, `"usb"`, or `"none"`).
    pub transport: String,
    /// Detected console model string.
    pub model: String,
    /// Reported firmware/console name from `/info`.
    pub firmware: String,
    /// Console sample rate in Hz (`48000` or `44100`).
    pub sample_rate_hz: u32,
    /// Connection health (`"Connected"`, `"Reconnecting"`, `"Disconnected"`).
    pub connection_state: String,
    /// Input channels 1–32.
    pub channels: Vec<ChannelStrip>,
    /// Aux inputs 1–8.
    pub auxins: Vec<SimpleStrip>,
    /// Mix buses 1–16.
    pub buses: Vec<SimpleStrip>,
    /// Matrix outputs 1–6.
    pub matrices: Vec<SimpleStrip>,
    /// DCA groups 1–8.
    pub dcas: Vec<SimpleStrip>,
    /// Main LR fader (`0.0..=1.0`).
    pub main_fader: f32,
    /// Main LR unmuted/on state.
    pub main_on: bool,
    /// Routing matrix state.
    pub routing: RoutingState,
    /// Recent OSC messages ring buffer.
    pub recent_osc: Vec<OscLogEntry>,
}

impl Default for MixerSnapshotState {
    fn default() -> Self {
        let channels = (1..=32)
            .map(|i| ChannelStrip {
                index: i,
                name: format!("Ch {:02}", i),
                color: ((i as i32 - 1) % 7) + 1,
                icon: i as i32,
                source: i as i32,
                fader: 0.0,
                on: true,
                pan: 0.5,
                linked_stereo: false,
                meter: 0.0,
            })
            .collect();
        let auxins = (1..=8)
            .map(|i| SimpleStrip {
                index: i,
                name: format!("Aux {:02}", i),
                color: 3,
                fader: 0.0,
                on: true,
                linked_stereo: false,
            })
            .collect();
        let buses = (1..=16)
            .map(|i| SimpleStrip {
                index: i,
                name: format!("Bus {:02}", i),
                color: 6,
                fader: 0.0,
                on: true,
                linked_stereo: false,
            })
            .collect();
        let matrices = (1..=6)
            .map(|i| SimpleStrip {
                index: i,
                name: format!("Mtx {:02}", i),
                color: 5,
                fader: 0.0,
                on: true,
                linked_stereo: false,
            })
            .collect();
        let dcas = (1..=8)
            .map(|i| SimpleStrip {
                index: i,
                name: format!("DCA {}", i),
                color: 7,
                fader: 0.75,
                on: true,
                linked_stereo: false,
            })
            .collect();
        Self {
            connected: false,
            ip: String::new(),
            transport: "none".to_string(),
            model: "X32".to_string(),
            firmware: String::new(),
            sample_rate_hz: 48_000,
            connection_state: "Disconnected".to_string(),
            channels,
            auxins,
            buses,
            matrices,
            dcas,
            main_fader: 0.75,
            main_on: true,
            routing: RoutingState::default(),
            recent_osc: Vec::new(),
        }
    }
}

/// Serializable summary of a LAN-discovered mixer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredMixerDto {
    /// IP and port (`ip:port`).
    pub addr: String,
    /// Inferred model name.
    pub model: String,
    /// Console name.
    pub name: String,
    /// Raw `/info` string.
    pub raw_info: String,
}

impl From<DiscoveredMixer> for DiscoveredMixerDto {
    fn from(m: DiscoveredMixer) -> Self {
        Self {
            addr: m.addr.to_string(),
            model: format!("{:?}", m.model),
            name: m.name,
            raw_info: m.raw_info,
        }
    }
}

struct ActiveConnection {
    client: Arc<MixerClient>,
    listener_task: JoinHandle<()>,
    meter_task: JoinHandle<()>,
}

/// Manages the live connection to an X32/M32/Wing/X-Air mixer and caches its state.
pub struct MixerManager {
    state: Arc<RwLock<MixerSnapshotState>>,
    osc_log: Arc<Mutex<VecDeque<OscLogEntry>>>,
    connection: Mutex<Option<ActiveConnection>>,
    events_tx: broadcast::Sender<UiEvent>,
    raw_osc_tx: broadcast::Sender<OscMessage>,
}

/// Discovers mixers on the LAN and returns the first responding address, if any.
pub async fn discover_first_ip() -> Result<Option<String>> {
    let found = discover_mixers(Duration::from_millis(900)).await?;
    Ok(found.into_iter().next().map(|m| m.addr.to_string()))
}

impl MixerManager {
    /// Creates a new [`MixerManager`] publishing updates onto `events_tx`.
    pub fn new(events_tx: broadcast::Sender<UiEvent>) -> Self {
        let (raw_osc_tx, _) = broadcast::channel(512);
        Self {
            state: Arc::new(RwLock::new(MixerSnapshotState::default())),
            osc_log: Arc::new(Mutex::new(VecDeque::with_capacity(MAX_OSC_LOG_ENTRIES))),
            connection: Mutex::new(None),
            events_tx,
            raw_osc_tx,
        }
    }

    /// Subscribes to all incoming non-meter [`OscMessage`] packets from the connected mixer.
    pub fn subscribe_osc(&self) -> broadcast::Receiver<OscMessage> {
        self.raw_osc_tx.subscribe()
    }

    /// Broadcasts LAN discovery packets and returns all responding consoles.
    pub async fn discover(&self) -> Result<Vec<DiscoveredMixerDto>> {
        let found = discover_mixers(Duration::from_millis(1200)).await?;
        Ok(found.into_iter().map(DiscoveredMixerDto::from).collect())
    }

    /// Returns a cloned snapshot of the current mixer state.
    pub async fn snapshot(&self) -> MixerSnapshotState {
        let mut s = self.state.read().await.clone();
        s.recent_osc = self.osc_log.lock().await.iter().cloned().collect();
        s
    }

    /// Connects to a mixer and starts background OSC/meter synchronization.
    pub async fn connect(
        &self,
        ip: &str,
        transport: &str,
        aes50_ip: &str,
        usb_port: &str,
    ) -> Result<MixerSnapshotState> {
        self.disconnect().await;

        let resolved = if transport == "usb" {
            ip.to_string()
        } else {
            resolve_target_address(ip.trim(), MixerModel::X32)
        };

        let (client, active_transport) = match transport {
            "auto" => {
                MixerClient::connect_with_transport(&resolved, aes50_ip, usb_port, "auto", true)
                    .await?
            }
            "osc" | "udp" => {
                let c = MixerClient::connect(&resolved, true).await?;
                (c, "osc".to_string())
            }
            "aes50" => {
                let target = if aes50_ip.trim().is_empty() {
                    resolved.as_str()
                } else {
                    aes50_ip.trim()
                };
                let full = resolve_target_address(target, MixerModel::X32);
                let c = MixerClient::connect(&full, true).await?;
                (c, "aes50".to_string())
            }
            "usb" => {
                let c = MixerClient::connect_midi(usb_port, true)?;
                (c, "usb".to_string())
            }
            other => anyhow::bail!("Unsupported transport: {}", other),
        };

        let client = Arc::new(client);

        {
            let mut st = self.state.write().await;
            st.connected = true;
            st.ip = resolved.clone();
            st.transport = active_transport.clone();
            st.connection_state = "Connected".to_string();
        }

        let state_ref = self.state.clone();
        let osc_log_ref = self.osc_log.clone();
        let events_ref = self.events_tx.clone();
        let raw_osc_ref = self.raw_osc_tx.clone();
        let mut rx = client.subscribe();
        let mut conn_rx = client.connection_state();

        let listener_task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    recv = rx.recv() => {
                        match recv {
                            Ok(msg) => {
                                let is_meter = msg.path.starts_with("/meters");
                                apply_osc_to_state(&state_ref, &msg).await;
                                if !is_meter {
                                    let _ = raw_osc_ref.send(msg.clone());
                                    let entry = OscLogEntry {
                                        timestamp_ms: now_ms(),
                                        direction: "in".to_string(),
                                        path: msg.path.clone(),
                                        args: format_args(&msg.args),
                                    };
                                    push_osc_log(&osc_log_ref, entry.clone()).await;
                                    let _ = events_ref.send(UiEvent::Osc(entry));
                                }
                            }
                            Err(broadcast::error::RecvError::Lagged(_)) => continue,
                            Err(broadcast::error::RecvError::Closed) => break,
                        }
                    }
                    changed = conn_rx.changed() => {
                        if changed.is_err() {
                            break;
                        }
                        let health = format!("{:?}", *conn_rx.borrow());
                        state_ref.write().await.connection_state = health;
                    }
                }
            }
        });

        let meter_client = client.clone();
        let meter_state = self.state.clone();
        let meter_events = self.events_tx.clone();
        let meter_task = tokio::spawn(async move {
            let mut tick = tokio::time::interval(Duration::from_secs(4));
            let mut state_push = tokio::time::interval(Duration::from_millis(450));
            loop {
                tokio::select! {
                    _ = tick.tick() => {
                        let _ = meter_client
                            .send_message(
                                "/meters",
                                vec![OscArg::String("/meters/0".to_string()), OscArg::Int(2)],
                            )
                            .await;
                    }
                    _ = state_push.tick() => {
                        let snap = meter_state.read().await.clone();
                        let _ = meter_events.send(UiEvent::State(snap));
                    }
                }
            }
        });

        {
            let mut guard = self.connection.lock().await;
            *guard = Some(ActiveConnection {
                client,
                listener_task,
                meter_task,
            });
        }

        // Pull initial console state without failing connection if an emulator/device omits some paths.
        let _ = self.refresh_full_state().await;
        let snap = self.snapshot().await;
        let _ = self.events_tx.send(UiEvent::State(snap.clone()));
        Ok(snap)
    }

    /// Disconnects any active mixer client and stops background listener tasks.
    pub async fn disconnect(&self) {
        let prev = self.connection.lock().await.take();
        if let Some(conn) = prev {
            conn.listener_task.abort();
            conn.meter_task.abort();
        }
        let mut st = self.state.write().await;
        st.connected = false;
        st.transport = "none".to_string();
        st.connection_state = "Disconnected".to_string();
        let snap = st.clone();
        drop(st);
        let _ = self.events_tx.send(UiEvent::State(snap));
    }

    /// Returns the currently connected [`MixerClient`], or an error if disconnected.
    pub async fn require_client(&self) -> Result<Arc<MixerClient>> {
        let guard = self.connection.lock().await;
        guard
            .as_ref()
            .map(|c| c.client.clone())
            .ok_or_else(|| anyhow::anyhow!("Not connected to any mixer"))
    }

    /// Queries the console for all key mixer, channel, routing, and clock settings.
    pub async fn refresh_full_state(&self) -> Result<MixerSnapshotState> {
        let client = self.require_client().await?;

        let mut queries: Vec<&'static str> = vec![
            "/info",
            "/-prefs/clockrate",
            "/main/st/mix/fader",
            "/main/st/mix/on",
            "/config/routing/routswitch",
            "/config/routing/IN/1-8",
            "/config/routing/IN/9-16",
            "/config/routing/IN/17-24",
            "/config/routing/IN/25-32",
            "/config/routing/IN/AUX",
            "/config/routing/PLAY/1-8",
            "/config/routing/PLAY/9-16",
            "/config/routing/PLAY/17-24",
            "/config/routing/PLAY/25-32",
            "/config/routing/PLAY/AUX",
            "/config/routing/CARD/1-8",
            "/config/routing/CARD/9-16",
            "/config/routing/CARD/17-24",
            "/config/routing/CARD/25-32",
            "/config/routing/OUT/1-4",
            "/config/routing/OUT/5-8",
            "/config/routing/OUT/9-12",
            "/config/routing/OUT/13-16",
        ];
        for q in queries.drain(..) {
            let _ = client.send_message(q, vec![]).await;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;

        for pair in (1..=31).step_by(2) {
            let p = format!("/config/chlink/{}-{}", pair, pair + 1);
            let _ = client.send_message(&p, vec![]).await;
        }
        for pair in (1..=7).step_by(2) {
            let _ = client
                .send_message(&format!("/config/auxlink/{}-{}", pair, pair + 1), vec![])
                .await;
        }
        for pair in (1..=15).step_by(2) {
            let _ = client
                .send_message(&format!("/config/buslink/{}-{}", pair, pair + 1), vec![])
                .await;
        }
        for pair in (1..=5).step_by(2) {
            let _ = client
                .send_message(&format!("/config/mtxlink/{}-{}", pair, pair + 1), vec![])
                .await;
        }
        for block in ["1-8", "9-16", "17-24", "25-32", "33-40", "41-48"] {
            let _ = client
                .send_message(&format!("/config/routing/AES50A/{}", block), vec![])
                .await;
            let _ = client
                .send_message(&format!("/config/routing/AES50B/{}", block), vec![])
                .await;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;

        for ch in 1..=32u8 {
            let _ = client
                .send_message(&format!("/ch/{:02}/config/name", ch), vec![])
                .await;
            let _ = client
                .send_message(&format!("/ch/{:02}/config/color", ch), vec![])
                .await;
            let _ = client
                .send_message(&format!("/ch/{:02}/config/icon", ch), vec![])
                .await;
            let _ = client
                .send_message(&format!("/ch/{:02}/config/source", ch), vec![])
                .await;
            let _ = client
                .send_message(&format!("/ch/{:02}/mix/fader", ch), vec![])
                .await;
            let _ = client
                .send_message(&format!("/ch/{:02}/mix/on", ch), vec![])
                .await;
            let _ = client
                .send_message(&format!("/ch/{:02}/mix/pan", ch), vec![])
                .await;
            if ch % 8 == 0 {
                tokio::time::sleep(Duration::from_millis(12)).await;
            }
        }

        for dca in 1..=8u8 {
            let _ = client
                .send_message(&format!("/dca/{}/config/name", dca), vec![])
                .await;
            let _ = client
                .send_message(&format!("/dca/{}/config/color", dca), vec![])
                .await;
            let _ = client
                .send_message(&format!("/dca/{}/fader", dca), vec![])
                .await;
            let _ = client
                .send_message(&format!("/dca/{}/on", dca), vec![])
                .await;
        }
        for bus in 1..=16u8 {
            let _ = client
                .send_message(&format!("/bus/{:02}/config/name", bus), vec![])
                .await;
            let _ = client
                .send_message(&format!("/bus/{:02}/config/color", bus), vec![])
                .await;
            let _ = client
                .send_message(&format!("/bus/{:02}/mix/fader", bus), vec![])
                .await;
            let _ = client
                .send_message(&format!("/bus/{:02}/mix/on", bus), vec![])
                .await;
        }
        for aux in 1..=8u8 {
            let _ = client
                .send_message(&format!("/auxin/{:02}/config/name", aux), vec![])
                .await;
            let _ = client
                .send_message(&format!("/auxin/{:02}/config/color", aux), vec![])
                .await;
            let _ = client
                .send_message(&format!("/auxin/{:02}/mix/fader", aux), vec![])
                .await;
            let _ = client
                .send_message(&format!("/auxin/{:02}/mix/on", aux), vec![])
                .await;
        }
        for mtx in 1..=6u8 {
            let _ = client
                .send_message(&format!("/mtx/{:02}/config/name", mtx), vec![])
                .await;
            let _ = client
                .send_message(&format!("/mtx/{:02}/config/color", mtx), vec![])
                .await;
            let _ = client
                .send_message(&format!("/mtx/{:02}/mix/fader", mtx), vec![])
                .await;
            let _ = client
                .send_message(&format!("/mtx/{:02}/mix/on", mtx), vec![])
                .await;
        }

        // Wait briefly for UDP replies to land in the background listener.
        tokio::time::sleep(Duration::from_millis(180)).await;
        let snap = self.snapshot().await;
        let _ = self.events_tx.send(UiEvent::State(snap.clone()));
        Ok(snap)
    }

    /// Sends an OSC message to the mixer, logs it, and updates local cached state optimistically.
    pub async fn send_osc(&self, path: &str, args: Vec<OscArg>) -> Result<()> {
        let client = self.require_client().await?;
        client.send_message(path, args.clone()).await?;
        let msg = OscMessage::new(path.to_string(), args);
        apply_osc_to_state(&self.state, &msg).await;

        let entry = OscLogEntry {
            timestamp_ms: now_ms(),
            direction: "out".to_string(),
            path: msg.path.clone(),
            args: format_args(&msg.args),
        };
        push_osc_log(&self.osc_log, entry.clone()).await;
        let _ = self.events_tx.send(UiEvent::Osc(entry));
        Ok(())
    }

    /// Sets a channel, aux, bus, matrix, DCA, or main fader (`kind` = `"ch" | "auxin" | "bus" | "mtx" | "dca" | "main"`).
    pub async fn set_fader(&self, kind: &str, index: u8, value: f32) -> Result<()> {
        let clamped = value.clamp(0.0, 1.0);
        let path = match kind {
            "ch" => format!("/ch/{:02}/mix/fader", index),
            "auxin" => format!("/auxin/{:02}/mix/fader", index),
            "bus" => format!("/bus/{:02}/mix/fader", index),
            "mtx" => format!("/mtx/{:02}/mix/fader", index),
            "dca" => format!("/dca/{}/fader", index),
            "main" => "/main/st/mix/fader".to_string(),
            other => anyhow::bail!("Unknown strip kind: {}", other),
        };
        self.send_osc(&path, vec![OscArg::Float(clamped)]).await
    }

    /// Sets mute/on state (`on = true` means unmuted / audio passes).
    pub async fn set_mute(&self, kind: &str, index: u8, on: bool) -> Result<()> {
        let path = match kind {
            "ch" => format!("/ch/{:02}/mix/on", index),
            "auxin" => format!("/auxin/{:02}/mix/on", index),
            "bus" => format!("/bus/{:02}/mix/on", index),
            "mtx" => format!("/mtx/{:02}/mix/on", index),
            "dca" => format!("/dca/{}/on", index),
            "main" => "/main/st/mix/on".to_string(),
            other => anyhow::bail!("Unknown strip kind: {}", other),
        };
        self.send_osc(&path, vec![OscArg::Int(if on { 1 } else { 0 })])
            .await
    }

    /// Sets channel pan (`0.0..=1.0`).
    pub async fn set_pan(&self, index: u8, pan: f32) -> Result<()> {
        let path = format!("/ch/{:02}/mix/pan", index);
        self.send_osc(&path, vec![OscArg::Float(pan.clamp(0.0, 1.0))])
            .await
    }

    /// Updates a strip's scribble strip name or color (`kind` = `"ch" | "auxin" | "bus" | "mtx" | "dca"`).
    pub async fn set_strip_config(
        &self,
        kind: &str,
        index: u8,
        name: Option<&str>,
        color: Option<i32>,
    ) -> Result<()> {
        let prefix = match kind {
            "ch" => format!("/ch/{:02}", index),
            "auxin" => format!("/auxin/{:02}", index),
            "bus" => format!("/bus/{:02}", index),
            "mtx" => format!("/mtx/{:02}", index),
            "dca" => format!("/dca/{}", index),
            other => anyhow::bail!("Unknown strip kind: {}", other),
        };
        if let Some(n) = name {
            self.send_osc(
                &format!("{}/config/name", prefix),
                vec![OscArg::String(n.to_string())],
            )
            .await?;
        }
        if let Some(c) = color {
            self.send_osc(
                &format!("{}/config/color", prefix),
                vec![OscArg::Int(c.clamp(0, 15))],
            )
            .await?;
        }
        Ok(())
    }

    /// Updates channel scribble strip name or color.
    pub async fn set_channel_config(
        &self,
        index: u8,
        name: Option<&str>,
        color: Option<i32>,
    ) -> Result<()> {
        self.set_strip_config("ch", index, name, color).await
    }

    /// Updates a routing block or switch under `/config/routing/*`.
    pub async fn set_routing(&self, path: &str, value: i32) -> Result<()> {
        if !path.starts_with("/config/routing/") {
            anyhow::bail!("Invalid routing OSC path: {}", path);
        }
        self.send_osc(path, vec![OscArg::Int(value)]).await?;
        let snap = self.snapshot().await;
        let _ = self.events_tx.send(UiEvent::State(snap));
        Ok(())
    }

    /// Smoothly fades Main LR, Buses, and Matrices down to zero as a safe panic mute.
    pub async fn panic_mute(&self) -> Result<()> {
        let client = self.require_client().await?;
        let snap = self.state.read().await.clone();
        let mut targets = vec![("/main/st/mix/fader".to_string(), snap.main_fader)];
        for bus in &snap.buses {
            targets.push((format!("/bus/{:02}/mix/fader", bus.index), bus.fader));
        }
        for mtx in &snap.matrices {
            targets.push((format!("/mtx/{:02}/mix/fader", mtx.index), mtx.fader));
        }
        smooth_fade(&client, &targets, 10, Duration::from_millis(400)).await?;
        let _ = self.send_osc("/main/st/mix/on", vec![OscArg::Int(0)]).await;
        let updated = self.snapshot().await;
        let _ = self.events_tx.send(UiEvent::State(updated));
        Ok(())
    }

    /// Captures current channels, main LR, DCAs, buses, and routing into a [`ConsoleSnapshot`] file.
    pub async fn save_snapshot_backup(&self, path: &Path) -> Result<usize> {
        let st = self.state.read().await.clone();
        let snap = build_full_snapshot(&st);
        snap.save_to_file(path)?;
        Ok(snap.entries.len())
    }

    /// Restores a previously saved [`ConsoleSnapshot`] file to the connected mixer.
    pub async fn restore_snapshot_backup(&self, path: &Path) -> Result<usize> {
        let client = self.require_client().await?;
        let snap = ConsoleSnapshot::load_from_file(path)?;
        let count = snap.entries.len();
        snap.restore(&client).await?;
        for (p, val) in &snap.entries {
            let msg = OscMessage::new(p.clone(), vec![OscArg::from(val)]);
            apply_osc_to_state(&self.state, &msg).await;
        }
        let updated = self.snapshot().await;
        let _ = self.events_tx.send(UiEvent::State(updated));
        Ok(count)
    }
}

/// Builds a [`ConsoleSnapshot`] containing channels, buses, DCAs, main LR, and routing state.
pub fn build_full_snapshot(st: &MixerSnapshotState) -> ConsoleSnapshot {
    let mut snap = ConsoleSnapshot::new();
    for ch in &st.channels {
        snap.record_string(format!("/ch/{:02}/config/name", ch.index), &ch.name);
        snap.record_int(format!("/ch/{:02}/config/color", ch.index), ch.color);
        snap.record_float(format!("/ch/{:02}/mix/fader", ch.index), ch.fader);
        snap.record_int(format!("/ch/{:02}/mix/on", ch.index), i32::from(ch.on));
        snap.record_float(format!("/ch/{:02}/mix/pan", ch.index), ch.pan);
    }
    for bus in &st.buses {
        snap.record_float(format!("/bus/{:02}/mix/fader", bus.index), bus.fader);
        snap.record_int(format!("/bus/{:02}/mix/on", bus.index), i32::from(bus.on));
    }
    for mtx in &st.matrices {
        snap.record_string(format!("/mtx/{:02}/config/name", mtx.index), &mtx.name);
        snap.record_int(format!("/mtx/{:02}/config/color", mtx.index), mtx.color);
        snap.record_float(format!("/mtx/{:02}/mix/fader", mtx.index), mtx.fader);
        snap.record_int(format!("/mtx/{:02}/mix/on", mtx.index), i32::from(mtx.on));
    }
    for dca in &st.dcas {
        snap.record_float(format!("/dca/{}/fader", dca.index), dca.fader);
        snap.record_int(format!("/dca/{}/on", dca.index), i32::from(dca.on));
    }
    snap.record_float("/main/st/mix/fader", st.main_fader);
    snap.record_int("/main/st/mix/on", i32::from(st.main_on));
    record_routing_entries(&mut snap, &st.routing);
    snap
}

/// Records all routing paths from `routing` into `snap`.
pub fn record_routing_entries(snap: &mut ConsoleSnapshot, routing: &RoutingState) {
    snap.record_int("/config/routing/routswitch", routing.routswitch);
    for (idx, label) in ["1-8", "9-16", "17-24", "25-32"].iter().enumerate() {
        snap.record_int(format!("/config/routing/IN/{}", label), routing.in_blocks[idx]);
        snap.record_int(
            format!("/config/routing/PLAY/{}", label),
            routing.play_blocks[idx],
        );
        snap.record_int(
            format!("/config/routing/CARD/{}", label),
            routing.card_blocks[idx],
        );
    }
    for (idx, label) in ["1-4", "5-8", "9-12", "13-16"].iter().enumerate() {
        snap.record_int(
            format!("/config/routing/OUT/{}", label),
            routing.out_blocks[idx],
        );
    }
    snap.record_int("/config/routing/IN/AUX", routing.in_aux);
    snap.record_int("/config/routing/PLAY/AUX", routing.play_aux);
}

async fn apply_osc_to_state(state_lock: &Arc<RwLock<MixerSnapshotState>>, msg: &OscMessage) {
    let mut st = state_lock.write().await;
    let p = msg.path.as_str();

    if p == "/info" {
        let parts: Vec<String> = msg
            .args
            .iter()
            .filter_map(|a| match a {
                OscArg::String(s) => Some(s.clone()),
                _ => None,
            })
            .collect();
        let joined = parts.join(" ");
        st.model = format!("{:?}", infer_model_from_info(&joined));
        if let Some(fw) = parts.last() {
            st.firmware = fw.clone();
        }
        return;
    }

    if p == "/-prefs/clockrate" {
        if let Some(OscArg::Int(v)) = msg.args.first() {
            st.sample_rate_hz = if *v == 1 { 44_100 } else { 48_000 };
        }
        return;
    }

    if p == "/main/st/mix/fader" {
        if let Some(OscArg::Float(f)) = msg.args.first() {
            st.main_fader = *f;
        }
        return;
    }
    if p == "/main/st/mix/on" {
        if let Some(OscArg::Int(i)) = msg.args.first() {
            st.main_on = *i != 0;
        }
        return;
    }

    if p == "/meters/0" {
        if let Some(OscArg::Blob(blob)) = msg.args.first() {
            for (idx, ch) in st.channels.iter_mut().enumerate() {
                if let Some(val) = meter_value_at(blob, idx) {
                    ch.meter = val.clamp(0.0, 1.0);
                }
            }
        }
        return;
    }

    if let Some(rest) = p.strip_prefix("/config/routing/") {
        if let Some(OscArg::Int(v)) = msg.args.first() {
            match rest {
                "routswitch" => st.routing.routswitch = *v,
                "IN/1-8" => st.routing.in_blocks[0] = *v,
                "IN/9-16" => st.routing.in_blocks[1] = *v,
                "IN/17-24" => st.routing.in_blocks[2] = *v,
                "IN/25-32" => st.routing.in_blocks[3] = *v,
                "IN/AUX" => st.routing.in_aux = *v,
                "PLAY/1-8" => st.routing.play_blocks[0] = *v,
                "PLAY/9-16" => st.routing.play_blocks[1] = *v,
                "PLAY/17-24" => st.routing.play_blocks[2] = *v,
                "PLAY/25-32" => st.routing.play_blocks[3] = *v,
                "PLAY/AUX" => st.routing.play_aux = *v,
                "CARD/1-8" => st.routing.card_blocks[0] = *v,
                "CARD/9-16" => st.routing.card_blocks[1] = *v,
                "CARD/17-24" => st.routing.card_blocks[2] = *v,
                "CARD/25-32" => st.routing.card_blocks[3] = *v,
                "OUT/1-4" => st.routing.out_blocks[0] = *v,
                "OUT/5-8" => st.routing.out_blocks[1] = *v,
                "OUT/9-12" => st.routing.out_blocks[2] = *v,
                "OUT/13-16" => st.routing.out_blocks[3] = *v,
                "AES50A/1-8" => st.routing.aes50a_blocks[0] = *v,
                "AES50A/9-16" => st.routing.aes50a_blocks[1] = *v,
                "AES50A/17-24" => st.routing.aes50a_blocks[2] = *v,
                "AES50A/25-32" => st.routing.aes50a_blocks[3] = *v,
                "AES50A/33-40" => st.routing.aes50a_blocks[4] = *v,
                "AES50A/41-48" => st.routing.aes50a_blocks[5] = *v,
                "AES50B/1-8" => st.routing.aes50b_blocks[0] = *v,
                "AES50B/9-16" => st.routing.aes50b_blocks[1] = *v,
                "AES50B/17-24" => st.routing.aes50b_blocks[2] = *v,
                "AES50B/25-32" => st.routing.aes50b_blocks[3] = *v,
                "AES50B/33-40" => st.routing.aes50b_blocks[4] = *v,
                "AES50B/41-48" => st.routing.aes50b_blocks[5] = *v,
                _ => {}
            }
        }
        return;
    }

    if let Some(pair) = p.strip_prefix("/config/chlink/") {
        apply_link_pair(&mut st.channels, pair, msg.args.first());
        return;
    }
    if let Some(pair) = p.strip_prefix("/config/auxlink/") {
        apply_simple_link_pair(&mut st.auxins, pair, msg.args.first());
        return;
    }
    if let Some(pair) = p.strip_prefix("/config/buslink/") {
        apply_simple_link_pair(&mut st.buses, pair, msg.args.first());
        return;
    }
    if let Some(pair) = p.strip_prefix("/config/mtxlink/") {
        apply_simple_link_pair(&mut st.matrices, pair, msg.args.first());
        return;
    }

    if let Some(rest) = p.strip_prefix("/ch/") {
        if let Some((num_s, sub)) = rest.split_once('/') {
            if let Ok(num) = num_s.parse::<usize>() {
                if let Some(ch) = st.channels.get_mut(num.saturating_sub(1)) {
                    match sub {
                        "config/name" => {
                            if let Some(OscArg::String(s)) = msg.args.first() {
                                ch.name = s.clone();
                            }
                        }
                        "config/color" => {
                            if let Some(OscArg::Int(i)) = msg.args.first() {
                                ch.color = *i;
                            }
                        }
                        "config/icon" => {
                            if let Some(OscArg::Int(i)) = msg.args.first() {
                                ch.icon = *i;
                            }
                        }
                        "config/source" => {
                            if let Some(OscArg::Int(i)) = msg.args.first() {
                                ch.source = *i;
                            }
                        }
                        "mix/fader" => {
                            if let Some(OscArg::Float(f)) = msg.args.first() {
                                ch.fader = *f;
                            }
                        }
                        "mix/on" => {
                            if let Some(OscArg::Int(i)) = msg.args.first() {
                                ch.on = *i != 0;
                            }
                        }
                        "mix/pan" => {
                            if let Some(OscArg::Float(f)) = msg.args.first() {
                                ch.pan = *f;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        return;
    }

    if let Some(rest) = p.strip_prefix("/dca/") {
        if let Some((num_s, sub)) = rest.split_once('/') {
            if let Ok(num) = num_s.parse::<usize>() {
                if let Some(dca) = st.dcas.get_mut(num.saturating_sub(1)) {
                    match sub {
                        "config/name" => {
                            if let Some(OscArg::String(s)) = msg.args.first() {
                                dca.name = s.clone();
                            }
                        }
                        "config/color" => {
                            if let Some(OscArg::Int(i)) = msg.args.first() {
                                dca.color = *i;
                            }
                        }
                        "fader" => {
                            if let Some(OscArg::Float(f)) = msg.args.first() {
                                dca.fader = *f;
                            }
                        }
                        "on" => {
                            if let Some(OscArg::Int(i)) = msg.args.first() {
                                dca.on = *i != 0;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        return;
    }

    if let Some(rest) = p.strip_prefix("/bus/") {
        if let Some((num_s, sub)) = rest.split_once('/') {
            if let Ok(num) = num_s.parse::<usize>() {
                if let Some(bus) = st.buses.get_mut(num.saturating_sub(1)) {
                    match sub {
                        "config/name" => {
                            if let Some(OscArg::String(s)) = msg.args.first() {
                                bus.name = s.clone();
                            }
                        }
                        "config/color" => {
                            if let Some(OscArg::Int(i)) = msg.args.first() {
                                bus.color = *i;
                            }
                        }
                        "mix/fader" => {
                            if let Some(OscArg::Float(f)) = msg.args.first() {
                                bus.fader = *f;
                            }
                        }
                        "mix/on" => {
                            if let Some(OscArg::Int(i)) = msg.args.first() {
                                bus.on = *i != 0;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        return;
    }

    if let Some(rest) = p.strip_prefix("/auxin/") {
        if let Some((num_s, sub)) = rest.split_once('/') {
            if let Ok(num) = num_s.parse::<usize>() {
                if let Some(aux) = st.auxins.get_mut(num.saturating_sub(1)) {
                    match sub {
                        "config/name" => {
                            if let Some(OscArg::String(s)) = msg.args.first() {
                                aux.name = s.clone();
                            }
                        }
                        "config/color" => {
                            if let Some(OscArg::Int(i)) = msg.args.first() {
                                aux.color = *i;
                            }
                        }
                        "mix/fader" => {
                            if let Some(OscArg::Float(f)) = msg.args.first() {
                                aux.fader = *f;
                            }
                        }
                        "mix/on" => {
                            if let Some(OscArg::Int(i)) = msg.args.first() {
                                aux.on = *i != 0;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        return;
    }

    if let Some(rest) = p.strip_prefix("/mtx/") {
        if let Some((num_s, sub)) = rest.split_once('/') {
            if let Ok(num) = num_s.parse::<usize>() {
                if let Some(mtx) = st.matrices.get_mut(num.saturating_sub(1)) {
                    match sub {
                        "config/name" => {
                            if let Some(OscArg::String(s)) = msg.args.first() {
                                mtx.name = s.clone();
                            }
                        }
                        "config/color" => {
                            if let Some(OscArg::Int(i)) = msg.args.first() {
                                mtx.color = *i;
                            }
                        }
                        "mix/fader" => {
                            if let Some(OscArg::Float(f)) = msg.args.first() {
                                mtx.fader = *f;
                            }
                        }
                        "mix/on" => {
                            if let Some(OscArg::Int(i)) = msg.args.first() {
                                mtx.on = *i != 0;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

fn apply_link_pair(channels: &mut [ChannelStrip], pair: &str, arg: Option<&OscArg>) {
    if let (Some((a, b)), Some(OscArg::Int(v))) = (pair.split_once('-'), arg) {
        if let (Ok(i1), Ok(i2)) = (a.parse::<usize>(), b.parse::<usize>()) {
            let linked = *v != 0;
            if let Some(c) = channels.get_mut(i1.saturating_sub(1)) {
                c.linked_stereo = linked;
            }
            if let Some(c) = channels.get_mut(i2.saturating_sub(1)) {
                c.linked_stereo = linked;
            }
        }
    }
}

fn apply_simple_link_pair(strips: &mut [SimpleStrip], pair: &str, arg: Option<&OscArg>) {
    if let (Some((a, b)), Some(OscArg::Int(v))) = (pair.split_once('-'), arg) {
        if let (Ok(i1), Ok(i2)) = (a.parse::<usize>(), b.parse::<usize>()) {
            let linked = *v != 0;
            if let Some(c) = strips.get_mut(i1.saturating_sub(1)) {
                c.linked_stereo = linked;
            }
            if let Some(c) = strips.get_mut(i2.saturating_sub(1)) {
                c.linked_stereo = linked;
            }
        }
    }
}

async fn push_osc_log(log: &Arc<Mutex<VecDeque<OscLogEntry>>>, entry: OscLogEntry) {
    let mut guard = log.lock().await;
    if guard.len() >= MAX_OSC_LOG_ENTRIES {
        guard.pop_front();
    }
    guard.push_back(entry);
}

fn format_args(args: &[OscArg]) -> String {
    args.iter()
        .map(|a| match a {
            OscArg::Int(i) => i.to_string(),
            OscArg::Float(f) => format!("{:.4}", f),
            OscArg::String(s) => format!("\"{}\"", s),
            OscArg::Blob(b) => format!("<blob {}B>", b.len()),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
