
use x32_lib::command::fx;
use x32_lib::create_socket;
use x32_emulator::server;
use std::thread;

#[test]
fn test_xfx_set_array() {
    // Start the emulator in a separate thread
    thread::spawn(|| {
        server::run("127.0.0.1:10023", None, None).unwrap();
    });

    // Give the server a moment to start
    thread::sleep(std::time::Duration::from_millis(100));

    let socket = create_socket("127.0.0.1", 2000).unwrap();
    let msg = fx::set_fx_param(&socket, 1, 1, 0.5);
    assert!(msg.is_ok());

    let (address, args) = fx::set_fx_param(&socket, 1, 1, 0.5).unwrap();
    assert_eq!(address, "/fx/1/par/01");
    let arg_value = match args[0] {
        osc_lib::OscArg::Float(val) => val,
        _ => panic!("Expected float argument"),
    };

    assert!((arg_value - 0.5).abs() < f32::EPSILON);
}
