
use super::*;
use osc_lib::{OscArg, OscMessage};
use crate::config::{Config, Range};
use crate::state::AppState;

#[test]
fn test_x32_fader_message() {
    let config = Config::default();
    let mut app_state = AppState::new(&config);
    let msg = OscMessage::new("/ch/01/mix/fader".to_string(), vec![OscArg::Float(0.5)]);
    let bytes = msg.to_bytes().unwrap();

    let result = handle_x32_message(&bytes, &config, &mut app_state).unwrap();
    assert_eq!(result.len(), 1);
    let reaper_msg = &result[0];
    assert_eq!(reaper_msg.path, "/track/1/volume");
    if let Some(OscArg::Float(level)) = reaper_msg.args.get(0) {
        assert_eq!(*level, 0.5);
    } else {
        panic!("Expected float argument");
    }
}

#[test]
fn test_x32_transport_play_message() {
    let mut config = Config::default();
    config.transport_on = true;
    let mut app_state = AppState::new(&config);
    let msg = OscMessage::new("/-stat/userpar/18/value".to_string(), vec![OscArg::Int(0)]);
    let bytes = msg.to_bytes().unwrap();

    let result = handle_x32_message(&bytes, &config, &mut app_state).unwrap();
    assert_eq!(result.len(), 1);
    let reaper_msg = &result[0];
    assert_eq!(reaper_msg.path, "/play");
}

#[test]
fn test_channel_bank_up() {
    let mut config = Config::default();
    config.channel_bank_on = true;
    config.bank_up_button = 9;
    config.bank_size = 8;
    let mut app_state = AppState::new(&config);
    app_state.track_states[8].fader = 0.75;

    let msg = OscMessage::new(format!("/-stat/userpar/{}/value", config.bank_up_button + 12), vec![OscArg::Int(0)]);
    let bytes = msg.to_bytes().unwrap();

    let result = handle_x32_message(&bytes, &config, &mut app_state).unwrap();
    println!("{:?}", result);
    let expected_level = (0.75_f32 * 1023.5).round() / 1023.0;
    assert!(result.iter().any(|m| m.path == "/ch/01/mix/fader" && m.args == vec![OscArg::Float(expected_level)]));
}

#[test]
fn test_dca_fader_message() {
    let mut config = Config::default();
    config.reaper_dca_map[0] = Range { min: 1, max: 2 };
    let mut app_state = AppState::new(&config);
    let msg = OscMessage::new("/dca/1/fader".to_string(), vec![OscArg::Float(0.75)]);
    let bytes = msg.to_bytes().unwrap();
    let result = handle_x32_message(&bytes, &config, &mut app_state).unwrap();
    assert_eq!(result.len(), 3);
    assert!(result.iter().any(|m| m.path == "/track/1/volume" && m.args == vec![OscArg::Float(0.75)]));
    assert!(result.iter().any(|m| m.path == "/track/2/volume" && m.args == vec![OscArg::Float(0.75)]));
}

#[test]
fn test_marker_button_message() {
    let mut config = Config::default();
    config.marker_button_on = true;
    config.marker_button = 8;
    let mut app_state = AppState::new(&config);
    let msg = OscMessage::new(format!("/-stat/userpar/{}/value", config.marker_button + 12), vec![OscArg::Int(0)]);
    let bytes = msg.to_bytes().unwrap();

    let result = handle_x32_message(&bytes, &config, &mut app_state).unwrap();
    assert_eq!(result.len(), 1);
    let reaper_msg = &result[0];
    assert_eq!(reaper_msg.path, "/action/40157");
}

#[test]
fn test_x32_eq_message() {
    let config = Config::default();
    let mut app_state = AppState::new(&config);
    let msg = OscMessage::new("/ch/01/eq/1/f".to_string(), vec![OscArg::Float(0.5)]);
    let bytes = msg.to_bytes().unwrap();

    let result = handle_x32_message(&bytes, &config, &mut app_state).unwrap();
    assert_eq!(result.len(), 1);
    let reaper_msg = &result[0];
    assert_eq!(reaper_msg.path, "/track/1/fx/fxparam/1/value");
    if let Some(OscArg::Float(level)) = reaper_msg.args.get(0) {
        assert_eq!(*level, 0.5);
    } else {
        panic!("Expected float argument");
    }
}

#[test]
fn test_x32_send_message() {
    let config = Config::default();
    let mut app_state = AppState::new(&config);
    let msg = OscMessage::new("/ch/01/mix/01/level".to_string(), vec![OscArg::Float(0.5)]);
    let bytes = msg.to_bytes().unwrap();

    let result = handle_x32_message(&bytes, &config, &mut app_state).unwrap();
    assert_eq!(result.len(), 1);
    let reaper_msg = &result[0];
    assert_eq!(reaper_msg.path, "/track/1/send/1/volume");
    if let Some(OscArg::Float(level)) = reaper_msg.args.get(0) {
        assert_eq!(*level, 0.5);
    } else {
        panic!("Expected float argument");
    }
}

#[test]
fn test_x32_solo_message() {
    let config = Config::default();
    let mut app_state = AppState::new(&config);
    let msg = OscMessage::new("/-stat/solosw/1".to_string(), vec![OscArg::Int(1)]);
    let bytes = msg.to_bytes().unwrap();

    let result = handle_x32_message(&bytes, &config, &mut app_state).unwrap();
    assert_eq!(result.len(), 1);
    let reaper_msg = &result[0];
    assert_eq!(reaper_msg.path, "/track/1/solo");
    if let Some(OscArg::Float(level)) = reaper_msg.args.get(0) {
        assert_eq!(*level, 1.0);
    } else {
        panic!("Expected float argument");
    }
}

#[test]
fn test_x32_encoder_next_beat() {
    let mut config = Config::default();
    config.transport_on = true;
    let mut app_state = AppState::new(&config);
    let msg = OscMessage::new("/-stat/userpar/33/value".to_string(), vec![OscArg::Int(65)]);
    let bytes = msg.to_bytes().unwrap();

    let result = handle_x32_message(&bytes, &config, &mut app_state).unwrap();
    assert_eq!(result.len(), 1);
    let reaper_msg = &result[0];
    assert_eq!(reaper_msg.path, "/action/40841");
}
