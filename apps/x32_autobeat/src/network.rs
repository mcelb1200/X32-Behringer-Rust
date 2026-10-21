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
use x32_lib::{MixerModel, client::MixerClient};

pub fn max_channels_for_model(model: MixerModel) -> usize {
    match model {
        MixerModel::X32 => 32,
        MixerModel::Wing => 40,
        MixerModel::XR18 | MixerModel::XR16 => 16,
        MixerModel::XR12 => 12,
    }
}

pub fn max_buses_for_model(model: MixerModel) -> usize {
    match model {
        MixerModel::X32 => 16,
        MixerModel::Wing => 28,
        MixerModel::XR18 => 6,
        MixerModel::XR16 => 4,
        MixerModel::XR12 => 2,
    }
}

pub fn max_auxes_for_model(model: MixerModel) -> usize {
    match model {
        MixerModel::X32 | MixerModel::XR18 => 6,
        MixerModel::Wing => 16,
        MixerModel::XR16 | MixerModel::XR12 => 0,
    }
}

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

impl Source {
    pub fn parse_with_model(s: &str, model: MixerModel) -> Result<Self, String> {
        let s = s.trim().to_lowercase();
        if s == "main" || s == "mainl" || s == "main/l" {
            return Ok(Source::MainL);
        }
        if s == "mainr" || s == "main/r" {
            return Ok(Source::MainR);
        }

        let max_ch = max_channels_for_model(model);
        let max_bus = max_buses_for_model(model);
        let max_aux = max_auxes_for_model(model);

        if let Some(num_str) = s.strip_prefix("ch") {
            if let Ok(val) = num_str.parse::<usize>() {
                if (1..=max_ch).contains(&val) {
                    return Ok(Source::Channel(val));
                }
            }
            return Err(format!(
                "Invalid channel number '{}' for model {}",
                s, model
            ));
        }
        if let Some(num_str) = s.strip_prefix("bus") {
            if let Ok(val) = num_str.parse::<usize>() {
                if (1..=max_bus).contains(&val) {
                    return Ok(Source::Bus(val));
                }
            }
            return Err(format!("Invalid bus number '{}' for model {}", s, model));
        }
        if let Some(num_str) = s.strip_prefix("aux") {
            if let Ok(val) = num_str.parse::<usize>() {
                if max_aux > 0 && (1..=max_aux).contains(&val) {
                    return Ok(Source::Aux(val));
                }
            }
            return Err(format!("Invalid aux number '{}' for model {}", s, model));
        }
        if let Ok(val) = s.parse::<usize>() {
            if (1..=max_ch).contains(&val) {
                return Ok(Source::Channel(val));
            }
        }
        Err(format!(
            "Invalid source: '{}' for model {}. Maximum bounds: ch1-{}, bus1-{}, aux1-{}",
            s, model, max_ch, max_bus, max_aux
        ))
    }
}

impl std::str::FromStr for Source {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse_with_model(s, MixerModel::X32)
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
    sources: Vec<Source>,
    panic_btn_path: String,
    preset_enc_path: String,
}

impl NetworkManager {
    pub async fn new(
        ip: &str,
        sources: Vec<Source>,
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
            sources,
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
        let sources = self.sources.clone();
        let panic_path = self.panic_btn_path.clone();
        let enc_path = self.preset_enc_path.clone();

        task::spawn(async move {
            let mut rx = client.subscribe();

            let mut meter_interval = time::interval(Duration::from_millis(50));
            let mut fx_interval = time::interval(Duration::from_secs(2));

            let mut meter_paths = vec![];
            for source in &sources {
                let path = match source {
                    Source::Channel(_) => "/meters/1",
                    Source::Bus(_) | Source::MainL | Source::MainR => "/meters/2",
                    Source::Aux(_) => "/meters/0",
                };
                if !meter_paths.contains(&path) {
                    meter_paths.push(path);
                }
            }

            loop {
                if !is_connected.load(Ordering::Relaxed) {
                    time::sleep(Duration::from_millis(100)).await;
                    continue;
                }

                tokio::select! {
                    _ = meter_interval.tick() => {
                        for path in &meter_paths {
                            let _ = client.send_message("/meters", vec![osc_lib::OscArg::String(path.to_string())]).await;
                        }
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
                                Self::handle_message(msg, &sender, &sources, &panic_path, &enc_path);
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
        sources: &[Source],
        panic_path: &str,
        enc_path: &str,
    ) {
        if msg.path.starts_with("/meters/") {
            if let Some(osc_lib::OscArg::Blob(data)) = msg.args.first() {
                let mut max_level = 0.0_f32;
                let mut found_any = false;

                for source in sources {
                    let (expected_path, offset_idx) = match source {
                        Source::Channel(ch) => ("/meters/1", ch - 1),
                        Source::Bus(b) => ("/meters/2", b - 1),
                        Source::MainL => ("/meters/2", 22),
                        Source::MainR => ("/meters/2", 23),
                        Source::Aux(a) => ("/meters/0", 32 + (a - 1)),
                    };

                    if msg.path == expected_path {
                        let start = offset_idx * 4;
                        let end = start + 4;
                        if data.len() >= end {
                            let mut bytes = [0u8; 4];
                            if let Some(slice) = data.get(start..end) {
                                bytes.copy_from_slice(slice);
                                let level = f32::from_le_bytes(bytes);
                                if level > max_level {
                                    max_level = level;
                                }
                                found_any = true;
                            }
                        }
                    }
                }

                if found_any {
                    let _ = sender.send(NetworkEvent::MeterLevel(max_level));
                }
            }
        } else if msg.path.starts_with("/fx/") && msg.path.ends_with("/type") {
            // Path is /fx/n/type
            // ⚡ Bolt: Avoid creating intermediate Vec simply to get the 5th char
            if let Some(c) = msg.path.chars().nth(4) {
                if let Some(digit) = c.to_digit(10) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_bounds() {
        assert_eq!(max_channels_for_model(MixerModel::X32), 32);
        assert_eq!(max_channels_for_model(MixerModel::Wing), 40);
        assert_eq!(max_channels_for_model(MixerModel::XR18), 16);
        assert_eq!(max_channels_for_model(MixerModel::XR16), 16);
        assert_eq!(max_channels_for_model(MixerModel::XR12), 12);

        assert_eq!(max_buses_for_model(MixerModel::X32), 16);
        assert_eq!(max_buses_for_model(MixerModel::Wing), 28);
        assert_eq!(max_buses_for_model(MixerModel::XR18), 6);
        assert_eq!(max_buses_for_model(MixerModel::XR16), 4);
        assert_eq!(max_buses_for_model(MixerModel::XR12), 2);

        assert_eq!(max_auxes_for_model(MixerModel::X32), 6);
        assert_eq!(max_auxes_for_model(MixerModel::Wing), 16);
        assert_eq!(max_auxes_for_model(MixerModel::XR18), 6);
        assert_eq!(max_auxes_for_model(MixerModel::XR16), 0);
        assert_eq!(max_auxes_for_model(MixerModel::XR12), 0);
    }

    #[test]
    fn test_source_parse_with_model_x32() {
        assert_eq!(
            Source::parse_with_model("ch1", MixerModel::X32),
            Ok(Source::Channel(1))
        );
        assert_eq!(
            Source::parse_with_model("ch32", MixerModel::X32),
            Ok(Source::Channel(32))
        );
        assert!(Source::parse_with_model("ch33", MixerModel::X32).is_err());

        assert_eq!(
            Source::parse_with_model("bus1", MixerModel::X32),
            Ok(Source::Bus(1))
        );
        assert_eq!(
            Source::parse_with_model("bus16", MixerModel::X32),
            Ok(Source::Bus(16))
        );
        assert!(Source::parse_with_model("bus17", MixerModel::X32).is_err());

        assert_eq!(
            Source::parse_with_model("aux1", MixerModel::X32),
            Ok(Source::Aux(1))
        );
        assert_eq!(
            Source::parse_with_model("aux6", MixerModel::X32),
            Ok(Source::Aux(6))
        );
        assert!(Source::parse_with_model("aux7", MixerModel::X32).is_err());

        assert_eq!(
            Source::parse_with_model("main", MixerModel::X32),
            Ok(Source::MainL)
        );
        assert_eq!(
            Source::parse_with_model("main/r", MixerModel::X32),
            Ok(Source::MainR)
        );
    }

    #[test]
    fn test_source_parse_with_model_wing() {
        assert_eq!(
            Source::parse_with_model("ch40", MixerModel::Wing),
            Ok(Source::Channel(40))
        );
        assert!(Source::parse_with_model("ch41", MixerModel::Wing).is_err());

        assert_eq!(
            Source::parse_with_model("bus28", MixerModel::Wing),
            Ok(Source::Bus(28))
        );
        assert!(Source::parse_with_model("bus29", MixerModel::Wing).is_err());

        assert_eq!(
            Source::parse_with_model("aux16", MixerModel::Wing),
            Ok(Source::Aux(16))
        );
        assert!(Source::parse_with_model("aux17", MixerModel::Wing).is_err());
    }

    #[test]
    fn test_source_parse_with_model_xr_series() {
        // XR18
        assert_eq!(
            Source::parse_with_model("ch16", MixerModel::XR18),
            Ok(Source::Channel(16))
        );
        assert!(Source::parse_with_model("ch17", MixerModel::XR18).is_err());
        assert_eq!(
            Source::parse_with_model("bus6", MixerModel::XR18),
            Ok(Source::Bus(6))
        );
        assert!(Source::parse_with_model("bus7", MixerModel::XR18).is_err());
        assert_eq!(
            Source::parse_with_model("aux6", MixerModel::XR18),
            Ok(Source::Aux(6))
        );

        // XR16
        assert_eq!(
            Source::parse_with_model("ch16", MixerModel::XR16),
            Ok(Source::Channel(16))
        );
        assert_eq!(
            Source::parse_with_model("bus4", MixerModel::XR16),
            Ok(Source::Bus(4))
        );
        assert!(Source::parse_with_model("bus5", MixerModel::XR16).is_err());
        assert!(Source::parse_with_model("aux1", MixerModel::XR16).is_err());

        // XR12
        assert_eq!(
            Source::parse_with_model("ch12", MixerModel::XR12),
            Ok(Source::Channel(12))
        );
        assert!(Source::parse_with_model("ch13", MixerModel::XR12).is_err());
        assert_eq!(
            Source::parse_with_model("bus2", MixerModel::XR12),
            Ok(Source::Bus(2))
        );
        assert!(Source::parse_with_model("bus3", MixerModel::XR12).is_err());
        assert!(Source::parse_with_model("aux1", MixerModel::XR12).is_err());
    }
}
