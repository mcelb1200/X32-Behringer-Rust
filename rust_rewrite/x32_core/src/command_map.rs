use std::collections::HashMap;

use crate::commands::{Command, DeserializationConfig};
use crate::logic_fns::{handle_params_f32, handle_params_i32, handle_params_str};
use crate::mixer_state::MixerState;
use crate::node_tree::handle_node_get;
use crate::remote_clients::handle_subscribe;
use crate::state_management::{handle_delete, handle_load, handle_save};
use crate::status::{handle_info, handle_status};

pub fn build_command_map() -> HashMap<String, Command> {
    let mut command_map: HashMap<String, Command> = HashMap::new();

    // Special Commands
    command_map.insert(
        "/info".to_string(),
        Command::new_special(Box::new(handle_info)),
    );
    command_map.insert(
        "/status".to_string(),
        Command::new_special(Box::new(handle_status)),
    );
    command_map.insert(
        "/subscribe".to_string(),
        Command::new_special(Box::new(handle_subscribe)),
    );
    command_map.insert(
        "/node".to_string(),
        Command::new_special(Box::new(handle_node_get)),
    );
    command_map.insert(
        "/-snap/load".to_string(),
        Command::new_special(Box::new(handle_load)),
    );
    command_map.insert(
        "/-snap/save".to_string(),
        Command::new_special(Box::new(handle_save)),
    );
    command_map.insert(
        "/-snap/delete".to_string(),
        Command::new_special(Box::new(handle_delete)),
    );

    // Parameter Commands
    for i in 1..=32 {
        let channel_index = (i - 1) as usize;

        // Config
        command_map.insert(
            format!("/ch/{:02}/config/name", i),
            Command::new_params(
                DeserializationConfig::new("s"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_str(state, msg, |s, v| {
                        s.channels[channel_index].config.name = v.to_string();
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![format!(
                        "s:{}",
                        state.channels[channel_index].config.name.clone()
                    )
                    .into()])
                }),
            ),
        );
        command_map.insert(
            format!("/ch/{:02}/config/icon", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.channels[channel_index].config.icon = v as u32;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("i:{}", state.channels[channel_index].config.icon).into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/ch/{:02}/config/color", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.channels[channel_index].config.color = v as u32;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("i:{}", state.channels[channel_index].config.color).into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/ch/{:02}/config/source", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.channels[channel_index].config.source = v as u32;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("i:{}", state.channels[channel_index].config.source).into(),
                    ])
                }),
            ),
        );

        // Group
        command_map.insert(
            format!("/ch/{:02}/grp/dca", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.channels[channel_index].grp.dca = v as u32;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("i:{}", state.channels[channel_index].grp.dca).into()
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/ch/{:02}/grp/mute", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.channels[channel_index].grp.mute = v as u32;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("i:{}", state.channels[channel_index].grp.mute).into(),
                    ])
                }),
            ),
        );

        // Preamp
        command_map.insert(
            format!("/ch/{:02}/preamp/trim", i),
            Command::new_params(
                DeserializationConfig::new("f"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_f32(state, msg, |s, v| {
                        s.channels[channel_index].preamp.trim = v;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("f:{}", state.channels[channel_index].preamp.trim).into(),
                    ])
                }),
            ),
        );

        // Delay
        command_map.insert(
            format!("/ch/{:02}/delay/on", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.channels[channel_index].delay.on = v != 0;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!(
                            "i:{}",
                            if state.channels[channel_index].delay.on {
                                1
                            } else {
                                0
                            }
                        )
                        .into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/ch/{:02}/delay/time", i),
            Command::new_params(
                DeserializationConfig::new("f"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_f32(state, msg, |s, v| {
                        s.channels[channel_index].delay.time = v;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("f:{}", state.channels[channel_index].delay.time).into(),
                    ])
                }),
            ),
        );

        // Insert
        command_map.insert(
            format!("/ch/{:02}/insert/on", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.channels[channel_index].insert.on = v != 0;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!(
                            "i:{}",
                            if state.channels[channel_index].insert.on {
                                1
                            } else {
                                0
                            }
                        )
                        .into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/ch/{:02}/insert/pos", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.channels[channel_index].insert.pos = v as u32;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("i:{}", state.channels[channel_index].insert.pos).into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/ch/{:02}/insert/sel", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.channels[channel_index].insert.sel = v as u32;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("i:{}", state.channels[channel_index].insert.sel).into(),
                    ])
                }),
            ),
        );

        // Gate
        command_map.insert(
            format!("/ch/{:02}/gate/on", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.channels[channel_index].gate.on = v != 0;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!(
                            "i:{}",
                            if state.channels[channel_index].gate.on {
                                1
                            } else {
                                0
                            }
                        )
                        .into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/ch/{:02}/gate/mode", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.channels[channel_index].gate.mode = v as u32;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("i:{}", state.channels[channel_index].gate.mode).into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/ch/{:02}/gate/thr", i),
            Command::new_params(
                DeserializationConfig::new("f"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_f32(state, msg, |s, v| {
                        s.channels[channel_index].gate.thr = v;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("f:{}", state.channels[channel_index].gate.thr).into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/ch/{:02}/gate/range", i),
            Command::new_params(
                DeserializationConfig::new("f"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_f32(state, msg, |s, v| {
                        s.channels[channel_index].gate.range = v;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("f:{}", state.channels[channel_index].gate.range).into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/ch/{:02}/gate/attack", i),
            Command::new_params(
                DeserializationConfig::new("f"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_f32(state, msg, |s, v| {
                        s.channels[channel_index].gate.attack = v;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("f:{}", state.channels[channel_index].gate.attack).into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/ch/{:02}/gate/hold", i),
            Command::new_params(
                DeserializationConfig::new("f"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_f32(state, msg, |s, v| {
                        s.channels[channel_index].gate.hold = v;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("f:{}", state.channels[channel_index].gate.hold).into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/ch/{:02}/gate/release", i),
            Command::new_params(
                DeserializationConfig::new("f"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_f32(state, msg, |s, v| {
                        s.channels[channel_index].gate.release = v;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("f:{}", state.channels[channel_index].gate.release).into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/ch/{:02}/gate/keysrc", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.channels[channel_index].gate.keysrc = v as u32;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("i:{}", state.channels[channel_index].gate.keysrc).into(),
                    ])
                }),
            ),
        );

        // Dyn
        command_map.insert(
            format!("/ch/{:02}/dyn/on", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.channels[channel_index].dyn.on = v != 0;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!(
                            "i:{}",
                            if state.channels[channel_index].dyn.on {
                                1
                            } else {
                                0
                            }
                        )
                        .into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/ch/{:02}/dyn/mode", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.channels[channel_index].dyn.mode = v as u32;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("i:{}", state.channels[channel_index].dyn.mode).into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/ch/{:02}/dyn/det", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.channels[channel_index].dyn.det = v as u32;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("i:{}", state.channels[channel_index].dyn.det).into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/ch/{:02}/dyn/env", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.channels[channel_index].dyn.env = v as u32;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("i:{}", state.channels[channel_index].dyn.env).into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/ch/{:02}/dyn/thr", i),
            Command::new_params(
                DeserializationConfig::new("f"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_f32(state, msg, |s, v| {
                        s.channels[channel_index].dyn.thr = v;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("f:{}", state.channels[channel_index].dyn.thr).into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/ch/{:02}/dyn/ratio", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.channels[channel_index].dyn.ratio = v as u32;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("i:{}", state.channels[channel_index].dyn.ratio).into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/ch/{:02}/dyn/knee", i),
            Command::new_params(
                DeserializationConfig::new("f"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_f32(state, msg, |s, v| {
                        s.channels[channel_index].dyn.knee = v;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("f:{}", state.channels[channel_index].dyn.knee).into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/ch/{:02}/dyn/mgain", i),
            Command::new_params(
                DeserializationConfig::new("f"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_f32(state, msg, |s, v| {
                        s.channels[channel_index].dyn.mgain = v;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("f:{}", state.channels[channel_index].dyn.mgain).into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/ch/{:02}/dyn/attack", i),
            Command::new_params(
                DeserializationConfig::new("f"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_f32(state, msg, |s, v| {
                        s.channels[channel_index].dyn.attack = v;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("f:{}", state.channels[channel_index].dyn.attack).into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/ch/{:02}/dyn/hold", i),
            Command::new_params(
                DeserializationConfig::new("f"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_f32(state, msg, |s, v| {
                        s.channels[channel_index].dyn.hold = v;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("f:{}", state.channels[channel_index].dyn.hold).into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/ch/{:02}/dyn/release", i),
            Command::new_params(
                DeserializationConfig::new("f"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_f32(state, msg, |s, v| {
                        s.channels[channel_index].dyn.release = v;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("f:{}", state.channels[channel_index].dyn.release).into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/ch/{:02}/dyn/pos", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.channels[channel_index].dyn.pos = v as u32;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("i:{}", state.channels[channel_index].dyn.pos).into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/ch/{:02}/dyn/keysrc", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.channels[channel_index].dyn.keysrc = v as u32;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("i:{}", state.channels[channel_index].dyn.keysrc).into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/ch/{:02}/dyn/mix", i),
            Command::new_params(
                DeserializationConfig::new("f"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_f32(state, msg, |s, v| {
                        s.channels[channel_index].dyn.mix = v;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("f:{}", state.channels[channel_index].dyn.mix).into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/ch/{:02}/dyn/auto", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.channels[channel_index].dyn.auto = v != 0;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!(
                            "i:{}",
                            if state.channels[channel_index].dyn.auto {
                                1
                            } else {
                                0
                            }
                        )
                        .into(),
                    ])
                }),
            ),
        );

        // EQ
        command_map.insert(
            format!("/ch/{:02}/eq/on", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.channels[channel_index].eq.on = v != 0;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!(
                            "i:{}",
                            if state.channels[channel_index].eq.on {
                                1
                            } else {
                                0
                            }
                        )
                        .into(),
                    ])
                }),
            ),
        );
        for band in 1..=4 {
            let band_index = (band - 1) as usize;
            command_map.insert(
                format!("/ch/{:02}/eq/{}/type", i, band),
                Command::new_params(
                    DeserializationConfig::new("i"),
                    Box::new(move |state: &mut MixerState, msg| {
                        handle_params_i32(state, msg, |s, v| {
                            s.channels[channel_index].eq.bands[band_index].eq_type = v as u32;
                        })
                    }),
                    Box::new(move |state: &MixerState, _| {
                        Ok(vec![format!(
                            "i:{}",
                            state.channels[channel_index].eq.bands[band_index].eq_type
                        )
                        .into()])
                    }),
                ),
            );
            command_map.insert(
                format!("/ch/{:02}/eq/{}/f", i, band),
                Command::new_params(
                    DeserializationConfig::new("f"),
                    Box::new(move |state: &mut MixerState, msg| {
                        handle_params_f32(state, msg, |s, v| {
                            s.channels[channel_index].eq.bands[band_index].f = v;
                        })
                    }),
                    Box::new(move |state: &MixerState, _| {
                        Ok(vec![format!(
                            "f:{}",
                            state.channels[channel_index].eq.bands[band_index].f
                        )
                        .into()])
                    }),
                ),
            );
            command_map.insert(
                format!("/ch/{:02}/eq/{}/g", i, band),
                Command::new_params(
                    DeserializationConfig::new("f"),
                    Box::new(move |state: &mut MixerState, msg| {
                        handle_params_f32(state, msg, |s, v| {
                            s.channels[channel_index].eq.bands[band_index].g = v;
                        })
                    }),
                    Box::new(move |state: &MixerState, _| {
                        Ok(vec![format!(
                            "f:{}",
                            state.channels[channel_index].eq.bands[band_index].g
                        )
                        .into()])
                    }),
                ),
            );
            command_map.insert(
                format!("/ch/{:02}/eq/{}/q", i, band),
                Command::new_params(
                    DeserializationConfig::new("f"),
                    Box::new(move |state: &mut MixerState, msg| {
                        handle_params_f32(state, msg, |s, v| {
                            s.channels[channel_index].eq.bands[band_index].q = v;
                        })
                    }),
                    Box::new(move |state: &MixerState, _| {
                        Ok(vec![format!(
                            "f:{}",
                            state.channels[channel_index].eq.bands[band_index].q
                        )
                        .into()])
                    }),
                ),
            );
        }

        // Mix
        command_map.insert(
            format!("/ch/{:02}/mix/on", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.channels[channel_index].mix.on = v != 0;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!(
                            "i:{}",
                            if state.channels[channel_index].mix.on {
                                1
                            } else {
                                0
                            }
                        )
                        .into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/ch/{:02}/mix/fader", i),
            Command::new_params(
                DeserializationConfig::new("f"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_f32(state, msg, |s, v| {
                        s.channels[channel_index].mix.fader = v;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("f:{}", state.channels[channel_index].mix.fader).into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/ch/{:02}/mix/st", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.channels[channel_index].mix.st = v != 0;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!(
                            "i:{}",
                            if state.channels[channel_index].mix.st {
                                1
                            } else {
                                0
                            }
                        )
                        .into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/ch/{:02}/mix/pan", i),
            Command::new_params(
                DeserializationConfig::new("f"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_f32(state, msg, |s, v| {
                        s.channels[channel_index].mix.pan = v;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("f:{}", state.channels[channel_index].mix.pan).into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/ch/{:02}/mix/mono", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.channels[channel_index].mix.mono = v != 0;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!(
                            "i:{}",
                            if state.channels[channel_index].mix.mono {
                                1
                            } else {
                                0
                            }
                        )
                        .into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/ch/{:02}/mix/mlevel", i),
            Command::new_params(
                DeserializationConfig::new("f"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_f32(state, msg, |s, v| {
                        s.channels[channel_index].mix.mlevel = v;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("f:{}", state.channels[channel_index].mix.mlevel).into(),
                    ])
                }),
            ),
        );
        for bus in 1..=16 {
            let bus_index = (bus - 1) as usize;
            command_map.insert(
                format!("/ch/{:02}/mix/{:02}/on", i, bus),
                Command::new_params(
                    DeserializationConfig::new("i"),
                    Box::new(move |state: &mut MixerState, msg| {
                        handle_params_i32(state, msg, |s, v| {
                            s.channels[channel_index].mix.sends[bus_index].on = v != 0;
                        })
                    }),
                    Box::new(move |state: &MixerState, _| {
                        Ok(vec![
                            format!(
                                "i:{}",
                                if state.channels[channel_index].mix.sends[bus_index].on {
                                    1
                                } else {
                                    0
                                }
                            )
                            .into(),
                        ])
                    }),
                ),
            );
            command_map.insert(
                format!("/ch/{:02}/mix/{:02}/level", i, bus),
                Command::new_params(
                    DeserializationConfig::new("f"),
                    Box::new(move |state: &mut MixerState, msg| {
                        handle_params_f32(state, msg, |s, v| {
                            s.channels[channel_index].mix.sends[bus_index].level = v;
                        })
                    }),
                    Box::new(move |state: &MixerState, _| {
                        Ok(vec![format!(
                            "f:{}",
                            state.channels[channel_index].mix.sends[bus_index].level
                        )
                        .into()])
                    }),
                ),
            );
        }

        // Automix
        command_map.insert(
            format!("/ch/{:02}/automix/group", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.channels[channel_index].automix.group = v as u32;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("i:{}", state.channels[channel_index].automix.group).into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/ch/{:02}/automix/weight", i),
            Command::new_params(
                DeserializationConfig::new("f"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_f32(state, msg, |s, v| {
                        s.channels[channel_index].automix.weight = v;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("f:{}", state.channels[channel_index].automix.weight).into(),
                    ])
                }),
            ),
        );
    }

    for i in 1..=6 {
        let mtx_index = (i - 1) as usize;

        // Config
        command_map.insert(
            format!("/mtx/{:02}/config/name", i),
            Command::new_params(
                DeserializationConfig::new("s"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_str(state, msg, |s, v| {
                        s.mtx[mtx_index].config.name = v.to_string();
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![format!(
                        "s:{}",
                        state.mtx[mtx_index].config.name.clone()
                    )
                    .into()])
                }),
            ),
        );
        command_map.insert(
            format!("/mtx/{:02}/config/icon", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.mtx[mtx_index].config.icon = v as u32;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("i:{}", state.mtx[mtx_index].config.icon).into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/mtx/{:02}/config/color", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.mtx[mtx_index].config.color = v as u32;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("i:{}", state.mtx[mtx_index].config.color).into(),
                    ])
                }),
            ),
        );

        // Preamp
        command_map.insert(
            format!("/mtx/{:02}/preamp/invert", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.mtx[mtx_index].preamp.invert = v != 0;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!(
                            "i:{}",
                            if state.mtx[mtx_index].preamp.invert {
                                1
                            } else {
                                0
                            }
                        )
                        .into(),
                    ])
                }),
            ),
        );

        // Dyn
        command_map.insert(
            format!("/mtx/{:02}/dyn/on", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.mtx[mtx_index].dyn.on = v != 0;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!(
                            "i:{}",
                            if state.mtx[mtx_index].dyn.on { 1 } else { 0 }
                        )
                        .into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/mtx/{:02}/dyn/mode", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.mtx[mtx_index].dyn.mode = v as u32;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![format!("i:{}", state.mtx[mtx_index].dyn.mode).into()])
                }),
            ),
        );
        command_map.insert(
            format!("/mtx/{:02}/dyn/det", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.mtx[mtx_index].dyn.det = v as u32;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![format!("i:{}", state.mtx[mtx_index].dyn.det).into()])
                }),
            ),
        );
        command_map.insert(
            format!("/mtx/{:02}/dyn/env", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.mtx[mtx_index].dyn.env = v as u32;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![format!("i:{}", state.mtx[mtx_index].dyn.env).into()])
                }),
            ),
        );
        command_map.insert(
            format!("/mtx/{:02}/dyn/thr", i),
            Command::new_params(
                DeserializationConfig::new("f"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_f32(state, msg, |s, v| {
                        s.mtx[mtx_index].dyn.thr = v;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![format!("f:{}", state.mtx[mtx_index].dyn.thr).into()])
                }),
            ),
        );
        command_map.insert(
            format!("/mtx/{:02}/dyn/ratio", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.mtx[mtx_index].dyn.ratio = v as u32;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("i:{}", state.mtx[mtx_index].dyn.ratio).into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/mtx/{:02}/dyn/knee", i),
            Command::new_params(
                DeserializationConfig::new("f"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_f32(state, msg, |s, v| {
                        s.mtx[mtx_index].dyn.knee = v;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![format!("f:{}", state.mtx[mtx_index].dyn.knee).into()])
                }),
            ),
        );
        command_map.insert(
            format!("/mtx/{:02}/dyn/mgain", i),
            Command::new_params(
                DeserializationConfig::new("f"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_f32(state, msg, |s, v| {
                        s.mtx[mtx_index].dyn.mgain = v;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("f:{}", state.mtx[mtx_index].dyn.mgain).into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/mtx/{:02}/dyn/attack", i),
            Command::new_params(
                DeserializationConfig::new("f"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_f32(state, msg, |s, v| {
                        s.mtx[mtx_index].dyn.attack = v;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("f:{}", state.mtx[mtx_index].dyn.attack).into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/mtx/{:02}/dyn/hold", i),
            Command::new_params(
                DeserializationConfig::new("f"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_f32(state, msg, |s, v| {
                        s.mtx[mtx_index].dyn.hold = v;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![format!("f:{}", state.mtx[mtx_index].dyn.hold).into()])
                }),
            ),
        );
        command_map.insert(
            format!("/mtx/{:02}/dyn/release", i),
            Command::new_params(
                DeserializationConfig::new("f"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_f32(state, msg, |s, v| {
                        s.mtx[mtx_index].dyn.release = v;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("f:{}", state.mtx[mtx_index].dyn.release).into(),
                    ])
                }),
            ),
        );

        // Insert
        command_map.insert(
            format!("/mtx/{:02}/insert/on", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.mtx[mtx_index].insert.on = v != 0;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!(
                            "i:{}",
                            if state.mtx[mtx_index].insert.on {
                                1
                            } else {
                                0
                            }
                        )
                        .into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/mtx/{:02}/insert/pos", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.mtx[mtx_index].insert.pos = v as u32;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("i:{}", state.mtx[mtx_index].insert.pos).into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/mtx/{:02}/insert/sel", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.mtx[mtx_index].insert.sel = v as u32;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("i:{}", state.mtx[mtx_index].insert.sel).into(),
                    ])
                }),
            ),
        );

        // EQ
        command_map.insert(
            format!("/mtx/{:02}/eq/on", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.mtx[mtx_index].eq.on = v != 0;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!(
                            "i:{}",
                            if state.mtx[mtx_index].eq.on { 1 } else { 0 }
                        )
                        .into(),
                    ])
                }),
            ),
        );
        for band in 1..=6 {
            let band_index = (band - 1) as usize;
            command_map.insert(
                format!("/mtx/{:02}/eq/{}/type", i, band),
                Command::new_params(
                    DeserializationConfig::new("i"),
                    Box::new(move |state: &mut MixerState, msg| {
                        handle_params_i32(state, msg, |s, v| {
                            s.mtx[mtx_index].eq.bands[band_index].eq_type = v as u32;
                        })
                    }),
                    Box::new(move |state: &MixerState, _| {
                        Ok(vec![format!(
                            "i:{}",
                            state.mtx[mtx_index].eq.bands[band_index].eq_type
                        )
                        .into()])
                    }),
                ),
            );
            command_map.insert(
                format!("/mtx/{:02}/eq/{}/f", i, band),
                Command::new_params(
                    DeserializationConfig::new("f"),
                    Box::new(move |state: &mut MixerState, msg| {
                        handle_params_f32(state, msg, |s, v| {
                            s.mtx[mtx_index].eq.bands[band_index].f = v;
                        })
                    }),
                    Box::new(move |state: &MixerState, _| {
                        Ok(vec![format!(
                            "f:{}",
                            state.mtx[mtx_index].eq.bands[band_index].f
                        )
                        .into()])
                    }),
                ),
            );
            command_map.insert(
                format!("/mtx/{:02}/eq/{}/g", i, band),
                Command::new_params(
                    DeserializationConfig::new("f"),
                    Box::new(move |state: &mut MixerState, msg| {
                        handle_params_f32(state, msg, |s, v| {
                            s.mtx[mtx_index].eq.bands[band_index].g = v;
                        })
                    }),
                    Box::new(move |state: &MixerState, _| {
                        Ok(vec![format!(
                            "f:{}",
                            state.mtx[mtx_index].eq.bands[band_index].g
                        )
                        .into()])
                    }),
                ),
            );
            command_map.insert(
                format!("/mtx/{:02}/eq/{}/q", i, band),
                Command::new_params(
                    DeserializationConfig::new("f"),
                    Box::new(move |state: &mut MixerState, msg| {
                        handle_params_f32(state, msg, |s, v| {
                            s.mtx[mtx_index].eq.bands[band_index].q = v;
                        })
                    }),
                    Box::new(move |state: &MixerState, _| {
                        Ok(vec![format!(
                            "f:{}",
                            state.mtx[mtx_index].eq.bands[band_index].q
                        )
                        .into()])
                    }),
                ),
            );
        }

        // Mix
        command_map.insert(
            format!("/mtx/{:02}/mix/on", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.mtx[mtx_index].mix.on = v != 0;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!(
                            "i:{}",
                            if state.mtx[mtx_index].mix.on { 1 } else { 0 }
                        )
                        .into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/mtx/{:02}/mix/fader", i),
            Command::new_params(
                DeserializationConfig::new("f"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_f32(state, msg, |s, v| {
                        s.mtx[mtx_index].mix.fader = v;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("f:{}", state.mtx[mtx_index].mix.fader).into(),
                    ])
                }),
            ),
        );

        // Group
        command_map.insert(
            format!("/mtx/{:02}/grp/dca", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.mtx[mtx_index].grp.dca = v as u32;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![format!("i:{}", state.mtx[mtx_index].grp.dca).into()])
                }),
            ),
        );
        command_map.insert(
            format!("/mtx/{:02}/grp/mute", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.mtx[mtx_index].grp.mute = v as u32;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![format!("i:{}", state.mtx[mtx_index].grp.mute).into()])
                }),
            ),
        );
    }

    for i in 1..=8 {
        let auxin_index = (i - 1) as usize;

        // Config
        command_map.insert(
            format!("/auxin/{:02}/config/name", i),
            Command::new_params(
                DeserializationConfig::new("s"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_str(state, msg, |s, v| {
                        s.auxin[auxin_index].config.name = v.to_string();
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![format!(
                        "s:{}",
                        state.auxin[auxin_index].config.name.clone()
                    )
                    .into()])
                }),
            ),
        );
        command_map.insert(
            format!("/auxin/{:02}/config/icon", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.auxin[auxin_index].config.icon = v as u32;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("i:{}", state.auxin[auxin_index].config.icon).into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/auxin/{:02}/config/color", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.auxin[auxin_index].config.color = v as u32;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("i:{}", state.auxin[auxin_index].config.color).into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/auxin/{:02}/config/source", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.auxin[auxin_index].config.source = v as u32;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("i:{}", state.auxin[auxin_index].config.source).into(),
                    ])
                }),
            ),
        );

        // Group
        command_map.insert(
            format!("/auxin/{:02}/grp/dca", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.auxin[auxin_index].grp.dca = v as u32;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![format!("i:{}", state.auxin[auxin_index].grp.dca).into()])
                }),
            ),
        );
        command_map.insert(
            format!("/auxin/{:02}/grp/mute", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.auxin[auxin_index].grp.mute = v as u32;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![format!("i:{}", state.auxin[auxin_index].grp.mute).into()])
                }),
            ),
        );

        // Preamp
        command_map.insert(
            format!("/auxin/{:02}/preamp/trim", i),
            Command::new_params(
                DeserializationConfig::new("f"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_f32(state, msg, |s, v| {
                        s.auxin[auxin_index].preamp.trim = v;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("f:{}", state.auxin[auxin_index].preamp.trim).into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/auxin/{:02}/preamp/invert", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.auxin[auxin_index].preamp.invert = v != 0;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!(
                            "i:{}",
                            if state.auxin[auxin_index].preamp.invert {
                                1
                            } else {
                                0
                            }
                        )
                        .into(),
                    ])
                }),
            ),
        );

        // EQ
        command_map.insert(
            format!("/auxin/{:02}/eq/on", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.auxin[auxin_index].eq.on = v != 0;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!(
                            "i:{}",
                            if state.auxin[auxin_index].eq.on {
                                1
                            } else {
                                0
                            }
                        )
                        .into(),
                    ])
                }),
            ),
        );
        for band in 1..=4 {
            let band_index = (band - 1) as usize;
            command_map.insert(
                format!("/auxin/{:02}/eq/{}/type", i, band),
                Command::new_params(
                    DeserializationConfig::new("i"),
                    Box::new(move |state: &mut MixerState, msg| {
                        handle_params_i32(state, msg, |s, v| {
                            s.auxin[auxin_index].eq.bands[band_index].eq_type = v as u32;
                        })
                    }),
                    Box::new(move |state: &MixerState, _| {
                        Ok(vec![format!(
                            "i:{}",
                            state.auxin[auxin_index].eq.bands[band_index].eq_type
                        )
                        .into()])
                    }),
                ),
            );
            command_map.insert(
                format!("/auxin/{:02}/eq/{}/f", i, band),
                Command::new_params(
                    DeserializationConfig::new("f"),
                    Box::new(move |state: &mut MixerState, msg| {
                        handle_params_f32(state, msg, |s, v| {
                            s.auxin[auxin_index].eq.bands[band_index].f = v;
                        })
                    }),
                    Box::new(move |state: &MixerState, _| {
                        Ok(vec![format!(
                            "f:{}",
                            state.auxin[auxin_index].eq.bands[band_index].f
                        )
                        .into()])
                    }),
                ),
            );
            command_map.insert(
                format!("/auxin/{:02}/eq/{}/g", i, band),
                Command::new_params(
                    DeserializationConfig::new("f"),
                    Box::new(move |state: &mut MixerState, msg| {
                        handle_params_f32(state, msg, |s, v| {
                            s.auxin[auxin_index].eq.bands[band_index].g = v;
                        })
                    }),
                    Box::new(move |state: &MixerState, _| {
                        Ok(vec![format!(
                            "f:{}",
                            state.auxin[auxin_index].eq.bands[band_index].g
                        )
                        .into()])
                    }),
                ),
            );
            command_map.insert(
                format!("/auxin/{:02}/eq/{}/q", i, band),
                Command::new_params(
                    DeserializationConfig::new("f"),
                    Box::new(move |state: &mut MixerState, msg| {
                        handle_params_f32(state, msg, |s, v| {
                            s.auxin[auxin_index].eq.bands[band_index].q = v;
                        })
                    }),
                    Box::new(move |state: &MixerState, _| {
                        Ok(vec![format!(
                            "f:{}",
                            state.auxin[auxin_index].eq.bands[band_index].q
                        )
                        .into()])
                    }),
                ),
            );
        }

        // Mix
        command_map.insert(
            format!("/auxin/{:02}/mix/on", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.auxin[auxin_index].mix.on = v != 0;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!(
                            "i:{}",
                            if state.auxin[auxin_index].mix.on {
                                1
                            } else {
                                0
                            }
                        )
                        .into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/auxin/{:02}/mix/fader", i),
            Command::new_params(
                DeserializationConfig::new("f"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_f32(state, msg, |s, v| {
                        s.auxin[auxin_index].mix.fader = v;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("f:{}", state.auxin[auxin_index].mix.fader).into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/auxin/{:02}/mix/st", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.auxin[auxin_index].mix.st = v != 0;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!(
                            "i:{}",
                            if state.auxin[auxin_index].mix.st {
                                1
                            } else {
                                0
                            }
                        )
                        .into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/auxin/{:02}/mix/pan", i),
            Command::new_params(
                DeserializationConfig::new("f"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_f32(state, msg, |s, v| {
                        s.auxin[auxin_index].mix.pan = v;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![format!("f:{}", state.auxin[auxin_index].mix.pan).into()])
                }),
            ),
        );
        command_map.insert(
            format!("/auxin/{:02}/mix/mono", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.auxin[auxin_index].mix.mono = v != 0;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!(
                            "i:{}",
                            if state.auxin[auxin_index].mix.mono {
                                1
                            } else {
                                0
                            }
                        )
                        .into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/auxin/{:02}/mix/mlevel", i),
            Command::new_params(
                DeserializationConfig::new("f"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_f32(state, msg, |s, v| {
                        s.auxin[auxin_index].mix.mlevel = v;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("f:{}", state.auxin[auxin_index].mix.mlevel).into(),
                    ])
                }),
            ),
        );
        for bus in 1..=16 {
            let bus_index = (bus - 1) as usize;
            command_map.insert(
                format!("/auxin/{:02}/mix/{:02}/on", i, bus),
                Command::new_params(
                    DeserializationConfig::new("i"),
                    Box::new(move |state: &mut MixerState, msg| {
                        handle_params_i32(state, msg, |s, v| {
                            s.auxin[auxin_index].mix.sends[bus_index].on = v != 0;
                        })
                    }),
                    Box::new(move |state: &MixerState, _| {
                        Ok(vec![
                            format!(
                                "i:{}",
                                if state.auxin[auxin_index].mix.sends[bus_index].on {
                                    1
                                } else {
                                    0
                                }
                            )
                            .into(),
                        ])
                    }),
                ),
            );
            command_map.insert(
                format!("/auxin/{:02}/mix/{:02}/level", i, bus),
                Command::new_params(
                    DeserializationConfig::new("f"),
                    Box::new(move |state: &mut MixerState, msg| {
                        handle_params_f32(state, msg, |s, v| {
                            s.auxin[auxin_index].mix.sends[bus_index].level = v;
                        })
                    }),
                    Box::new(move |state: &MixerState, _| {
                        Ok(vec![format!(
                            "f:{}",
                            state.auxin[auxin_index].mix.sends[bus_index].level
                        )
                        .into()])
                    }),
                ),
            );
        }
    }

    for i in 1..=16 {
        let bus_index = (i - 1) as usize;

        // Config
        command_map.insert(
            format!("/bus/{:02}/config/name", i),
            Command::new_params(
                DeserializationConfig::new("s"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_str(state, msg, |s, v| {
                        s.bus[bus_index].config.name = v.to_string();
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![format!(
                        "s:{}",
                        state.bus[bus_index].config.name.clone()
                    )
                    .into()])
                }),
            ),
        );
        command_map.insert(
            format!("/bus/{:02}/config/icon", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.bus[bus_index].config.icon = v as u32;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("i:{}", state.bus[bus_index].config.icon).into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/bus/{:02}/config/color", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.bus[bus_index].config.color = v as u32;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("i:{}", state.bus[bus_index].config.color).into(),
                    ])
                }),
            ),
        );

        // Dyn
        command_map.insert(
            format!("/bus/{:02}/dyn/on", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.bus[bus_index].dyn.on = v != 0;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!(
                            "i:{}",
                            if state.bus[bus_index].dyn.on {
                                1
                            } else {
                                0
                            }
                        )
                        .into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/bus/{:02}/dyn/mode", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.bus[bus_index].dyn.mode = v as u32;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![format!("i:{}", state.bus[bus_index].dyn.mode).into()])
                }),
            ),
        );
        command_map.insert(
            format!("/bus/{:02}/dyn/det", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.bus[bus_index].dyn.det = v as u32;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![format!("i:{}", state.bus[bus_index].dyn.det).into()])
                }),
            ),
        );
        command_map.insert(
            format!("/bus/{:02}/dyn/env", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.bus[bus_index].dyn.env = v as u32;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![format!("i:{}", state.bus[bus_index].dyn.env).into()])
                }),
            ),
        );
        command_map.insert(
            format!("/bus/{:02}/dyn/thr", i),
            Command::new_params(
                DeserializationConfig::new("f"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_f32(state, msg, |s, v| {
                        s.bus[bus_index].dyn.thr = v;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![format!("f:{}", state.bus[bus_index].dyn.thr).into()])
                }),
            ),
        );
        command_map.insert(
            format!("/bus/{:02}/dyn/ratio", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.bus[bus_index].dyn.ratio = v as u32;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("i:{}", state.bus[bus_index].dyn.ratio).into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/bus/{:02}/dyn/knee", i),
            Command::new_params(
                DeserializationConfig::new("f"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_f32(state, msg, |s, v| {
                        s.bus[bus_index].dyn.knee = v;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![format!("f:{}", state.bus[bus_index].dyn.knee).into()])
                }),
            ),
        );
        command_map.insert(
            format!("/bus/{:02}/dyn/mgain", i),
            Command::new_params(
                DeserializationConfig::new("f"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_f32(state, msg, |s, v| {
                        s.bus[bus_index].dyn.mgain = v;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("f:{}", state.bus[bus_index].dyn.mgain).into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/bus/{:02}/dyn/attack", i),
            Command::new_params(
                DeserializationConfig::new("f"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_f32(state, msg, |s, v| {
                        s.bus[bus_index].dyn.attack = v;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("f:{}", state.bus[bus_index].dyn.attack).into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/bus/{:02}/dyn/hold", i),
            Command::new_params(
                DeserializationConfig::new("f"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_f32(state, msg, |s, v| {
                        s.bus[bus_index].dyn.hold = v;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![format!("f:{}", state.bus[bus_index].dyn.hold).into()])
                }),
            ),
        );
        command_map.insert(
            format!("/bus/{:02}/dyn/release", i),
            Command::new_params(
                DeserializationConfig::new("f"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_f32(state, msg, |s, v| {
                        s.bus[bus_index].dyn.release = v;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("f:{}", state.bus[bus_index].dyn.release).into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/bus/{:02}/dyn/pos", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.bus[bus_index].dyn.pos = v as u32;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![format!("i:{}", state.bus[bus_index].dyn.pos).into()])
                }),
            ),
        );
        command_map.insert(
            format!("/bus/{:02}/dyn/keysrc", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.bus[bus_index].dyn.keysrc = v as u32;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("i:{}", state.bus[bus_index].dyn.keysrc).into(),
                    ])
                }),
            ),
        );

        // Insert
        command_map.insert(
            format!("/bus/{:02}/insert/on", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.bus[bus_index].insert.on = v != 0;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!(
                            "i:{}",
                            if state.bus[bus_index].insert.on {
                                1
                            } else {
                                0
                            }
                        )
                        .into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/bus/{:02}/insert/pos", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.bus[bus_index].insert.pos = v as u32;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("i:{}", state.bus[bus_index].insert.pos).into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/bus/{:02}/insert/sel", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.bus[bus_index].insert.sel = v as u32;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("i:{}", state.bus[bus_index].insert.sel).into(),
                    ])
                }),
            ),
        );

        // EQ
        command_map.insert(
            format!("/bus/{:02}/eq/on", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.bus[bus_index].eq.on = v != 0;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!(
                            "i:{}",
                            if state.bus[bus_index].eq.on { 1 } else { 0 }
                        )
                        .into(),
                    ])
                }),
            ),
        );
        for band in 1..=6 {
            let band_index = (band - 1) as usize;
            command_map.insert(
                format!("/bus/{:02}/eq/{}/type", i, band),
                Command::new_params(
                    DeserializationConfig::new("i"),
                    Box::new(move |state: &mut MixerState, msg| {
                        handle_params_i32(state, msg, |s, v| {
                            s.bus[bus_index].eq.bands[band_index].eq_type = v as u32;
                        })
                    }),
                    Box::new(move |state: &MixerState, _| {
                        Ok(vec![format!(
                            "i:{}",
                            state.bus[bus_index].eq.bands[band_index].eq_type
                        )
                        .into()])
                    }),
                ),
            );
            command_map.insert(
                format!("/bus/{:02}/eq/{}/f", i, band),
                Command::new_params(
                    DeserializationConfig::new("f"),
                    Box::new(move |state: &mut MixerState, msg| {
                        handle_params_f32(state, msg, |s, v| {
                            s.bus[bus_index].eq.bands[band_index].f = v;
                        })
                    }),
                    Box::new(move |state: &MixerState, _| {
                        Ok(vec![format!(
                            "f:{}",
                            state.bus[bus_index].eq.bands[band_index].f
                        )
                        .into()])
                    }),
                ),
            );
            command_map.insert(
                format!("/bus/{:02}/eq/{}/g", i, band),
                Command::new_params(
                    DeserializationConfig::new("f"),
                    Box::new(move |state: &mut MixerState, msg| {
                        handle_params_f32(state, msg, |s, v| {
                            s.bus[bus_index].eq.bands[band_index].g = v;
                        })
                    }),
                    Box::new(move |state: &MixerState, _| {
                        Ok(vec![format!(
                            "f:{}",
                            state.bus[bus_index].eq.bands[band_index].g
                        )
                        .into()])
                    }),
                ),
            );
            command_map.insert(
                format!("/bus/{:02}/eq/{}/q", i, band),
                Command::new_params(
                    DeserializationConfig::new("f"),
                    Box::new(move |state: &mut MixerState, msg| {
                        handle_params_f32(state, msg, |s, v| {
                            s.bus[bus_index].eq.bands[band_index].q = v;
                        })
                    }),
                    Box::new(move |state: &MixerState, _| {
                        Ok(vec![format!(
                            "f:{}",
                            state.bus[bus_index].eq.bands[band_index].q
                        )
                        .into()])
                    }),
                ),
            );
        }

        // Mix
        command_map.insert(
            format!("/bus/{:02}/mix/on", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.bus[bus_index].mix.on = v != 0;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!(
                            "i:{}",
                            if state.bus[bus_index].mix.on { 1 } else { 0 }
                        )
                        .into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/bus/{:02}/mix/fader", i),
            Command::new_params(
                DeserializationConfig::new("f"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_f32(state, msg, |s, v| {
                        s.bus[bus_index].mix.fader = v;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("f:{}", state.bus[bus_index].mix.fader).into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/bus/{:02}/mix/st", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.bus[bus_index].mix.st = v != 0;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!(
                            "i:{}",
                            if state.bus[bus_index].mix.st { 1 } else { 0 }
                        )
                        .into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/bus/{:02}/mix/pan", i),
            Command::new_params(
                DeserializationConfig::new("f"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_f32(state, msg, |s, v| {
                        s.bus[bus_index].mix.pan = v;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![format!("f:{}", state.bus[bus_index].mix.pan).into()])
                }),
            ),
        );
        command_map.insert(
            format!("/bus/{:02}/mix/mono", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.bus[bus_index].mix.mono = v != 0;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!(
                            "i:{}",
                            if state.bus[bus_index].mix.mono {
                                1
                            } else {
                                0
                            }
                        )
                        .into(),
                    ])
                }),
            ),
        );
        command_map.insert(
            format!("/bus/{:02}/mix/mlevel", i),
            Command::new_params(
                DeserializationConfig::new("f"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_f32(state, msg, |s, v| {
                        s.bus[bus_index].mix.mlevel = v;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("f:{}", state.bus[bus_index].mix.mlevel).into(),
                    ])
                }),
            ),
        );
        for mtx in 1..=6 {
            let mtx_index = (mtx - 1) as usize;
            command_map.insert(
                format!("/bus/{:02}/mix/{:02}/on", i, mtx),
                Command::new_params(
                    DeserializationConfig::new("i"),
                    Box::new(move |state: &mut MixerState, msg| {
                        handle_params_i32(state, msg, |s, v| {
                            s.bus[bus_index].mix.sends[mtx_index].on = v != 0;
                        })
                    }),
                    Box::new(move |state: &MixerState, _| {
                        Ok(vec![
                            format!(
                                "i:{}",
                                if state.bus[bus_index].mix.sends[mtx_index].on {
                                    1
                                } else {
                                    0
                                }
                            )
                            .into(),
                        ])
                    }),
                ),
            );
            command_map.insert(
                format!("/bus/{:02}/mix/{:02}/level", i, mtx),
                Command::new_params(
                    DeserializationConfig::new("f"),
                    Box::new(move |state: &mut MixerState, msg| {
                        handle_params_f32(state, msg, |s, v| {
                            s.bus[bus_index].mix.sends[mtx_index].level = v;
                        })
                    }),
                    Box::new(move |state: &MixerState, _| {
                        Ok(vec![format!(
                            "f:{}",
                            state.bus[bus_index].mix.sends[mtx_index].level
                        )
                        .into()])
                    }),
                ),
            );
        }

        // Group
        command_map.insert(
            format!("/bus/{:02}/grp/dca", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.bus[bus_index].grp.dca = v as u32;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![format!("i:{}", state.bus[bus_index].grp.dca).into()])
                }),
            ),
        );
        command_map.insert(
            format!("/bus/{:02}/grp/mute", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.bus[bus_index].grp.mute = v as u32;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![format!("i:{}", state.bus[bus_index].grp.mute).into()])
                }),
            ),
        );
    }

    add_config_commands(&mut command_map);
    add_main_commands(&mut command_map);
    command_map
}

fn add_main_commands(command_map: &mut HashMap<String, Command>) {
    // Main Stereo
    command_map.insert(
        "/main/st/config/name".to_string(),
        Command::new_params(
            DeserializationConfig::new("s"),
            Box::new(move |state: &mut MixerState, msg| {
                handle_params_str(state, msg, |s, v| {
                    s.main.config.name = v.to_string();
                })
            }),
            Box::new(move |state: &MixerState, _| {
                Ok(vec![format!("s:{}", state.main.config.name.clone()).into()])
            }),
        ),
    );
}

fn add_config_commands(command_map: &mut HashMap<String, Command>) {
    // Channel Link
    for i in 0..16 {
        let pair = i + 1;
        let index = i as usize;
        command_map.insert(
            format!("/config/chlink/{}-{}", pair * 2 - 1, pair * 2),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.link_config.ch[index] = v != 0;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!(
                            "i:{}",
                            if state.link_config.ch[index] { 1 } else { 0 }
                        )
                        .into(),
                    ])
                }),
            ),
        );
    }

    // Aux Link
    for i in 0..4 {
        let pair = i + 1;
        let index = i as usize;
        command_map.insert(
            format!("/config/auxlink/{}-{}", pair * 2 - 1, pair * 2),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.link_config.aux[index] = v != 0;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!(
                            "i:{}",
                            if state.link_config.aux[index] { 1 } else { 0 }
                        )
                        .into(),
                    ])
                }),
            ),
        );
    }

    // FX Link
    for i in 0..4 {
        let pair = i + 1;
        let index = i as usize;
        command_map.insert(
            format!("/config/fxlink/{}-{}", pair * 2 - 1, pair * 2),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.link_config.fx[index] = v != 0;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!(
                            "i:{}",
                            if state.link_config.fx[index] { 1 } else { 0 }
                        )
                        .into(),
                    ])
                }),
            ),
        );
    }

    // Bus Link
    for i in 0..8 {
        let pair = i + 1;
        let index = i as usize;
        command_map.insert(
            format!("/config/buslink/{}-{}", pair * 2 - 1, pair * 2),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.link_config.bus[index] = v != 0;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!(
                            "i:{}",
                            if state.link_config.bus[index] { 1 } else { 0 }
                        )
                        .into(),
                    ])
                }),
            ),
        );
    }

    // Matrix Link
    for i in 0..3 {
        let pair = i + 1;
        let index = i as usize;
        command_map.insert(
            format!("/config/mtxlink/{}-{}", pair * 2 - 1, pair * 2),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.link_config.mtx[index] = v != 0;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!(
                            "i:{}",
                            if state.link_config.mtx[index] { 1 } else { 0 }
                        )
                        .into(),
                    ])
                }),
            ),
        );
    }

    // Mute Groups
    for i in 1..=6 {
        let index = (i - 1) as usize;
        command_map.insert(
            format!("/config/mute/{}", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.mute_group.on[index] = v != 0;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!(
                            "i:{}",
                            if state.mute_group.on[index] { 1 } else { 0 }
                        )
                        .into(),
                    ])
                }),
            ),
        );
    }

    // Link Config
    command_map.insert(
        "/config/linkcfg/hadly".to_string(),
        Command::new_params(
            DeserializationConfig::new("i"),
            Box::new(move |state: &mut MixerState, msg| {
                handle_params_i32(state, msg, |s, v| {
                    s.link_config.hadly = v != 0;
                })
            }),
            Box::new(move |state: &MixerState, _| {
                Ok(vec![
                    format!(
                        "i:{}",
                        if state.link_config.hadly { 1 } else { 0 }
                    )
                    .into(),
                ])
            }),
        ),
    );
    command_map.insert(
        "/config/linkcfg/eq".to_string(),
        Command::new_params(
            DeserializationConfig::new("i"),
            Box::new(move |state: &mut MixerState, msg| {
                handle_params_i32(state, msg, |s, v| {
                    s.link_config.eq = v != 0;
                })
            }),
            Box::new(move |state: &MixerState, _| {
                Ok(vec![
                    format!(
                        "i:{}",
                        if state.link_config.eq { 1 } else { 0 }
                    )
                    .into(),
                ])
            }),
        ),
    );
    command_map.insert(
        "/config/linkcfg/dyn".to_string(),
        Command::new_params(
            DeserializationConfig::new("i"),
            Box::new(move |state: &mut MixerState, msg| {
                handle_params_i32(state, msg, |s, v| {
                    s.link_config.dyn = v != 0;
                })
            }),
            Box::new(move |state: &MixerState, _| {
                Ok(vec![
                    format!(
                        "i:{}",
                        if state.link_config.dyn { 1 } else { 0 }
                    )
                    .into(),
                ])
            }),
        ),
    );
    command_map.insert(
        "/config/linkcfg/fdrmute".to_string(),
        Command::new_params(
            DeserializationConfig::new("i"),
            Box::new(move |state: &mut MixerState, msg| {
                handle_params_i32(state, msg, |s, v| {
                    s.link_config.fdrmute = v != 0;
                })
            }),
            Box::new(move |state: &MixerState, _| {
                Ok(vec![
                    format!(
                        "i:{}",
                        if state.link_config.fdrmute { 1 } else { 0 }
                    )
                    .into(),
                ])
            }),
        ),
    );

    // Mono Bus
    command_map.insert(
        "/config/mono/mode".to_string(),
        Command::new_params(
            DeserializationConfig::new("i"),
            Box::new(move |state: &mut MixerState, msg| {
                handle_params_i32(state, msg, |s, v| {
                    s.mono.config.source = v;
                })
            }),
            Box::new(move |state: &MixerState, _| {
                Ok(vec![format!("i:{}", state.mono.config.source).into()])
            }),
        ),
    );
    command_map.insert(
        "/config/mono/link".to_string(),
        Command::new_params(
            DeserializationConfig::new("i"),
            Box::new(move |state: &mut MixerState, msg| {
                handle_params_i32(state, msg, |s, v| {
                    s.mono.config.color = v;
                })
            }),
            Box::new(move |state: &MixerState, _| {
                Ok(vec![format!("i:{}", state.mono.config.color).into()])
            }),
        ),
    );

    // Solo
    command_map.insert(
        "/config/solo/level".to_string(),
        Command::new_params(
            DeserializationConfig::new("f"),
            Box::new(move |state: &mut MixerState, msg| {
                handle_params_f32(state, msg, |s, v| {
                    s.solo_config.level = v;
                })
            }),
            Box::new(move |state: &MixerState, _| {
                Ok(vec![format!("f:{}", state.solo_config.level).into()])
            }),
        ),
    );

    // Talkback
    command_map.insert(
        "/config/talk/enable".to_string(),
        Command::new_params(
            DeserializationConfig::new("i"),
            Box::new(move |state: &mut MixerState, msg| {
                handle_params_i32(state, msg, |s, v| {
                    s.talkback_config.enable = v != 0;
                })
            }),
            Box::new(move |state: &MixerState, _| {
                Ok(vec![
                    format!(
                        "i:{}",
                        if state.talkback_config.enable { 1 } else { 0 }
                    )
                    .into(),
                ])
            }),
        ),
    );
    command_map.insert(
        "/config/talk/source".to_string(),
        Command::new_params(
            DeserializationConfig::new("i"),
            Box::new(move |state: &mut MixerState, msg| {
                handle_params_i32(state, msg, |s, v| {
                    s.talkback_config.source = v;
                })
            }),
            Box::new(move |state: &MixerState, _| {
                Ok(vec![
                    format!("i:{}", state.talkback_config.source).into(),
                ])
            }),
        ),
    );

    // Talkback A
    command_map.insert(
        "/config/talk/A/level".to_string(),
        Command::new_params(
            DeserializationConfig::new("f"),
            Box::new(move |state: &mut MixerState, msg| {
                handle_params_f32(state, msg, |s, v| {
                    s.talkback_config.a.level = v;
                })
            }),
            Box::new(move |state: &MixerState, _| {
                Ok(vec![format!("f:{}", state.talkback_config.a.level).into()])
            }),
        ),
    );
    command_map.insert(
        "/config/talk/A/dim".to_string(),
        Command::new_params(
            DeserializationConfig::new("i"),
            Box::new(move |state: &mut MixerState, msg| {
                handle_params_i32(state, msg, |s, v| {
                    s.talkback_config.a.dim = v != 0;
                })
            }),
            Box::new(move |state: &MixerState, _| {
                Ok(vec![
                    format!(
                        "i:{}",
                        if state.talkback_config.a.dim { 1 } else { 0 }
                    )
                    .into(),
                ])
            }),
        ),
    );
    command_map.insert(
        "/config/talk/A/latch".to_string(),
        Command::new_params(
            DeserializationConfig::new("i"),
            Box::new(move |state: &mut MixerState, msg| {
                handle_params_i32(state, msg, |s, v| {
                    s.talkback_config.a.latch = v != 0;
                })
            }),
            Box::new(move |state: &MixerState, _| {
                Ok(vec![
                    format!(
                        "i:{}",
                        if state.talkback_config.a.latch { 1 } else { 0 }
                    )
                    .into(),
                ])
            }),
        ),
    );
    command_map.insert(
        "/config/talk/A/destmap".to_string(),
        Command::new_params(
            DeserializationConfig::new("i"),
            Box::new(move |state: &mut MixerState, msg| {
                handle_params_i32(state, msg, |s, v| {
                    s.talkback_config.a.destmap = v;
                })
            }),
            Box::new(move |state: &MixerState, _| {
                Ok(vec![
                    format!("i:{}", state.talkback_config.a.destmap).into(),
                ])
            }),
        ),
    );

    // Talkback B
    command_map.insert(
        "/config/talk/B/level".to_string(),
        Command::new_params(
            DeserializationConfig::new("f"),
            Box::new(move |state: &mut MixerState, msg| {
                handle_params_f32(state, msg, |s, v| {
                    s.talkback_config.b.level = v;
                })
            }),
            Box::new(move |state: &MixerState, _| {
                Ok(vec![format!("f:{}", state.talkback_config.b.level).into()])
            }),
        ),
    );
    command_map.insert(
        "/config/talk/B/dim".to_string(),
        Command::new_params(
            DeserializationConfig::new("i"),
            Box::new(move |state: &mut MixerState, msg| {
                handle_params_i32(state, msg, |s, v| {
                    s.talkback_config.b.dim = v != 0;
                })
            }),
            Box::new(move |state: &MixerState, _| {
                Ok(vec![
                    format!(
                        "i:{}",
                        if state.talkback_config.b.dim { 1 } else { 0 }
                    )
                    .into(),
                ])
            }),
        ),
    );
    command_map.insert(
        "/config/talk/B/latch".to_string(),
        Command::new_params(
            DeserializationConfig::new("i"),
            Box::new(move |state: &mut MixerState, msg| {
                handle_params_i32(state, msg, |s, v| {
                    s.talkback_config.b.latch = v != 0;
                })
            }),
            Box::new(move |state: &MixerState, _| {
                Ok(vec![
                    format!(
                        "i:{}",
                        if state.talkback_config.b.latch { 1 } else { 0 }
                    )
                    .into(),
                ])
            }),
        ),
    );
    command_map.insert(
        "/config/talk/B/destmap".to_string(),
        Command::new_params(
            DeserializationConfig::new("i"),
            Box::new(move |state: &mut MixerState, msg| {
                handle_params_i32(state, msg, |s, v| {
                    s.talkback_config.b.destmap = v;
                })
            }),
            Box::new(move |state: &MixerState, _| {
                Ok(vec![
                    format!("i:{}", state.talkback_config.b.destmap).into(),
                ])
            }),
        ),
    );

    // OSC
    command_map.insert(
        "/config/osc/level".to_string(),
        Command::new_params(
            DeserializationConfig::new("f"),
            Box::new(move |state: &mut MixerState, msg| {
                handle_params_f32(state, msg, |s, v| {
                    s.osc_config.level = v;
                })
            }),
            Box::new(move |state: &MixerState, _| {
                Ok(vec![format!("f:{}", state.osc_config.level).into()])
            }),
        ),
    );
    command_map.insert(
        "/config/osc/f1".to_string(),
        Command::new_params(
            DeserializationConfig::new("f"),
            Box::new(move |state: &mut MixerState, msg| {
                handle_params_f32(state, msg, |s, v| {
                    s.osc_config.f1 = v;
                })
            }),
            Box::new(move |state: &MixerState, _| {
                Ok(vec![format!("f:{}", state.osc_config.f1).into()])
            }),
        ),
    );
    command_map.insert(
        "/config/osc/f2".to_string(),
        Command::new_params(
            DeserializationConfig::new("f"),
            Box::new(move |state: &mut MixerState, msg| {
                handle_params_f32(state, msg, |s, v| {
                    s.osc_config.f2 = v;
                })
            }),
            Box::new(move |state: &MixerState, _| {
                Ok(vec![format!("f:{}", state.osc_config.f2).into()])
            }),
        ),
    );
    command_map.insert(
        "/config/osc/fsel".to_string(),
        Command::new_params(
            DeserializationConfig::new("i"),
            Box::new(move |state: &mut MixerState, msg| {
                handle_params_i32(state, msg, |s, v| {
                    s.osc_config.fsel = v;
                })
            }),
            Box::new(move |state: &MixerState, _| {
                Ok(vec![format!("i:{}", state.osc_config.fsel).into()])
            }),
        ),
    );
    command_map.insert(
        "/config/osc/type".to_string(),
        Command::new_params(
            DeserializationConfig::new("i"),
            Box::new(move |state: &mut MixerState, msg| {
                handle_params_i32(state, msg, |s, v| {
                    s.osc_config.osc_type = v;
                })
            }),
            Box::new(move |state: &MixerState, _| {
                Ok(vec![format!("i:{}", state.osc_config.osc_type).into()])
            }),
        ),
    );
    command_map.insert(
        "/config/osc/dest".to_string(),
        Command::new_params(
            DeserializationConfig::new("i"),
            Box::new(move |state: &mut MixerState, msg| {
                handle_params_i32(state, msg, |s, v| {
                    s.osc_config.dest = v;
                })
            }),
            Box::new(move |state: &MixerState, _| {
                Ok(vec![format!("i:{}", state.osc_config.dest).into()])
            }),
        ),
    );

    // User Routing
    for i in 1..=32 {
        let index = (i - 1) as usize;
        command_map.insert(
            format!("/config/userrout/in/{:02}", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.user_routing.input[index] = v;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("i:{}", state.user_routing.input[index]).into(),
                    ])
                }),
            ),
        );
    }
    for i in 1..=48 {
        let index = (i - 1) as usize;
        command_map.insert(
            format!("/config/userrout/out/{:02}", i),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.user_routing.output[index] = v;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![
                        format!("i:{}", state.user_routing.output[index]).into(),
                    ])
                }),
            ),
        );
    }

    // Routing
    command_map.insert(
        "/config/routing/routswitch".to_string(),
        Command::new_params(
            DeserializationConfig::new("i"),
            Box::new(move |state: &mut MixerState, msg| {
                handle_params_i32(state, msg, |s, v| {
                    s.routing.routswitch = v;
                })
            }),
            Box::new(move |state: &MixerState, _| {
                Ok(vec![format!("i:{}", state.routing.routswitch).into()])
            }),
        ),
    );
    let routing_inputs = [
        (1, 8),
        (9, 16),
        (17, 24),
        (25, 32),
        (0, 0), // Aux
    ];
    for (i, (start, end)) in routing_inputs.iter().enumerate() {
        let index = i as usize;
        let path = if *start == 0 {
            "AUX".to_string()
        } else {
            format!("{}-{}", start, end)
        };
        command_map.insert(
            format!("/config/routing/IN/{}", path),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.routing.input[index] = v;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![format!("i:{}", state.routing.input[index]).into()])
                }),
            ),
        );
    }

    macro_rules! add_routing_commands {
        ($name:expr, $field:ident, $num_routes:expr) => {
            for i in 0..$num_routes {
                let index = i as usize;
                let start = i * 8 + 1;
                let end = (i + 1) * 8;
                command_map.insert(
                    format!("/config/routing/{}/{}-{}", $name, start, end),
                    Command::new_params(
                        DeserializationConfig::new("i"),
                        Box::new(move |state: &mut MixerState, msg| {
                            handle_params_i32(state, msg, |s, v| {
                                s.routing.$field[index] = v;
                            })
                        }),
                        Box::new(move |state: &MixerState, _| {
                            Ok(vec![format!("i:{}", state.routing.$field[index]).into()])
                        }),
                    ),
                );
            }
        };
    }

    add_routing_commands!("AES50A", aes50a, 6);
    add_routing_commands!("AES50B", aes50b, 6);
    add_routing_commands!("CARD", card, 4);

    let output_routing = [(1, 4), (9, 12), (5, 8), (13, 16)];
    for (i, (start, end)) in output_routing.iter().enumerate() {
        let index = i as usize;
        command_map.insert(
            format!("/config/routing/OUT/{}-{}", start, end),
            Command::new_params(
                DeserializationConfig::new("i"),
                Box::new(move |state: &mut MixerState, msg| {
                    handle_params_i32(state, msg, |s, v| {
                        s.routing.output[index] = v;
                    })
                }),
                Box::new(move |state: &MixerState, _| {
                    Ok(vec![format!("i:{}", state.routing.output[index]).into()])
                }),
            ),
        );
    }
}
