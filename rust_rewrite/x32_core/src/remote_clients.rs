use crate::mixer_state::MixerState;
use osc_lib::OscMessage;
use std::net::SocketAddr;

pub fn handle_subscribe(
    _state: &mut MixerState,
    _msg: &OscMessage,
    _client_addr: SocketAddr,
) -> Result<Vec<OscMessage>, String> {
    Ok(vec![])
}
