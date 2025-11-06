use crate::mixer_state::MixerState;
use osc_lib::{OscArg, OscMessage};
use std::net::SocketAddr;

pub fn handle_node_get(
    _state: &mut MixerState,
    msg: &OscMessage,
    _client_addr: SocketAddr,
) -> Result<Vec<OscMessage>, String> {
    if let Some(OscArg::String(node)) = msg.args.get(0) {
        // This is a simplified implementation. A real implementation would need to
        // parse the node string and return the corresponding part of the state tree.
        Ok(vec![OscMessage {
            path: "/node".to_string(),
            args: vec![
                OscArg::String(node.clone()),
                OscArg::String("s:unimplemented".to_string()),
            ],
        }])
    } else {
        Err("Invalid arguments for /node".to_string())
    }
}
