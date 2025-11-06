use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Result};

pub mod state;
pub mod commands;
pub mod logic_fns;
pub mod command_map;
pub mod node_tree;
pub mod remote_clients;
pub mod state_management;
pub mod status;

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Result};

use osc_lib::{OscArg, OscMessage};
use state::MixerState;
use crate::command_map::build_command_map;
use crate::commands::Command;
use crate::status::{handle_info, handle_status};
use crate::node_tree::handle_node_get;
use crate::remote_clients::handle_subscribe;
use crate::state_management::{handle_load, handle_save, handle_delete, handle_copy};

pub type CommandHandler = fn(&mut Mixer, &OscMessage, SocketAddr) -> Result<Vec<(Vec<u8>, SocketAddr)>>;

pub struct Mixer {
    pub state: MixerState,
    ip_address: String,
    command_map: HashMap<String, Command>,
    clients: HashMap<SocketAddr, Instant>,
    meter_subscriptions: Vec<MeterSubscription>,
}

pub struct MeterSubscription {
    pub meter_group: i32,
    pub client_addr: SocketAddr,
    pub interval: Duration,
    pub next_send_time: Instant,
    pub expiry_time: Instant,
}

impl Mixer {
    pub fn new(ip_address: String) -> Self {
        let state = Mixer::load().unwrap_or_default();
        let command_map = build_command_map();

        Self {
            state,
            command_map,
            ip_address,
            clients: HashMap::new(),
            meter_subscriptions: Vec::new(),
        }
    }

    pub fn process_subscriptions(&mut self) -> Vec<(Vec<u8>, SocketAddr)> {
        let now = Instant::now();
        self.meter_subscriptions.retain(|sub| sub.expiry_time > now);

        let mut responses = Vec::new();
        for sub in &mut self.meter_subscriptions {
            if sub.next_send_time <= now {
                // For now, just send a dummy blob of zeros
                let blob_data = vec![0u8; 296];
                let response = OscMessage {
                    path: format!("/meters/{}", sub.meter_group),
                    args: vec![OscArg::Blob(blob_data)],
                };
                responses.push((response.to_bytes().unwrap(), sub.client_addr));
                sub.next_send_time = now + sub.interval;
            }
        }
        responses
    }

    pub fn dispatch(&mut self, msg: &[u8], remote_addr: SocketAddr) -> Result<Vec<(Vec<u8>, SocketAddr)>> {
        let osc_msg = match OscMessage::from_str(std::str::from_utf8(msg)?) {
            Ok(m) => m,
            Err(e) => return Err(anyhow!("OSC parsing error: {}", e)),
        };

        if let Some(command) = self.command_map.get_mut(&osc_msg.path) {
            match command {
                Command::Special { handler } => {
                    let result = handler(&mut self.state, &osc_msg, remote_addr)?;
                    return Ok(result.into_iter().map(|m| (m.to_string().into(), remote_addr)).collect());
                }
                Command::Params { set_handler, get_handler, .. } => {
                    let mut responses = Vec::new();
                    if osc_msg.args.is_empty() {
                        // GET
                        let result = get_handler(&self.state, &osc_msg)?;
                        for r in result {
                            responses.push((r.to_string().into_bytes(), remote_addr));
                        }
                    } else {
                        // SET
                        if let Some(propagate_msgs) = set_handler(&mut self.state, &osc_msg)? {
                            for client_addr in self.clients.keys() {
                                for msg in &propagate_msgs {
                                    responses.push((msg.to_string().into_bytes(), *client_addr));
                                }
                            }
                        } else {
                            for client_addr in self.clients.keys() {
                                responses.push((osc_msg.to_string().into_bytes(), *client_addr));
                            }
                        }
                    }
                    return Ok(responses);
                }
            }
        }

        println!("No handler for path: {}", &osc_msg.path);
        Ok(vec![])
    }

    pub fn load() -> Result<MixerState> {
        let file = File::open(".X32res.rc");
        let mut state: MixerState = match file {
            Ok(file) => {
                let reader = BufReader::new(file);
                serde_json::from_reader(reader).unwrap_or_default()
            }
            Err(_) => MixerState::default(),
        };

        if state.preferences.name.is_empty() {
            state.preferences.name = "X32 Emulator".to_string();
        }
        if state.channel_presets.is_empty() {
            state.channel_presets.resize_with(100, Default::default);
        }

        Ok(state)
    }

    pub fn save(&self) -> Result<()> {
        let file = File::create(".X32res.rc")?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, &self.state)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dispatch() {
        let mut mixer = Mixer::new("127.0.0.1".to_string());
        let remote_addr: SocketAddr = "127.0.0.1:12345".parse().unwrap();

        let info_msg = OscMessage { path: "/info".to_string(), args: vec![] };
        let status_msg = OscMessage { path: "/status".to_string(), args: vec![] };
        let unknown_msg = OscMessage { path: "/xxxx".to_string(), args: vec![] };

        assert!(!mixer.dispatch(&info_msg.to_string().into_bytes(), remote_addr).unwrap().is_empty());
        assert!(!mixer.dispatch(&status_msg.to_string().into_bytes(), remote_addr).unwrap().is_empty());
        assert!(mixer.dispatch(&unknown_msg.to_string().into_bytes(), remote_addr).unwrap().is_empty());
    }

    #[test]
    fn test_status_response() {
        let mut mixer = Mixer::new("192.168.1.2".to_string());
        mixer.state.preferences.name = "MyX32".to_string();
        let remote_addr: SocketAddr = "127.0.0.1:12345".parse().unwrap();

        let status_msg = OscMessage { path: "/status".to_string(), args: vec![] };
        let responses = mixer.dispatch(&status_msg.to_string().into_bytes(), remote_addr).unwrap();
        let (response_buf, response_addr) = &responses[0];
        let response_msg = OscMessage::from_str(std::str::from_utf8(response_buf).unwrap()).unwrap();

        assert_eq!(response_addr, &remote_addr);
        assert_eq!(response_msg.path, "/status");
        assert_eq!(response_msg.args, vec![
            OscArg::String("active".to_string()),
            OscArg::String("192.168.1.2".to_string()),
            OscArg::String("MyX32".to_string()),
        ]);
    }

    #[test]
    fn test_get_set_params() {
        let mut mixer = Mixer::new("127.0.0.1".to_string());
        let remote_addr: SocketAddr = "127.0.0.1:12345".parse().unwrap();

        // Test SET
        let set_msg = OscMessage {
            path: "/ch/01/config/name".to_string(),
            args: vec![OscArg::String("My Channel".to_string())],
        };
        mixer.dispatch(&set_msg.to_string().into_bytes(), remote_addr).unwrap();

        // Test GET
        let get_msg = OscMessage {
            path: "/ch/01/config/name".to_string(),
            args: vec![],
        };
        let responses = mixer.dispatch(&get_msg.to_string().into_bytes(), remote_addr).unwrap();
        let (response_buf, _) = &responses[0];
        let response_msg = OscMessage::from_str(std::str::from_utf8(response_buf).unwrap()).unwrap();

        assert_eq!(response_msg.path, "/ch/01/config/name");
        assert_eq!(response_msg.args, vec![OscArg::String("My Channel".to_string())]);
    }

    #[test]
    fn test_slash_command() {
        let mut mixer = Mixer::new("127.0.0.1".to_string());
        let remote_addr: SocketAddr = "127.0.0.1:12345".parse().unwrap();

        let slash_msg = OscMessage {
            path: "/".to_string(),
            args: vec![OscArg::String("/ch/01/config/name NewName\n/ch/02/config/name Chan2".to_string())]
        };
        mixer.dispatch(&slash_msg.to_string().into_bytes(), remote_addr).unwrap();

        assert_eq!(mixer.state.channels[0].config.name, "NewName");
        assert_eq!(mixer.state.channels[1].config.name, "Chan2");
    }

    #[test]
    fn test_stub_handlers() {
        let mut mixer = Mixer::new("127.0.0.1".to_string());
        let remote_addr: SocketAddr = "127.0.0.1:12345".parse().unwrap();

        let paths = vec![
            "/-prefs", "/-stat", "/-urec",
            "/-action", "/copy", "/load", "/save", "/delete"
        ];

        for path in paths {
            let msg = OscMessage { path: path.to_string(), args: vec![] };
            assert!(mixer.dispatch(&msg.to_string().into_bytes(), remote_addr).unwrap().is_empty());
        }
    }

    #[test]
    fn test_xremote() {
        let mut mixer = Mixer::new("127.0.0.1".to_string());
        let remote_addr: SocketAddr = "127.0.0.1:12345".parse().unwrap();

        let xremote_msg = OscMessage { path: "/xremote".to_string(), args: vec![] };
        mixer.dispatch(&xremote_msg.to_string().into_bytes(), remote_addr).unwrap();

        assert!(mixer.clients.contains_key(&remote_addr));

        // Test that a set command propagates to the subscribed client
        let remote_addr2: SocketAddr = "127.0.0.1:54321".parse().unwrap();
        let set_msg = OscMessage {
            path: "/ch/01/config/name".to_string(),
            args: vec![OscArg::String("New Name".to_string())],
        };
        let responses = mixer.dispatch(&set_msg.to_string().into_bytes(), remote_addr2).unwrap();

        assert_eq!(responses.len(), 2);
        assert_eq!(responses[0].1, remote_addr2);
        assert_eq!(responses[1].1, remote_addr);
    }

    #[test]
    fn test_meters() {
        let mut mixer = Mixer::new("127.0.0.1".to_string());
        let remote_addr: SocketAddr = "127.0.0.1:12345".parse().unwrap();

        let meters_msg = OscMessage {
            path: "/meters".to_string(),
            args: vec![
                OscArg::String("/meters/1".to_string()),
                OscArg::Int(1),
            ],
        };
        mixer.dispatch(&meters_msg.to_string().into_bytes(), remote_addr).unwrap();

        assert_eq!(mixer.meter_subscriptions.len(), 1);

        let responses = mixer.process_subscriptions();
        assert_eq!(responses.len(), 1);
        let (response_buf, response_addr) = &responses[0];
        let response_msg = OscMessage::from_bytes(response_buf).unwrap();

        assert_eq!(response_addr, &remote_addr);
        assert_eq!(response_msg.path, "/meters/1");
        assert!(matches!(response_msg.args[0], OscArg::Blob(_)));
    }

    #[test]
    fn test_copy() {
        let mut mixer = Mixer::new("127.0.0.1".to_string());
        let remote_addr: SocketAddr = "127.0.0.1:12345".parse().unwrap();

        mixer.state.channels[0].config.name = "Source".to_string();
        mixer.state.channels[1].config.name = "Destination".to_string();

        let copy_msg = OscMessage {
            path: "/copy".to_string(),
            args: vec![
                OscArg::String("libchan".to_string()),
                OscArg::Int(0),
                OscArg::Int(1),
                OscArg::Int(C_CONFIG),
            ],
        };

        let responses = mixer.dispatch(&copy_msg.to_string().into_bytes(), remote_addr).unwrap();
        assert_eq!(responses.len(), 1);
        let (response_buf, _) = &responses[0];
        let response_msg = OscMessage::from_str(std::str::from_utf8(response_buf).unwrap()).unwrap();

        assert_eq!(response_msg.path, "/copy");
        assert_eq!(response_msg.args, vec![
            OscArg::String("libchan".to_string()),
            OscArg::Int(1),
        ]);

        assert_eq!(mixer.state.channels[1].config.name, "Source");
    }

    #[test]
    fn test_save_load_delete() {
        let mut mixer = Mixer::new("127.0.0.1".to_string());
        let remote_addr: SocketAddr = "127.0.0.1:12345".parse().unwrap();

        mixer.state.channels[5].config.name = "Preset Channel".to_string();

        // Save
        let save_msg = OscMessage {
            path: "/save".to_string(),
            args: vec![
                OscArg::String("libchan".to_string()),
                OscArg::Int(5),
                OscArg::String("My Preset".to_string()),
            ],
        };
        let responses = mixer.dispatch(&save_msg.to_string().into_bytes(), remote_addr).unwrap();
        let (response_buf, _) = &responses[0];
        let response_msg = OscMessage::from_str(std::str::from_utf8(response_buf).unwrap()).unwrap();
        assert_eq!(response_msg.args[1], OscArg::Int(1));
        assert_eq!(mixer.state.channel_presets[5].name, "My Preset");

        // Change channel name
        mixer.state.channels[5].config.name = "New Name".to_string();

        // Load
        let load_msg = OscMessage {
            path: "/load".to_string(),
            args: vec![
                OscArg::String("libchan".to_string()),
                OscArg::Int(5),
            ],
        };
        let responses = mixer.dispatch(&load_msg.to_string().into_bytes(), remote_addr).unwrap();
        let (response_buf, _) = &responses[0];
        let response_msg = OscMessage::from_str(std::str::from_utf8(response_buf).unwrap()).unwrap();
        assert_eq!(response_msg.args[1], OscArg::Int(1));
        assert_eq!(mixer.state.channels[5].config.name, "Preset Channel");

        // Delete
        let delete_msg = OscMessage {
            path: "/delete".to_string(),
            args: vec![
                OscArg::String("libchan".to_string()),
                OscArg::Int(5),
            ],
        };
        let responses = mixer.dispatch(&delete_msg.to_string().into_bytes(), remote_addr).unwrap();
        let (response_buf, _) = &responses[0];
        let response_msg = OscMessage::from_str(std::str::from_utf8(response_buf).unwrap()).unwrap();
        assert_eq!(response_msg.args[1], OscArg::Int(1));
        assert_eq!(mixer.state.channel_presets[5].name, "");
    }
}
