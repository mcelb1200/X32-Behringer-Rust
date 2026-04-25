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
    fn test_mixer_dispatch_copy_command() {
        let mut mixer = Mixer::new();
        // Set up initial state for source channel 1
        mixer
            .state
            .set("/ch/01/config/name", OscArg::String("Vox 1".to_string()));
        mixer.state.set("/ch/01/preamp/trim", OscArg::Float(0.5));

        // Ensure destination channel 2 is empty initially
        assert_eq!(mixer.state.get("/ch/02/config/name"), None);
        assert_eq!(mixer.state.get("/ch/02/preamp/trim"), None);

        // /copy~~~,siii~~~type source destination mask
        // Mask 0x02 is C_CONFIG
        let msg = OscMessage {
            path: "/copy".to_string(),
            args: vec![
                OscArg::String("libchan".to_string()),
                OscArg::Int(0), // source 0 (ch 01)
                OscArg::Int(1), // dest 1 (ch 02)
                OscArg::Int(2), // mask C_CONFIG
            ],
        };
        let bytes = msg.to_bytes().unwrap();
        let responses = mixer.dispatch(&bytes, test_addr(1234)).unwrap();

        // Check that state was updated appropriately based on mask
        assert_eq!(
            mixer.state.get("/ch/02/config/name"),
            Some(&OscArg::String("Vox 1".to_string()))
        );
        assert_eq!(mixer.state.get("/ch/02/preamp/trim"), None); // HA not copied because mask didn't include it

        // Ensure acknowledgment response sent
        assert_eq!(responses.len(), 1);
        let resp_msg = OscMessage::from_bytes(&responses[0].1).unwrap();
        assert_eq!(resp_msg.path, "/copy");
        assert_eq!(resp_msg.args.len(), 2);
        assert_eq!(resp_msg.args[0], OscArg::String("libchan".to_string()));
        assert_eq!(resp_msg.args[1], OscArg::Int(1));
    }

    #[test]
    fn test_mixer_dispatch_add_command() {
        let mut mixer = Mixer::new();
        let msg = OscMessage {
            path: "/add".to_string(),
            args: vec![OscArg::String("cue".to_string()), OscArg::Int(1)],
        };
        let bytes = msg.to_bytes().unwrap();
        let responses = mixer.dispatch(&bytes, test_addr(1234)).unwrap();

        assert_eq!(responses.len(), 1);
        let resp_msg = OscMessage::from_bytes(&responses[0].1).unwrap();
        assert_eq!(resp_msg.path, "/add");
        assert_eq!(
            resp_msg.args,
            vec![OscArg::String("cue".to_string()), OscArg::Int(1)]
        );
    }

    #[test]
    fn test_mixer_dispatch_load_command() {
        let mut mixer = Mixer::new();
        let msg = OscMessage {
            path: "/load".to_string(),
            args: vec![OscArg::String("libchan".to_string()), OscArg::Int(1)],
        };
        let bytes = msg.to_bytes().unwrap();
        let responses = mixer.dispatch(&bytes, test_addr(1234)).unwrap();

        assert_eq!(responses.len(), 1);
        let resp_msg = OscMessage::from_bytes(&responses[0].1).unwrap();
        assert_eq!(resp_msg.path, "/load");
        assert_eq!(
            resp_msg.args,
            vec![OscArg::String("libchan".to_string()), OscArg::Int(1)]
        );
    }

    #[test]
    fn test_mixer_dispatch_save_command() {
        let mut mixer = Mixer::new();
        let msg = OscMessage {
            path: "/save".to_string(),
            args: vec![
                OscArg::String("scene".to_string()),
                OscArg::Int(5), // scene 5
                OscArg::String("My Scene".to_string()),
                OscArg::String("Note".to_string()),
            ],
        };
        let bytes = msg.to_bytes().unwrap();
        let responses = mixer.dispatch(&bytes, test_addr(1234)).unwrap();

        // C code saves to internal state based on type
        assert_eq!(
            mixer.state.get("/-show/showfile/scene/005/name"),
            Some(&OscArg::String("My Scene".to_string()))
        );

        assert_eq!(responses.len(), 1);
        let resp_msg = OscMessage::from_bytes(&responses[0].1).unwrap();
        assert_eq!(resp_msg.path, "/save");
        assert_eq!(
            resp_msg.args,
            vec![OscArg::String("scene".to_string()), OscArg::Int(1)]
        );
    }
}
