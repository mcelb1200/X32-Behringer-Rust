use crate::mixer_state::MixerState;
use osc_lib::{OscArg, OscMessage};
use std::net::SocketAddr;

pub fn handle_info(
    _state: &mut MixerState,
    _msg: &OscMessage,
    _client_addr: SocketAddr,
) -> Result<Vec<OscMessage>, String> {
    Ok(vec![OscMessage {
        path: "/info".to_string(),
        args: vec![
            OscArg::String("X32-Emulator".to_string()),
            OscArg::String("Rust".to_string()),
            OscArg::String("0.0.1".to_string()),
        ],
    }])
}

pub fn handle_status(
    _state: &mut MixerState,
    _msg: &OscMessage,
    _client_addr: SocketAddr,
) -> Result<Vec<OscMessage>, String> {
    Ok(vec![OscMessage {
        path: "/status".to_string(),
        args: vec![
            OscArg::String("X32-Emulator".to_string()),
            OscArg::String("127.0.0.1".to_string()),
        ],
    }])
}
