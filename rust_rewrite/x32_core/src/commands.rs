use std::net::SocketAddr;

use crate::mixer_state::MixerState;
use osc_lib::OscMessage;

pub struct DeserializationConfig {
    pub types: String,
}

impl DeserializationConfig {
    pub fn new(types: &str) -> Self {
        DeserializationConfig {
            types: types.to_string(),
        }
    }
}

pub enum Command {
    Special {
        handler: Box<dyn Fn(&mut MixerState, &OscMessage, SocketAddr) -> Result<Vec<OscMessage>, String>>,
    },
    Params {
        deserialization_config: DeserializationConfig,
        set_handler:
            Box<dyn Fn(&mut MixerState, &OscMessage) -> Result<Option<Vec<OscMessage>>, String>>,
        get_handler: Box<dyn Fn(&MixerState, &OscMessage) -> Result<Vec<OscMessage>, String>>,
    },
}

impl Command {
    pub fn new_special(
        handler: Box<dyn Fn(&mut MixerState, &OscMessage, SocketAddr) -> Result<Vec<OscMessage>, String>>,
    ) -> Self {
        Command::Special { handler }
    }

    pub fn new_params(
        deserialization_config: DeserializationConfig,
        set_handler: Box<
            dyn Fn(&mut MixerState, &OscMessage) -> Result<Option<Vec<OscMessage>>, String>,
        >,
        get_handler: Box<dyn Fn(&MixerState, &OscMessage) -> Result<Vec<OscMessage>, String>>,
    ) -> Self {
        Command::Params {
            deserialization_config,
            set_handler,
            get_handler,
        }
    }
}
