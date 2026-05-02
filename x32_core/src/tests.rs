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
}
