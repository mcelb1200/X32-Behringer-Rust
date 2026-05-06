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

pub enum NetworkEvent {
    MeterLevel(f32), // Normalized 0.0-1.0
    PanicTriggered,
    EncoderTurned(i32),          // Delta or Value
    EffectLoaded(usize, String), // Slot, Effect name
}

pub struct NetworkManager {
    client: Arc<MixerClient>,
    ip: String,
    connected: Arc<AtomicBool>,
    event_sender: Sender<NetworkEvent>,
    #[allow(dead_code)]
    panic_subscribed: Arc<AtomicBool>,
    // We need to know which channel to parse from the meter blob
    target_channel_idx: usize,
    panic_btn_path: String,
    preset_enc_path: String,
}

impl NetworkManager {
    pub async fn new(
        ip: &str,
        target_channel_idx: usize,
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
            target_channel_idx,
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
        let channel_idx = self.target_channel_idx;
        let panic_path = self.panic_btn_path.clone();
        let enc_path = self.preset_enc_path.clone();

        task::spawn(async move {
            let mut rx = client.subscribe();

            let mut meter_interval = time::interval(Duration::from_millis(50));
            let mut fx_interval = time::interval(Duration::from_secs(2));

            loop {
                if !is_connected.load(Ordering::Relaxed) {
                    time::sleep(Duration::from_millis(100)).await;
                    continue;
                }

                tokio::select! {
                    _ = meter_interval.tick() => {
                        let _ = client.send_message("/meters", vec![osc_lib::OscArg::String("/meters/1".to_string())]).await;
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
                                Self::handle_message(msg, &sender, channel_idx, &panic_path, &enc_path);
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
        channel_idx: usize,
        panic_path: &str,
        enc_path: &str,
    ) {
        if msg.path == "/meters/1" {
            if let Some(osc_lib::OscArg::Blob(data)) = msg.args.first() {
                let start = channel_idx * 4;
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
            let chars: Vec<char> = msg.path.chars().collect();
            if chars.len() >= 5 {
                if let Some(digit) = chars[4].to_digit(10) {
                    let slot = digit as usize;
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
            // Encoder turn
            // We can try to infer direction from value if it's relative,
            // but for now we just signal 'Next' behavior (+1).
            // Improving this would require knowing if the encoder sends absolute or relative values.
            // X32 Encoders usually send absolute values 0.0-1.0 unless configured otherwise?
            // If it's an assignable encoder, it tracks value.
            // But typically we want to use it as a scroll wheel here.
            // Just trigger an event.
            let _ = sender.send(NetworkEvent::EncoderTurned(1));
        }
    }

    pub async fn set_scribble_text(&self, channel_num: usize, text: &str) -> Result<()> {
        let path = format!("/ch/{:02}/config/name", channel_num);
        let msg = OscMessage {
            path,
            args: vec![osc_lib::OscArg::String(text.to_string())],
        };
        self.send(&msg).await
    }

    /// Set text on a specific Scribble Strip (e.g. Bus 1, DCA 1, etc.)
    /// `target`: e.g., "/bus/01/config/name"
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
