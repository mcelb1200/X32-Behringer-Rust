use crate::error::Result;
use osc_lib::OscMessage;
use async_trait::async_trait;

pub mod udp;
pub mod midi;

/// A trait for abstracting the physical transport layer of the X32/M32 mixer.
#[async_trait]
pub trait MixerTransport: Send + Sync {
    /// Sends an OSC message to the mixer.
    async fn send(&self, msg: OscMessage) -> Result<()>;
    /// Receives the next OSC message from the mixer.
    async fn recv(&self) -> Result<OscMessage>;
}
