use crate::error::Result;
use async_trait::async_trait;
use osc_lib::OscMessage;

/// MIDI transport implementation for communicating with X32/M32 mixers.
pub mod midi;
/// UDP transport implementation for communicating with X32/M32 mixers.
pub mod udp;

/// A trait for abstracting the physical transport layer of the X32/M32 mixer.
#[async_trait]
pub trait MixerTransport: Send + Sync {
    /// Sends an OSC message to the mixer.
    async fn send(&self, msg: OscMessage) -> Result<()>;
    /// Receives the next OSC message from the mixer.
    async fn recv(&self) -> Result<OscMessage>;
}
