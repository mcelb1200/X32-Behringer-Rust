use anyhow::Result;
use crossbeam_channel::Sender;
use osc_lib::OscMessage;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::sync::broadcast::error::RecvError;
use tokio::task;
use tokio::time::{self, Duration};
use x32_lib::client::MixerClient;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Channel(usize), // 1..32
    Bus(usize),     // 1..16
    Aux(usize),     // 1..6
    MainL,
    MainR,
}

impl std::fmt::Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Source::Channel(ch) => write!(f, "ch{}", ch),
            Source::Bus(b) => write!(f, "bus{}", b),
            Source::Aux(a) => write!(f, "aux{}", a),
            Source::MainL => write!(f, "main/l"),
            Source::MainR => write!(f, "main/r"),
        }
    }
}

impl std::str::FromStr for Source {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().to_lowercase();
        if s == "main" || s == "mainl" || s == "main/l" {
            return Ok(Source::MainL);
        }
        if s == "mainr" || s == "main/r" {
            return Ok(Source::MainR);
        }
        if s.starts_with("ch") {
            let num_str = &s[2..];
            if let Ok(val) = num_str.parse::<usize>() {
                if val >= 1 && val <= 32 {
                    return Ok(Source::Channel(val));
                }
            }
            return Err(format!("Invalid channel number: {}", s));
        }
        if s.starts_with("bus") {
            let num_str = &s[3..];
            if let Ok(val) = num_str.parse::<usize>() {
                if val >= 1 && val <= 16 {
                    return Ok(Source::Bus(val));
                }
            }
            return Err(format!("Invalid bus number: {}", s));
        }
        if s.starts_with("aux") {
            let num_str = &s[3..];
            if let Ok(val) = num_str.parse::<usize>() {
                if val >= 1 && val <= 6 {
                    return Ok(Source::Aux(val));
                }
            }
            return Err(format!("Invalid aux number: {}", s));
        }
        if let Ok(val) = s.parse::<usize>() {
            if val >= 1 && val <= 32 {
                return Ok(Source::Channel(val));
            }
        }
        Err(format!(
            "Invalid source: '{}'. Use ch1-32, bus1-16, aux1-6, or main",
            s
        ))
    }
}

pub enum NetworkEvent {
    MeterLevel(f32), // Normalized 0.0-1.0
    PanicTriggered,
    EncoderTurned(i32),          // Delta or Value
    EffectLoaded(usize, String), // Slot, Effect name
}

pub struct NetworkManager {
    client: Arc<MixerClient>,
    #[allow(dead_code)]
    ip: String,
    connected: Arc<AtomicBool>,
    event_sender: Sender<NetworkEvent>,
    #[allow(dead_code)]
    panic_subscribed: Arc<AtomicBool>,
    // We need to know which source to parse from the meter blob
    source: Source,
    panic_btn_path: String,
    preset_enc_path: String,
}

impl NetworkManager {
    pub async fn new(
        ip: &str,
        source: Source,
        event_sender: Sender<NetworkEvent>,
        panic_btn_path: &str,
        preset_enc_path: &str,
    ) -> Result<Self> {
        let client = Arc::new(MixerClient::connect(ip, true).await?);

        Ok(Self {
            client,

            ip: ip.to_string(),
            connected: Arc::new(AtomicBool::new(false)),
            event_sender,
            panic_subscribed: Arc::new(AtomicBool::new(false)),
            source,
            panic_btn_path: panic_btn_path.to_string(),
            preset_enc_path: preset_enc_path.to_string(),
        })
    }

    pub fn connect(&self) -> Result<()> {
        self.connected.store(true, Ordering::Relaxed);
        Ok(())
    }

    pub async fn send(&self, msg: &OscMessage) -> Result<()> {
        self.client
            .send_message(&msg.path, msg.args.clone())
            .await?;
        Ok(())
    }

    pub fn start_polling(&self, _target_slot: usize) {
        let client = self.client.clone();
        let sender = self.event_sender.clone();
        let is_connected = self.connected.clone();
        let source = self.source;
        let panic_path = self.panic_btn_path.clone();
        let enc_path = self.preset_enc_path.clone();

        task::spawn(async move {
            let mut rx = client.subscribe();

            let mut meter_interval = time::interval(Duration::from_millis(50));
            let mut fx_interval = time::interval(Duration::from_secs(2));

            let meter_path = match source {
                Source::Channel(_) => "/meters/1",
                Source::Bus(_) | Source::MainL | Source::MainR => "/meters/2",
                Source::Aux(_) => "/meters/0",
            };

            loop {
                if !is_connected.load(Ordering::Relaxed) {
                    time::sleep(Duration::from_millis(100)).await;
                    continue;
                }

                tokio::select! {
                    _ = meter_interval.tick() => {
                        let _ = client.send_message("/meters", vec![osc_lib::OscArg::String(meter_path.to_string())]).await;
                    }
                    _ = fx_interval.tick() => {
                        for slot in 1..=8 {
                            let fx_path = format!("/fx/{}/type", slot);
                            let _ = client.send_message(&fx_path, vec![]).await;
                            time::sleep(Duration::from_millis(10)).await;
                        }
                    }
                    msg_res = rx.recv() => {
                        match msg_res {
                            Ok(msg) => {
                                Self::handle_message(msg, &sender, source, &panic_path, &enc_path);
                            }
                            Err(RecvError::Lagged(_)) => continue,
                            Err(RecvError::Closed) => break,
                        }
                    }
                }
            }
        });
    }

    fn handle_message(
        msg: OscMessage,
        sender: &Sender<NetworkEvent>,
        source: Source,
        panic_path: &str,
        enc_path: &str,
    ) {
        let (expected_path, offset_idx) = match source {
            Source::Channel(ch) => ("/meters/1", ch - 1),
            Source::Bus(b) => ("/meters/2", b - 1),
            Source::MainL => ("/meters/2", 22),
            Source::MainR => ("/meters/2", 23),
            Source::Aux(a) => ("/meters/0", 32 + (a - 1)),
        };

        if msg.path == expected_path {
            if let Some(osc_lib::OscArg::Blob(data)) = msg.args.first() {
                let start = offset_idx * 4;
                let end = start + 4;
                if data.len() >= end {
                    let mut bytes = [0u8; 4];
                    if let Some(slice) = data.get(start..end) {
                        bytes.copy_from_slice(slice);
                        let level = f32::from_le_bytes(bytes);
                        let _ = sender.send(NetworkEvent::MeterLevel(level));
                    }
                }
            }
        } else if msg.path.starts_with("/fx/") && msg.path.ends_with("/type") {
            // Path is /fx/n/type
            // ⚡ Bolt: Use direct byte slice indexing instead of .chars().collect::<Vec<char>>()
            // to extract the FX slot digit. This avoids an O(N) heap allocation per OSC message.
            if let Some(&byte) = msg.path.as_bytes().get(4) {
                if byte.is_ascii_digit() {
                    let slot = (byte - b'0') as usize;
                    if let Some(osc_lib::OscArg::String(s)) = msg.args.first() {
                        let _ = sender.send(NetworkEvent::EffectLoaded(slot, s.clone()));
                    }
                }
            }
        }
        // Check User Controls against configured paths
        else if msg.path.contains(panic_path) {
            // Check if button is PRESSED (val 1)
            if let Some(arg) = msg.args.first() {
                let pressed = match arg {
                    osc_lib::OscArg::Int(i) => *i == 1,
                    osc_lib::OscArg::Float(f) => *f > 0.5,
                    _ => false,
                };
                if pressed {
                    let _ = sender.send(NetworkEvent::PanicTriggered);
                }
            }
        } else if msg.path.contains(enc_path) {
            let _ = sender.send(NetworkEvent::EncoderTurned(1));
        }
    }

    pub async fn set_scribble_text(&self, source: Source, text: &str) -> Result<()> {
        let path = match source {
            Source::Channel(ch) => format!("/ch/{:02}/config/name", ch),
            Source::Bus(b) => format!("/bus/{:02}/config/name", b),
            Source::Aux(a) => format!("/auxin/{:02}/config/name", a),
            Source::MainL | Source::MainR => "/main/st/config/name".to_string(),
        };
        let msg = OscMessage {
            path,
            args: vec![osc_lib::OscArg::String(text.to_string())],
        };
        self.send(&msg).await
    }

    #[allow(dead_code)]
    pub async fn set_scribble_target(&self, target_path: &str, text: &str) -> Result<()> {
        let msg = OscMessage {
            path: target_path.to_string(),
            args: vec![osc_lib::OscArg::String(text.to_string())],
        };
        self.send(&msg).await
    }

    pub async fn set_effect_param(&self, slot: usize, param_idx: usize, value: f32) -> Result<()> {
        let path = format!("/fx/{}/par/{:02}", slot, param_idx);
        let msg = OscMessage {
            path,
            args: vec![osc_lib::OscArg::Float(value)],
        };
        self.send(&msg).await
    }

    pub async fn send_osc_float(&self, path: &str, value: f32) -> Result<()> {
        let msg = OscMessage {
            path: path.to_string(),
            args: vec![osc_lib::OscArg::Float(value)],
        };
        self.send(&msg).await
    }
}
