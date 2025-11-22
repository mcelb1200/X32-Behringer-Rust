use anyhow::Result;
use crossbeam_channel::Sender;
use osc_lib::OscMessage;
use std::net::UdpSocket;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

pub enum NetworkEvent {
    MeterLevel(f32), // Normalized 0.0-1.0
    PanicTriggered,
    EncoderTurned(i32),   // Delta or Value
    EffectLoaded(String), // Effect name
}

pub struct NetworkManager {
    socket: Arc<UdpSocket>,
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
    pub fn new(
        ip: &str,
        target_channel_idx: usize,
        event_sender: Sender<NetworkEvent>,
        panic_btn_path: &str,
        preset_enc_path: &str,
    ) -> Result<Self> {
        let socket = x32_lib::create_socket("0.0.0.0", 0)?;
        socket.set_read_timeout(Some(Duration::from_millis(100)))?;
        let socket = Arc::new(socket);

        Ok(Self {
            socket,
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
        let msg = OscMessage {
            path: "/xremote".to_string(),
            args: vec![],
        };
        self.send(&msg)?;
        self.connected.store(true, Ordering::Relaxed);
        Ok(())
    }

    pub fn send(&self, msg: &OscMessage) -> Result<()> {
        let bytes = msg.to_bytes()?;
        let target = format!("{}:10023", self.ip);
        self.socket.send_to(&bytes, &target)?;
        Ok(())
    }

    pub fn start_polling(&self, target_slot: usize) {
        let socket = self.socket.clone();
        let sender = self.event_sender.clone();
        let ip = self.ip.clone();
        let is_connected = self.connected.clone();
        let channel_idx = self.target_channel_idx;
        let panic_path = self.panic_btn_path.clone();
        let enc_path = self.preset_enc_path.clone();

        thread::spawn(move || {
            let mut buf = [0u8; 2048];
            let mut last_meter_poll = std::time::Instant::now();
            let mut last_xremote = std::time::Instant::now();
            let mut last_fx_poll = std::time::Instant::now();

            loop {
                if !is_connected.load(Ordering::Relaxed) {
                    thread::sleep(Duration::from_millis(100));
                    continue;
                }

                // 1. Maintain Subscription
                if last_xremote.elapsed() > Duration::from_secs(9) {
                    let renew = OscMessage {
                        path: "/xremote".to_string(),
                        args: vec![],
                    };
                    if let Ok(bytes) = renew.to_bytes() {
                        let _ = socket.send_to(&bytes, format!("{}:10023", ip));
                    }
                    last_xremote = std::time::Instant::now();
                }

                // 2. Poll Meters
                if last_meter_poll.elapsed() > Duration::from_millis(50) {
                    let meter_req = OscMessage {
                        path: "/meters".to_string(),
                        args: vec![osc_lib::OscArg::String("/meters/1".to_string())],
                    };
                    if let Ok(bytes) = meter_req.to_bytes() {
                        let _ = socket.send_to(&bytes, format!("{}:10023", ip));
                    }
                    last_meter_poll = std::time::Instant::now();
                }

                // 3. Poll FX Type
                if last_fx_poll.elapsed() > Duration::from_secs(1) {
                    let fx_path = format!("/fx/{}/type", target_slot);
                    let fx_req = OscMessage {
                        path: fx_path,
                        args: vec![],
                    };
                    if let Ok(bytes) = fx_req.to_bytes() {
                        let _ = socket.send_to(&bytes, format!("{}:10023", ip));
                    }
                    last_fx_poll = std::time::Instant::now();
                }

                // 4. Read Responses
                if let Ok((size, _addr)) = socket.recv_from(&mut buf) {
                    if let Ok(msg) = osc_lib::OscMessage::from_bytes(&buf[..size]) {
                        Self::handle_message(msg, &sender, channel_idx, &panic_path, &enc_path);
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
            if let Some(osc_lib::OscArg::String(s)) = msg.args.first() {
                let _ = sender.send(NetworkEvent::EffectLoaded(s.clone()));
            }
        }
        // Check User Controls against configured paths
        // We allow substring match for flexibility (e.g. "/config/userctrl/A/btn/5" or just "A/btn/5")
        else if msg.path.contains(panic_path) {
            // Check if button is PRESSED (val 1)
            // X32 sends val 1 on press, 0 on release. We only want to trigger on press.
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
            // Encoder turn usually sends delta or new value.
            // For absolute encoders (LED ring), it sends new value.
            // Just trigger a change event.
            let _ = sender.send(NetworkEvent::EncoderTurned(1));
        }
    }

    pub fn set_scribble_text(&self, channel_num: usize, text: &str) -> Result<()> {
        let path = format!("/ch/{:02}/config/name", channel_num);
        let msg = OscMessage {
            path,
            args: vec![osc_lib::OscArg::String(text.to_string())],
        };
        self.send(&msg)
    }

    pub fn set_effect_param(&self, slot: usize, param_idx: usize, value: f32) -> Result<()> {
        let path = format!("/fx/{}/par/{:02}", slot, param_idx);
        let msg = OscMessage {
            path,
            args: vec![osc_lib::OscArg::Float(value)],
        };
        self.send(&msg)
    }
}
