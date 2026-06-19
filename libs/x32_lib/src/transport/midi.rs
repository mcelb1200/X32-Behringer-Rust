use crate::error::{Result, X32Error};
use crate::transport::MixerTransport;
use async_trait::async_trait;
use midir::{Ignore, MidiInput, MidiOutput, MidiOutputConnection};
use osc_lib::OscMessage;
use std::sync::Mutex;
use tokio::sync::Mutex as TokioMutex;
use tokio::sync::mpsc;

/// A MIDI Sysex transport implementation of `MixerTransport` for USB connection.
pub struct MidiTransport {
    conn_out: Mutex<MidiOutputConnection>,
    rx: TokioMutex<mpsc::UnboundedReceiver<OscMessage>>,
    // Keep the input connection alive by wrapping in Mutex to satisfy Sync bound
    _conn_in: Mutex<midir::MidiInputConnection<()>>,
}

impl MidiTransport {
    /// Connects to the MIDI port matching the target name.
    pub fn connect(port_name: &str) -> Result<Self> {
        let mut midi_in = MidiInput::new("x32_midi_in")
            .map_err(|e| X32Error::Custom(format!("MIDI init error: {}", e)))?;
        midi_in.ignore(Ignore::None);
        let midi_out = MidiOutput::new("x32_midi_out")
            .map_err(|e| X32Error::Custom(format!("MIDI init error: {}", e)))?;

        let in_ports = midi_in.ports();
        let out_ports = midi_out.ports();

        let in_port = Self::find_port(&midi_in, &in_ports, port_name)?;
        let out_port = Self::find_port(&midi_out, &out_ports, port_name)?;

        let (tx, rx) = mpsc::unbounded_channel();
        let conn_in = midi_in
            .connect(
                &in_port,
                "x32_midi_in_conn",
                move |_timestamp, data: &[u8], _| {
                    if data.len() >= 7
                        && data[0] == 0xF0
                        && data[1] == 0x00
                        && data[2] == 0x20
                        && data[3] == 0x32
                        && data[4] == 0x32
                        && data[data.len() - 1] == 0xF7
                    {
                        let payload = &data[5..data.len() - 1];
                        if let Ok(msg) = OscMessage::from_bytes(payload) {
                            let _ = tx.send(msg);
                        }
                    }
                },
                (),
            )
            .map_err(|e| X32Error::Custom(format!("Failed to connect MIDI input: {}", e)))?;

        let conn_out = midi_out
            .connect(&out_port, "x32_midi_out_conn")
            .map_err(|e| X32Error::Custom(format!("Failed to connect MIDI output: {}", e)))?;

        Ok(Self {
            conn_out: Mutex::new(conn_out),
            rx: TokioMutex::new(rx),
            _conn_in: Mutex::new(conn_in),
        })
    }

    fn find_port<T: midir::MidiIO>(
        midi_io: &T,
        ports: &[T::Port],
        target_name: &str,
    ) -> Result<T::Port> {
        if ports.is_empty() {
            return Err(X32Error::Custom("No MIDI ports available".to_string()));
        }

        // 1. Try target_name if provided
        if !target_name.is_empty() {
            for port in ports {
                if let Ok(name) = midi_io.port_name(port) {
                    if name.to_lowercase().contains(&target_name.to_lowercase()) {
                        return Ok(port.clone());
                    }
                }
            }
        }

        // 2. Try common default names for X32 USB card
        for port in ports {
            if let Ok(name) = midi_io.port_name(port) {
                let lower = name.to_lowercase();
                if lower.contains("x-usb")
                    || lower.contains("x-live")
                    || lower.contains("behringer")
                {
                    return Ok(port.clone());
                }
            }
        }

        // 3. Fallback to first port
        Ok(ports[0].clone())
    }
}

#[async_trait]
impl MixerTransport for MidiTransport {
    async fn send(&self, msg: OscMessage) -> Result<()> {
        let osc_bytes = msg.to_bytes()?;
        let mut sysex = Vec::with_capacity(osc_bytes.len() + 6);
        sysex.extend_from_slice(&[0xF0, 0x00, 0x20, 0x32, 0x32]);
        sysex.extend_from_slice(&osc_bytes);
        sysex.push(0xF7);

        let mut conn = self.conn_out.lock().unwrap();
        conn.send(&sysex)
            .map_err(|e| X32Error::Custom(format!("MIDI send error: {}", e)))?;
        Ok(())
    }

    async fn recv(&self) -> Result<OscMessage> {
        let mut rx = self.rx.lock().await;
        rx.recv()
            .await
            .ok_or_else(|| X32Error::Custom("MIDI input closed".to_string()))
    }
}
