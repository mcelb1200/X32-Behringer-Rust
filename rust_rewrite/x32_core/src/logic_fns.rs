use crate::mixer_state::MixerState;
use osc_lib::{OscArg, OscMessage};

pub fn handle_params_str<F>(
    state: &mut MixerState,
    msg: &OscMessage,
    mut setter: F,
) -> Result<Option<Vec<OscMessage>>, String>
where
    F: FnMut(&mut MixerState, &str),
{
    if let Some(OscArg::String(value)) = msg.args.get(0) {
        setter(state, value);
        Ok(None)
    } else {
        Err("Invalid arguments".to_string())
    }
}

pub fn handle_params_i32<F>(
    state: &mut MixerState,
    msg: &OscMessage,
    mut setter: F,
) -> Result<Option<Vec<OscMessage>>, String>
where
    F: FnMut(&mut MixerState, i32),
{
    if let Some(OscArg::Int(value)) = msg.args.get(0) {
        setter(state, *value);
        Ok(None)
    } else {
        Err("Invalid arguments".to_string())
    }
}

pub fn handle_params_f32<F>(
    state: &mut MixerState,
    msg: &OscMessage,
    mut setter: F,
) -> Result<Option<Vec<OscMessage>>, String>
where
    F: FnMut(&mut MixerState, f32),
{
    if let Some(OscArg::Float(value)) = msg.args.get(0) {
        setter(state, *value);
        Ok(None)
    } else {
        Err("Invalid arguments".to_string())
    }
}
