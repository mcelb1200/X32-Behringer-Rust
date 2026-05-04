use crate::*;
use tokio::time::Duration;
use osc_lib::OscArg;
use std::thread;

#[tokio::test]
async fn test_async_methods() {
    // Start the emulator in a separate thread
    thread::spawn(|| {
        x32_emulator::server::run("127.0.0.1:10025", None, None).unwrap();
    });

    // Give emulator a moment to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = MixerClient::connect("127.0.0.1:10025", false).await.unwrap();

    // Test query_value_async
    let arg = query_value_async(&client, "/info").await.unwrap();
    assert!(matches!(arg, OscArg::String(_)));

    // Test set and get parameter async
    let test_addr = "/ch/01/mix/fader";
    set_parameter_async(&client, test_addr, 0.75).await.unwrap();

    // Give emulator a moment to process
    tokio::time::sleep(Duration::from_millis(50)).await;

    let value = get_parameter_async(&client, test_addr).await.unwrap();
    assert_eq!(value, 0.75);
}
