#[cfg(test)]
mod tests {
    use crate::{Mixer, MixerState};
    use osc_lib::{OscArg, OscMessage};
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    fn test_addr(port: u16) -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port)
    }

    #[test]
    fn test_mixer_state_new() {
        let state = MixerState::new();
        assert!(state.values.is_empty());
    }

    #[test]
    fn test_mixer_state_set_get() {
        let mut state = MixerState::new();
        let path = "/ch/01/mix/fader";
        let arg = OscArg::Float(0.75);

        state.set(path, arg.clone());
        assert_eq!(state.get(path), Some(&arg));
        assert_eq!(state.get("/non/existent"), None);
    }

    #[test]
    fn test_mixer_seed_from_lines() {
        let mut mixer = Mixer::new();
        let lines = vec![
            "/ch/01/mix/fader,f\t0.75",
            "/ch/01/config/name,s\tMyChannel",
            "/ch/01/mix/on,i\t1",
        ];

        mixer.seed_from_lines(lines);

        assert_eq!(
            mixer.state.get("/ch/01/mix/fader"),
            Some(&OscArg::Float(0.75))
        );
        assert_eq!(
            mixer.state.get("/ch/01/config/name"),
            Some(&OscArg::String("MyChannel".to_string()))
        );
        assert_eq!(mixer.state.get("/ch/01/mix/on"), Some(&OscArg::Int(1)));
    }

    #[test]
    fn test_mixer_seed_from_lines_malformed() {
        let mut mixer = Mixer::new();
        let lines = vec![
            "/ch/01/mix/on,i\tnot_an_int",
            "/ch/01/mix/fader,f\tnot_a_float",
            "/ch/01/mix/fader,f\t0.5",
        ];

        // This should not panic
        mixer.seed_from_lines(lines);

        // Malformed lines should be skipped
        assert_eq!(mixer.state.get("/ch/01/mix/on"), None);
        // The valid line should be processed
        assert_eq!(
            mixer.state.get("/ch/01/mix/fader"),
            Some(&OscArg::Float(0.5))
        );
    }

    #[test]
    fn test_mixer_dispatch_info() {
        let mut mixer = Mixer::new();
        let msg = OscMessage {
            path: "/info".to_string(),
            args: vec![],
        };
        let bytes = msg.to_bytes().unwrap();

        let responses = mixer.dispatch(&bytes, test_addr(1234)).unwrap();
        assert_eq!(responses.len(), 1);
        let response_msg = OscMessage::from_bytes(&responses[0].1).unwrap();

        assert_eq!(response_msg.path, "/info");
        assert_eq!(response_msg.args.len(), 4);
        assert_eq!(response_msg.args[0], OscArg::String("V2.07".to_string()));
    }

    #[test]
    fn test_mixer_dispatch_status() {
        let mut mixer = Mixer::new();
        let msg = OscMessage {
            path: "/status".to_string(),
            args: vec![],
        };
        let bytes = msg.to_bytes().unwrap();

        let responses = mixer.dispatch(&bytes, test_addr(1234)).unwrap();
        assert_eq!(responses.len(), 1);
        let response_msg = OscMessage::from_bytes(&responses[0].1).unwrap();

        assert_eq!(response_msg.path, "/status");
        assert_eq!(response_msg.args.len(), 3);
        assert_eq!(response_msg.args[0], OscArg::String("active".to_string()));
        assert_eq!(response_msg.args[1], OscArg::String("0.0.0.0".to_string()));
        assert_eq!(
            response_msg.args[2],
            OscArg::String("X32 Emulator".to_string())
        );
    }

    #[test]
    fn test_mixer_dispatch_renew() {
        let mut mixer = Mixer::new();
        let msg = OscMessage {
            path: "/renew".to_string(),
            args: vec![],
        };
        let bytes = msg.to_bytes().unwrap();

        let responses = mixer.dispatch(&bytes, test_addr(1234)).unwrap();
        assert!(responses.is_empty());
    }

    #[test]
    fn test_mixer_dispatch_unsubscribe() {
        let mut mixer = Mixer::new();
        let xremote_msg = OscMessage {
            path: "/xremote".to_string(),
            args: vec![],
        };
        let xremote_bytes = xremote_msg.to_bytes().unwrap();
        mixer.dispatch(&xremote_bytes, test_addr(1234)).unwrap();

        assert_eq!(mixer.clients.len(), 1);
        assert_eq!(mixer.clients[0].0, test_addr(1234));

        let unsubscribe_msg = OscMessage {
            path: "/unsubscribe".to_string(),
            args: vec![],
        };
        let unsubscribe_bytes = unsubscribe_msg.to_bytes().unwrap();

        let responses = mixer.dispatch(&unsubscribe_bytes, test_addr(1234)).unwrap();
        assert!(responses.is_empty());
        assert_eq!(mixer.clients.len(), 0);
    }

    #[test]
    fn test_mixer_dispatch_set_value() {
        let mut mixer = Mixer::new();
        let msg = OscMessage {
            path: "/ch/01/mix/fader".to_string(),
            args: vec![OscArg::Float(0.5)],
        };
        let bytes = msg.to_bytes().unwrap();

        let responses = mixer.dispatch(&bytes, test_addr(1234)).unwrap();
        assert!(responses.is_empty());

        assert_eq!(
            mixer.state.get("/ch/01/mix/fader"),
            Some(&OscArg::Float(0.5))
        );
    }

    #[test]
    fn test_mixer_dispatch_get_value() {
        let mut mixer = Mixer::new();
        mixer.state.set("/ch/01/mix/fader", OscArg::Float(0.8));

        let msg = OscMessage {
            path: "/ch/01/mix/fader".to_string(),
            args: vec![],
        };
        let bytes = msg.to_bytes().unwrap();

        let responses = mixer.dispatch(&bytes, test_addr(1234)).unwrap();
        assert_eq!(responses.len(), 1);
        let response_msg = OscMessage::from_bytes(&responses[0].1).unwrap();

        assert_eq!(response_msg.path, "/ch/01/mix/fader");
        assert_eq!(response_msg.args, vec![OscArg::Float(0.8)]);
    }

    #[test]
    fn test_mixer_dispatch_get_non_existent_value() {
        let mut mixer = Mixer::new();
        let msg = OscMessage {
            path: "/non/existent".to_string(),
            args: vec![],
        };
        let bytes = msg.to_bytes().unwrap();

        let responses = mixer.dispatch(&bytes, test_addr(1234)).unwrap();
        assert!(responses.is_empty());
    }

    #[test]
    fn test_mixer_xremote_registration_and_broadcast() {
        let mut mixer = Mixer::new();

        let msg_xremote = OscMessage::new("/xremote".to_string(), vec![])
            .to_bytes()
            .unwrap();

        let responses = mixer.dispatch(&msg_xremote, test_addr(1111)).unwrap();
        assert!(responses.is_empty());

        assert_eq!(mixer.clients.len(), 1);
        assert_eq!(mixer.clients[0].0, test_addr(1111));

        let msg_set = OscMessage::new("/ch/01/mix/fader".to_string(), vec![OscArg::Float(0.5)])
            .to_bytes()
            .unwrap();
        let responses = mixer.dispatch(&msg_set, test_addr(2222)).unwrap();

        assert_eq!(responses.len(), 1);
        assert_eq!(responses[0].0, test_addr(1111));

        let response_msg = OscMessage::from_bytes(&responses[0].1).unwrap();
        assert_eq!(response_msg.path, "/ch/01/mix/fader");
        assert_eq!(response_msg.args, vec![OscArg::Float(0.5)]);
    }

    #[test]
    fn test_mixer_dispatch_system_admin_commands() {
        let mut mixer = Mixer::new();

        let commands = vec!["/copy", "/add", "/load", "/save", "/delete"];
        let item_type = "libchan".to_string();

        for cmd in commands {
            if cmd == "/copy" || cmd == "/save" {
                continue;
            } // Tested separately
            let msg = OscMessage {
                path: cmd.to_string(),
                args: vec![
                    OscArg::String(item_type.clone()),
                    OscArg::Int(1),
                    OscArg::Int(2),
                    OscArg::Int(3),
                ],
            };
            let bytes = msg.to_bytes().unwrap();

            let responses = mixer.dispatch(&bytes, test_addr(1234)).unwrap();
            assert_eq!(responses.len(), 1, "Failed on command: {}", cmd);

            let response_msg = OscMessage::from_bytes(&responses[0].1).unwrap();
            assert_eq!(response_msg.path, cmd);
            assert_eq!(response_msg.args.len(), 2);
            assert_eq!(response_msg.args[0], OscArg::String(item_type.clone()));
            assert_eq!(response_msg.args[1], OscArg::Int(1));
        }
    }

    #[test]
    fn test_mixer_xremote_max_clients() {
        let mut mixer = Mixer::new();
        let msg_xremote = OscMessage::new("/xremote".to_string(), vec![])
            .to_bytes()
            .unwrap();

        mixer.dispatch(&msg_xremote, test_addr(1111)).unwrap();
        mixer.dispatch(&msg_xremote, test_addr(2222)).unwrap();
        mixer.dispatch(&msg_xremote, test_addr(3333)).unwrap();
        mixer.dispatch(&msg_xremote, test_addr(4444)).unwrap();

        assert_eq!(mixer.clients.len(), 4);

        mixer.dispatch(&msg_xremote, test_addr(5555)).unwrap();
        assert_eq!(mixer.clients.len(), 4); // should still be 4

        let mut addrs: Vec<SocketAddr> = mixer.clients.iter().map(|c| c.0).collect();
        addrs.sort();
        assert_eq!(
            addrs,
            vec![
                test_addr(1111),
                test_addr(2222),
                test_addr(3333),
                test_addr(4444)
            ]
        );
    }

    #[test]
    fn test_mixer_dispatch_copy_libchan() {
        let mut mixer = Mixer::new();

        // Seed source channel (01)
        mixer
            .state
            .set("/ch/01/config/name", OscArg::String("Source".to_string()));
        mixer.state.set("/ch/01/mix/fader", OscArg::Float(0.75));
        // Seed dest channel (02)
        mixer
            .state
            .set("/ch/02/config/name", OscArg::String("Dest".to_string()));
        mixer.state.set("/ch/02/mix/fader", OscArg::Float(0.1));

        // Copy /ch/01 to /ch/02 with a mask that includes everything (-1)
        // format: /copy ,siii "libchan" source dest mask
        let msg = OscMessage {
            path: "/copy".to_string(),
            args: vec![
                OscArg::String("libchan".to_string()),
                OscArg::Int(0),  // Source 01 (0-based)
                OscArg::Int(1),  // Dest 02 (0-based)
                OscArg::Int(-1), // Mask all
            ],
        };
        let bytes = msg.to_bytes().unwrap();

        let responses = mixer.dispatch(&bytes, test_addr(1234)).unwrap();

        // We expect the destination to be updated
        assert_eq!(
            mixer.state.get("/ch/02/config/name"),
            Some(&OscArg::String("Source".to_string()))
        );
        assert_eq!(
            mixer.state.get("/ch/02/mix/fader"),
            Some(&OscArg::Float(0.75))
        );

        // We expect a response acknowledging the copy
        assert_eq!(responses.len(), 1);
        let response_msg = OscMessage::from_bytes(&responses[0].1).unwrap();
        assert_eq!(response_msg.path, "/copy");
        assert_eq!(response_msg.args.len(), 2);
        assert_eq!(response_msg.args[0], OscArg::String("libchan".to_string()));
        assert_eq!(response_msg.args[1], OscArg::Int(1));
    }

    #[test]
    fn test_mixer_dispatch_save_scene() {
        let mut mixer = Mixer::new();

        // Save scene format: /save ,siss "scene" idx name note
        let msg = OscMessage {
            path: "/save".to_string(),
            args: vec![
                OscArg::String("scene".to_string()),
                OscArg::Int(5), // Scene idx 5
                OscArg::String("My Scene".to_string()),
                OscArg::String("My Note".to_string()),
            ],
        };
        let bytes = msg.to_bytes().unwrap();

        let responses = mixer.dispatch(&bytes, test_addr(1234)).unwrap();

        // We expect the state to have "My Scene" at /-show/showfile/scene/005/name
        assert_eq!(
            mixer.state.get("/-show/showfile/scene/005/name"),
            Some(&OscArg::String("My Scene".to_string()))
        );
        // The original C code puts the note at the next index, but doesn't explicitly mention the path for note in save.
        // I will assume standard format /note for it. Let's just check name for now to see if basic implementation works.

        // We expect a response acknowledging the save
        assert_eq!(responses.len(), 1);
        let response_msg = OscMessage::from_bytes(&responses[0].1).unwrap();
        assert_eq!(response_msg.path, "/save");
        assert_eq!(response_msg.args.len(), 2);
        assert_eq!(response_msg.args[0], OscArg::String("scene".to_string()));
        assert_eq!(response_msg.args[1], OscArg::Int(1)); // Success
    }

    #[test]
    fn test_mixer_dispatch_save_libchan() {
        let mut mixer = Mixer::new();

        // Save libchan format: /save ,sis "libchan" idx name
        let msg = OscMessage {
            path: "/save".to_string(),
            args: vec![
                OscArg::String("libchan".to_string()),
                OscArg::Int(10), // Libchan idx 10
                OscArg::String("My Channel Preset".to_string()),
            ],
        };
        let bytes = msg.to_bytes().unwrap();

        let responses = mixer.dispatch(&bytes, test_addr(1234)).unwrap();

        assert_eq!(
            mixer.state.get("/-libs/ch/010/name"),
            Some(&OscArg::String("My Channel Preset".to_string()))
        );
        assert_eq!(
            mixer.state.get("/-libs/ch/010/hasdata"),
            Some(&OscArg::Int(1))
        );

        assert_eq!(responses.len(), 1);
        let response_msg = OscMessage::from_bytes(&responses[0].1).unwrap();
        assert_eq!(response_msg.path, "/save");
        assert_eq!(response_msg.args.len(), 2);
        assert_eq!(response_msg.args[0], OscArg::String("libchan".to_string()));
        assert_eq!(response_msg.args[1], OscArg::Int(1)); // Success
    }

    #[test]
    fn test_mixer_dispatch_save_libfx() {
        let mut mixer = Mixer::new();

        // Save libfx format: /save ,sis "libfx" idx name
        let msg = OscMessage {
            path: "/save".to_string(),
            args: vec![
                OscArg::String("libfx".to_string()),
                OscArg::Int(15), // Libfx idx 15
                OscArg::String("My FX Preset".to_string()),
            ],
        };
        let bytes = msg.to_bytes().unwrap();

        let responses = mixer.dispatch(&bytes, test_addr(1234)).unwrap();

        assert_eq!(
            mixer.state.get("/-libs/fx/015/name"),
            Some(&OscArg::String("My FX Preset".to_string()))
        );
        assert_eq!(
            mixer.state.get("/-libs/fx/015/hasdata"),
            Some(&OscArg::Int(1))
        );

        assert_eq!(responses.len(), 1);
        let response_msg = OscMessage::from_bytes(&responses[0].1).unwrap();
        assert_eq!(response_msg.path, "/save");
        assert_eq!(response_msg.args.len(), 2);
        assert_eq!(response_msg.args[0], OscArg::String("libfx".to_string()));
        assert_eq!(response_msg.args[1], OscArg::Int(1)); // Success
    }

    #[test]
    fn test_mixer_dispatch_save_librout() {
        let mut mixer = Mixer::new();

        // Save librout format: /save ,sis "librout" idx name
        let msg = OscMessage {
            path: "/save".to_string(),
            args: vec![
                OscArg::String("librout".to_string()),
                OscArg::Int(5), // Librout idx 5
                OscArg::String("My Routing Preset".to_string()),
            ],
        };
        let bytes = msg.to_bytes().unwrap();

        let responses = mixer.dispatch(&bytes, test_addr(1234)).unwrap();

        assert_eq!(
            mixer.state.get("/-libs/r/005/name"),
            Some(&OscArg::String("My Routing Preset".to_string()))
        );
        assert_eq!(
            mixer.state.get("/-libs/r/005/hasdata"),
            Some(&OscArg::Int(1))
        );

        assert_eq!(responses.len(), 1);
        let response_msg = OscMessage::from_bytes(&responses[0].1).unwrap();
        assert_eq!(response_msg.path, "/save");
        assert_eq!(response_msg.args.len(), 2);
        assert_eq!(response_msg.args[0], OscArg::String("librout".to_string()));
        assert_eq!(response_msg.args[1], OscArg::Int(1)); // Success
    }

    #[test]
    fn test_mixer_dispatch_save_snippet() {
        let mut mixer = Mixer::new();

        // Save snippet format: /save ,siss "snippet" idx name note
        let msg = OscMessage {
            path: "/save".to_string(),
            args: vec![
                OscArg::String("snippet".to_string()),
                OscArg::Int(2), // Snippet idx 2
                OscArg::String("My Snippet".to_string()),
                OscArg::String("My Note".to_string()),
            ],
        };
        let bytes = msg.to_bytes().unwrap();

        let responses = mixer.dispatch(&bytes, test_addr(1234)).unwrap();

        assert_eq!(
            mixer.state.get("/-show/showfile/snippet/002/name"),
            Some(&OscArg::String("My Snippet".to_string()))
        );

        assert_eq!(responses.len(), 1);
        let response_msg = OscMessage::from_bytes(&responses[0].1).unwrap();
        assert_eq!(response_msg.path, "/save");
        assert_eq!(response_msg.args.len(), 2);
        assert_eq!(response_msg.args[0], OscArg::String("snippet".to_string()));
        assert_eq!(response_msg.args[1], OscArg::Int(1)); // Success
    }

    #[test]
    fn test_mixer_dispatch_meters() {
        let mut mixer = Mixer::new();

        // Subscribe to /meters/1
        let msg = OscMessage {
            path: "/meters/1".to_string(),
            args: vec![],
        };
        let bytes = msg.to_bytes().unwrap();

        // Dispatch should process the subscription but might not return an immediate response depending on how tick() works
        let _ = mixer.dispatch(&bytes, test_addr(1234)).unwrap();

        // Now tick the mixer
        let responses = mixer.tick();

        // We expect one meter response blob
        assert_eq!(responses.len(), 1);
        let (addr, resp_bytes) = &responses[0];
        assert_eq!(*addr, test_addr(1234));

        // Check the blob structure. We won't deserialize full OSC here, just basic properties
        let msg_out = OscMessage::from_bytes(resp_bytes).unwrap();
        assert_eq!(msg_out.path, "/meters/1");
        // /meters/1 args should be a blob
        assert_eq!(msg_out.args.len(), 1);
        if let OscArg::Blob(blob) = &msg_out.args[0] {
            // /meters/1 expects 96 floats (96 * 4 = 384 bytes, plus the length header in standard OSC which we don't count here, but the data length)
            assert_eq!(blob.len(), 384);
        } else {
            panic!("Expected blob argument");
        }
    }
}
