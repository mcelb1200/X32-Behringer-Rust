//! LAN UDP broadcast discovery for Behringer/Midas digital mixers.

use crate::model::default_port_for_model;
use osc_lib::{OscArg, OscMessage};
use std::collections::HashSet;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::UdpSocket;
use x32_fxparse::MixerModel;

/// Metadata for a console discovered on the local network.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredMixer {
    /// Socket address (`ip:port`) of the responding console.
    pub addr: SocketAddr,
    /// Inferred console model (`X32`, `Wing`, `XR18`, `XR16`, `XR12`).
    pub model: MixerModel,
    /// Console name reported in `/info` or `/xinfo`, if available.
    pub name: String,
    /// Raw joined info string returned by the console.
    pub raw_info: String,
}

/// Infers a [`MixerModel`] from a console's `/info` or `/xinfo` response string.
pub fn infer_model_from_info(info: &str) -> MixerModel {
    let upper = info.to_ascii_uppercase();
    if upper.contains("WING") {
        MixerModel::Wing
    } else if upper.contains("XR18") || upper.contains("MR18") {
        MixerModel::XR18
    } else if upper.contains("XR16") {
        MixerModel::XR16
    } else if upper.contains("XR12") || upper.contains("MR12") {
        MixerModel::XR12
    } else {
        MixerModel::X32
    }
}

/// Broadcasts `/info` and `/xinfo` on standard X32/Wing/X-Air UDP ports and collects responses until `timeout`.
pub async fn discover_mixers(timeout: Duration) -> crate::Result<Vec<DiscoveredMixer>> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.set_broadcast(true)?;

    let info_bytes = OscMessage::new("/info".to_string(), vec![]).to_bytes()?;
    let xinfo_bytes = OscMessage::new("/xinfo".to_string(), vec![]).to_bytes()?;

    let ports = [
        default_port_for_model(MixerModel::X32),
        default_port_for_model(MixerModel::XR18),
        default_port_for_model(MixerModel::Wing),
    ];
    for port in ports {
        let target = format!("255.255.255.255:{}", port);
        let _ = socket.send_to(&info_bytes, &target).await;
        let _ = socket.send_to(&xinfo_bytes, &target).await;
    }

    let mut found = Vec::new();
    let mut seen = HashSet::new();
    let mut buf = [0u8; 1024];
    let deadline = tokio::time::Instant::now() + timeout;

    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            break;
        }
        match tokio::time::timeout(remaining, socket.recv_from(&mut buf)).await {
            Ok(Ok((len, addr))) => {
                if !seen.insert(addr) {
                    continue;
                }
                if let Ok(msg) = OscMessage::from_bytes(&buf[..len]) {
                    let parts: Vec<String> = msg
                        .args
                        .iter()
                        .filter_map(|a| match a {
                            OscArg::String(s) => Some(s.clone()),
                            _ => None,
                        })
                        .collect();
                    let raw_info = parts.join(" ");
                    let model = infer_model_from_info(&raw_info);
                    let name = parts
                        .get(1)
                        .or_else(|| parts.first())
                        .cloned()
                        .unwrap_or_else(|| format!("{:?}", model));
                    found.push(DiscoveredMixer {
                        addr,
                        model,
                        name,
                        raw_info,
                    });
                }
            }
            Ok(Err(_)) | Err(_) => break,
        }
    }

    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_model_from_info() {
        assert_eq!(
            infer_model_from_info("V4.06 FOH-X32 X32 4.06"),
            MixerModel::X32
        );
        assert_eq!(infer_model_from_info("WING 2.0"), MixerModel::Wing);
        assert_eq!(infer_model_from_info("XR18 1.18"), MixerModel::XR18);
        assert_eq!(infer_model_from_info("MR18 1.18"), MixerModel::XR18);
        assert_eq!(infer_model_from_info("XR16 1.17"), MixerModel::XR16);
        assert_eq!(infer_model_from_info("XR12 1.17"), MixerModel::XR12);
    }
}
