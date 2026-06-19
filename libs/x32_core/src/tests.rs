#[cfg(test)]
#[allow(clippy::module_inception)]
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
        assert!(responses.len() >= 1);
        let response_msg = OscMessage::from_bytes(&responses.last().unwrap().1).unwrap();

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
        assert!(responses.len() >= 1);
        let response_msg = OscMessage::from_bytes(&responses.last().unwrap().1).unwrap();

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
        assert!(responses.len() >= 1);
        let response_msg = OscMessage::from_bytes(&responses.last().unwrap().1).unwrap();

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

        assert!(responses.len() >= 1);
        assert_eq!(responses[0].0, test_addr(1111));

        let response_msg = OscMessage::from_bytes(&responses.last().unwrap().1).unwrap();
        assert_eq!(response_msg.path, "/ch/01/mix/fader");
        assert_eq!(response_msg.args, vec![OscArg::Float(0.5)]);
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
        assert!(responses.len() >= 1);
        let response_msg = OscMessage::from_bytes(&responses.last().unwrap().1).unwrap();
        assert_eq!(response_msg.path, "/copy");
        assert_eq!(response_msg.args.len(), 2);
        assert_eq!(response_msg.args[0], OscArg::String("libchan".to_string()));
        assert_eq!(response_msg.args[1], OscArg::Int(1));
    }

    #[test]
    fn test_mixer_dispatch_copy_libfx() {
        let mut mixer = Mixer::new();

        mixer
            .state
            .set("/-libs/fx/001/name", OscArg::String("SourceFX".to_string()));
        mixer.state.set("/-libs/fx/001/hasdata", OscArg::Int(1));

        mixer
            .state
            .set("/-libs/fx/002/name", OscArg::String("DestFX".to_string()));

        let msg = OscMessage {
            path: "/copy".to_string(),
            args: vec![
                OscArg::String("libfx".to_string()),
                OscArg::Int(1),
                OscArg::Int(2),
                OscArg::Int(-1),
            ],
        };
        let bytes = msg.to_bytes().unwrap();
        let responses = mixer.dispatch(&bytes, test_addr(1234)).unwrap();

        assert_eq!(
            mixer.state.get("/-libs/fx/002/name"),
            Some(&OscArg::String("SourceFX".to_string()))
        );
        assert_eq!(
            mixer.state.get("/-libs/fx/002/hasdata"),
            Some(&OscArg::Int(1))
        );

        assert!(responses.len() >= 1);
        let response_msg = OscMessage::from_bytes(&responses.last().unwrap().1).unwrap();
        assert_eq!(response_msg.path, "/copy");
        assert_eq!(response_msg.args.len(), 2);
        assert_eq!(response_msg.args[0], OscArg::String("libfx".to_string()));
        assert_eq!(response_msg.args[1], OscArg::Int(1));
    }

    #[test]
    fn test_mixer_dispatch_copy_librout() {
        let mut mixer = Mixer::new();

        mixer.state.set(
            "/-libs/r/005/name",
            OscArg::String("SourceRout".to_string()),
        );
        mixer.state.set("/-libs/r/005/hasdata", OscArg::Int(1));

        mixer
            .state
            .set("/-libs/r/010/name", OscArg::String("DestRout".to_string()));

        let msg = OscMessage {
            path: "/copy".to_string(),
            args: vec![
                OscArg::String("librout".to_string()),
                OscArg::Int(5),
                OscArg::Int(10),
                OscArg::Int(-1),
            ],
        };
        let bytes = msg.to_bytes().unwrap();
        let responses = mixer.dispatch(&bytes, test_addr(1234)).unwrap();

        assert_eq!(
            mixer.state.get("/-libs/r/010/name"),
            Some(&OscArg::String("SourceRout".to_string()))
        );
        assert_eq!(
            mixer.state.get("/-libs/r/010/hasdata"),
            Some(&OscArg::Int(1))
        );

        assert!(responses.len() >= 1);
        let response_msg = OscMessage::from_bytes(&responses.last().unwrap().1).unwrap();
        assert_eq!(response_msg.path, "/copy");
        assert_eq!(response_msg.args.len(), 2);
        assert_eq!(response_msg.args[0], OscArg::String("librout".to_string()));
        assert_eq!(response_msg.args[1], OscArg::Int(1));
    }

    #[test]
    fn test_mixer_dispatch_copy_scene() {
        let mut mixer = Mixer::new();

        mixer.state.set(
            "/-show/showfile/scene/015/name",
            OscArg::String("SourceScene".to_string()),
        );
        mixer.state.set(
            "/-show/showfile/scene/015/note",
            OscArg::String("SourceNote".to_string()),
        );

        mixer.state.set(
            "/-show/showfile/scene/020/name",
            OscArg::String("DestScene".to_string()),
        );

        let msg = OscMessage {
            path: "/copy".to_string(),
            args: vec![
                OscArg::String("scene".to_string()),
                OscArg::Int(15),
                OscArg::Int(20),
                OscArg::Int(-1),
            ],
        };
        let bytes = msg.to_bytes().unwrap();
        let responses = mixer.dispatch(&bytes, test_addr(1234)).unwrap();

        assert_eq!(
            mixer.state.get("/-show/showfile/scene/020/name"),
            Some(&OscArg::String("SourceScene".to_string()))
        );
        assert_eq!(
            mixer.state.get("/-show/showfile/scene/020/note"),
            Some(&OscArg::String("SourceNote".to_string()))
        );

        assert!(responses.len() >= 1);
        let response_msg = OscMessage::from_bytes(&responses.last().unwrap().1).unwrap();
        assert_eq!(response_msg.path, "/copy");
        assert_eq!(response_msg.args.len(), 2);
        assert_eq!(response_msg.args[0], OscArg::String("scene".to_string()));
        assert_eq!(response_msg.args[1], OscArg::Int(1));
    }

    #[test]
    fn test_mixer_dispatch_add() {
        let mut mixer = Mixer::new();

        let msg = OscMessage {
            path: "/add".to_string(),
            args: vec![
                OscArg::String("cue".to_string()),
                OscArg::Int(5),
                OscArg::String("My Cue".to_string()),
            ],
        };
        let bytes = msg.to_bytes().unwrap();
        let responses = mixer.dispatch(&bytes, test_addr(1234)).unwrap();

        assert_eq!(
            mixer.state.get("/-show/showfile/cue/005/name"),
            Some(&OscArg::String("My Cue".to_string()))
        );
        assert_eq!(
            mixer.state.get("/-show/showfile/cue/005/hasdata"),
            Some(&OscArg::Int(1))
        );

        assert!(responses.len() >= 1);
        let response_msg = OscMessage::from_bytes(&responses.last().unwrap().1).unwrap();
        assert_eq!(response_msg.path, "/add");
        assert_eq!(response_msg.args.len(), 2);
        assert_eq!(response_msg.args[0], OscArg::String("cue".to_string()));
        assert_eq!(response_msg.args[1], OscArg::Int(1));
    }

    #[test]
    fn test_mixer_dispatch_load() {
        let mut mixer = Mixer::new();

        mixer.state.set(
            "/-show/showfile/scene/001/name",
            OscArg::String("Preset Scene".to_string()),
        );
        mixer
            .state
            .set("/-show/showfile/scene/001/hasdata", OscArg::Int(1));
        mixer.state.set(
            "/-show/showfile/scene/001/ch/01/mix/fader",
            OscArg::Float(0.5),
        );
        mixer.state.set(
            "/-show/showfile/scene/001/ch/02/mix/fader",
            OscArg::Float(0.75),
        );

        let msg = OscMessage {
            path: "/load".to_string(),
            args: vec![OscArg::String("scene".to_string()), OscArg::Int(1)],
        };
        let bytes = msg.to_bytes().unwrap();
        let responses = mixer.dispatch(&bytes, test_addr(1234)).unwrap();

        // Check that state was copied (with "/" prefix replacing the scene prefix)
        assert_eq!(
            mixer.state.get("/ch/01/mix/fader"),
            Some(&OscArg::Float(0.5))
        );
        assert_eq!(
            mixer.state.get("/ch/02/mix/fader"),
            Some(&OscArg::Float(0.75))
        );

        // Name and hasdata should not be copied to root
        assert_eq!(mixer.state.get("/name"), None);
        assert_eq!(mixer.state.get("/hasdata"), None);

        assert!(responses.len() >= 1);
        let response_msg = OscMessage::from_bytes(&responses.last().unwrap().1).unwrap();
        assert_eq!(response_msg.path, "/load");
        assert_eq!(response_msg.args.len(), 2);
        assert_eq!(response_msg.args[0], OscArg::String("scene".to_string()));
        assert_eq!(response_msg.args[1], OscArg::Int(1));
    }

    #[test]
    fn test_mixer_dispatch_delete_scene_snippet() {
        let mut mixer = Mixer::new();

        // Setup some initial state
        mixer.state.set(
            "/-show/showfile/scene/005/name",
            OscArg::String("Old Scene".to_string()),
        );
        mixer.state.set(
            "/-show/showfile/scene/005/note",
            OscArg::String("Old Note".to_string()),
        );
        mixer.state.set(
            "/-show/showfile/snippet/010/name",
            OscArg::String("Old Snippet".to_string()),
        );
        mixer.state.set(
            "/-show/showfile/snippet/010/note",
            OscArg::String("Old Snippet Note".to_string()),
        );

        // Test delete scene
        let msg = OscMessage {
            path: "/delete".to_string(),
            args: vec![OscArg::String("scene".to_string()), OscArg::Int(5)],
        };
        let bytes = msg.to_bytes().unwrap();
        let responses = mixer.dispatch(&bytes, test_addr(1234)).unwrap();

        // Check state is cleared
        assert_eq!(
            mixer.state.get("/-show/showfile/scene/005/name"),
            Some(&OscArg::String("".to_string()))
        );
        assert_eq!(
            mixer.state.get("/-show/showfile/scene/005/note"),
            Some(&OscArg::String("".to_string()))
        );
        assert_eq!(
            mixer.state.get("/-show/showfile/scene/005/hasdata"),
            Some(&OscArg::Int(0))
        );

        // Check response (delete command status)
        let mut found_response = false;
        for (_, resp_bytes) in &responses {
            let resp = OscMessage::from_bytes(resp_bytes).unwrap();
            if resp.path == "/delete" {
                assert_eq!(resp.args[0], OscArg::String("scene".to_string()));
                assert_eq!(resp.args[1], OscArg::Int(1));
                found_response = true;
            }
        }
        assert!(found_response);

        // Test delete snippet
        let msg2 = OscMessage {
            path: "/delete".to_string(),
            args: vec![OscArg::String("snippet".to_string()), OscArg::Int(10)],
        };
        let bytes2 = msg2.to_bytes().unwrap();
        let responses2 = mixer.dispatch(&bytes2, test_addr(1234)).unwrap();

        // Check state is cleared
        assert_eq!(
            mixer.state.get("/-show/showfile/snippet/010/name"),
            Some(&OscArg::String("".to_string()))
        );
        assert_eq!(
            mixer.state.get("/-show/showfile/snippet/010/note"),
            Some(&OscArg::String("".to_string()))
        );
        assert_eq!(
            mixer.state.get("/-show/showfile/snippet/010/hasdata"),
            Some(&OscArg::Int(0))
        );

        // Check response
        let mut found_response2 = false;
        for (_, resp_bytes) in &responses2 {
            let resp = OscMessage::from_bytes(resp_bytes).unwrap();
            if resp.path == "/delete" {
                assert_eq!(resp.args[0], OscArg::String("snippet".to_string()));
                assert_eq!(resp.args[1], OscArg::Int(1));
                found_response2 = true;
            }
        }
        assert!(found_response2);
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
        assert_eq!(
            mixer.state.get("/-show/showfile/scene/005/note"),
            Some(&OscArg::String("My Note".to_string()))
        );
        assert_eq!(
            mixer.state.get("/-show/showfile/scene/005/hasdata"),
            Some(&OscArg::Int(1))
        );
        // The original C code puts the note at the next index, but doesn't explicitly mention the path for note in save.
        // I will assume standard format /note for it. Let's just check name for now to see if basic implementation works.

        // We expect a response acknowledging the save
        assert!(responses.len() >= 1);
        let response_msg = OscMessage::from_bytes(&responses.last().unwrap().1).unwrap();
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

        assert!(responses.len() >= 1);
        let response_msg = OscMessage::from_bytes(&responses.last().unwrap().1).unwrap();
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

        assert!(responses.len() >= 1);
        let response_msg = OscMessage::from_bytes(&responses.last().unwrap().1).unwrap();
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

        assert!(responses.len() >= 1);
        let response_msg = OscMessage::from_bytes(&responses.last().unwrap().1).unwrap();
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
        assert_eq!(
            mixer.state.get("/-show/showfile/snippet/002/note"),
            Some(&OscArg::String("My Note".to_string()))
        );
        assert_eq!(
            mixer.state.get("/-show/showfile/snippet/002/hasdata"),
            Some(&OscArg::Int(1))
        );

        assert!(responses.len() >= 1);
        let response_msg = OscMessage::from_bytes(&responses.last().unwrap().1).unwrap();
        assert_eq!(response_msg.path, "/save");
        assert_eq!(response_msg.args.len(), 2);
        assert_eq!(response_msg.args[0], OscArg::String("snippet".to_string()));
        assert_eq!(response_msg.args[1], OscArg::Int(1)); // Success
    }

    #[test]
    fn test_mixer_dispatch_node() {
        let mut mixer = Mixer::new();

        mixer.state.set("/ch/01/config/color", OscArg::Int(3));
        mixer
            .state
            .set("/ch/01/config/name", OscArg::String("MyName".to_string()));

        let msg = OscMessage {
            path: "/node".to_string(),
            args: vec![OscArg::String("ch/01/config".to_string())],
        };
        let bytes = msg.to_bytes().unwrap();
        let responses = mixer.dispatch(&bytes, test_addr(1234)).unwrap();

        assert!(responses.len() >= 1);
        let response_msg = OscMessage::from_bytes(&responses.last().unwrap().1).unwrap();

        assert_eq!(response_msg.path, "node");
        assert_eq!(response_msg.args.len(), 1);
        assert_eq!(
            response_msg.args[0],
            OscArg::String("ch/01/config 3 \"MyName\"".to_string())
        );
    }

    #[test]
    fn test_mixer_solosw_updates_solo() {
        let mut mixer = Mixer::new();

        // Connect a client to receive broadcast messages
        let msg_xremote = OscMessage::new("/xremote".to_string(), vec![])
            .to_bytes()
            .unwrap();
        mixer.dispatch(&msg_xremote, test_addr(1234)).unwrap();

        // Set solosw 01 to 1
        let msg1 = OscMessage {
            path: "/-stat/solosw/01".to_string(),
            args: vec![OscArg::Int(1)],
        };
        let bytes1 = msg1.to_bytes().unwrap();
        let responses1 = mixer.dispatch(&bytes1, test_addr(1234)).unwrap();

        assert_eq!(mixer.state.get("/-stat/solo"), Some(&OscArg::Int(1)));

        let mut found_solo = false;
        for (_, resp_bytes) in &responses1 {
            let resp = OscMessage::from_bytes(resp_bytes).unwrap();
            if resp.path == "/-stat/solo" {
                assert_eq!(resp.args[0], OscArg::Int(1));
                found_solo = true;
            }
        }
        assert!(found_solo);

        // Set solosw 02 to 1
        let msg2 = OscMessage {
            path: "/-stat/solosw/02".to_string(),
            args: vec![OscArg::Int(1)],
        };
        let bytes2 = msg2.to_bytes().unwrap();
        mixer.dispatch(&bytes2, test_addr(1234)).unwrap();

        assert_eq!(mixer.state.get("/-stat/solo"), Some(&OscArg::Int(1)));

        // Set solosw 01 to 0
        let msg3 = OscMessage {
            path: "/-stat/solosw/01".to_string(),
            args: vec![OscArg::Int(0)],
        };
        let bytes3 = msg3.to_bytes().unwrap();
        mixer.dispatch(&bytes3, test_addr(1234)).unwrap();

        assert_eq!(mixer.state.get("/-stat/solo"), Some(&OscArg::Int(1))); // 02 is still on

        // Set solosw 02 to 0
        let msg4 = OscMessage {
            path: "/-stat/solosw/02".to_string(),
            args: vec![OscArg::Int(0)],
        };
        let bytes4 = msg4.to_bytes().unwrap();
        let responses4 = mixer.dispatch(&bytes4, test_addr(1234)).unwrap();

        assert_eq!(mixer.state.get("/-stat/solo"), Some(&OscArg::Int(0)));

        let mut found_solo_off = false;
        for (_, resp_bytes) in &responses4 {
            let resp = OscMessage::from_bytes(resp_bytes).unwrap();
            if resp.path == "/-stat/solo" {
                assert_eq!(resp.args[0], OscArg::Int(0));
                found_solo_off = true;
            }
        }
        assert!(found_solo_off);
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
        assert!(responses.len() >= 1);
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
