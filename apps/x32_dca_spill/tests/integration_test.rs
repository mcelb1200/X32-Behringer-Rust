use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;

use osc_lib::OscArg;
use x32_dca_spill::{Args, run};

#[tokio::test]
async fn test_dca_spill_custom_bank_mapping() {
    let server_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let port = server_socket.local_addr().unwrap().port();
    let ip = format!("127.0.0.1:{}", port);

    let server_socket = Arc::new(server_socket);
    let rx_socket = server_socket.clone();
    let tx_socket = server_socket.clone();

    // Spawn server to handle messages and simulate X32 responses
    let server_handle = tokio::spawn(async move {
        let mut buf = [0; 1024];
        let mut dca_members = HashMap::new();

        // Simulate some DCA members.
        // Let's put channel 1 and 2 in DCA 1
        dca_members.insert("/ch/01/grp/dca".to_string(), 1);
        dca_members.insert("/ch/02/grp/dca".to_string(), 1);
        // Auxin 1 in DCA 1
        dca_members.insert("/auxin/01/grp/dca".to_string(), 1);

        let mut received_bank_assignments = Vec::new();
        let mut client_addr = None;

        // Run until we receive 24 assignments (3 blocks of 8 faders)
        while received_bank_assignments.len() < 24 {
            if let Ok(Ok((size, src_addr))) =
                tokio::time::timeout(Duration::from_millis(50), rx_socket.recv_from(&mut buf)).await
            {
                if client_addr.is_none() {
                    client_addr = Some(src_addr);
                }
                if let Ok(msg) = osc_lib::OscMessage::from_bytes(&buf[..size]) {
                    if msg.path.ends_with("/grp/dca") {
                        let mask = dca_members.get(&msg.path).unwrap_or(&0);
                        let response = osc_lib::OscMessage {
                            path: msg.path.clone(),
                            args: vec![OscArg::Int(*mask)],
                        };
                        let bytes = osc_lib::OscMessage::serialize_to_bytes(
                            &response.path,
                            [&OscArg::Int(*mask)],
                        )
                        .unwrap();
                        tx_socket.send_to(&bytes, src_addr).await.unwrap();
                    } else if msg.path.starts_with("/-prefs/custom_bank/") {
                        if let Some(OscArg::Int(src_id)) = msg.args.first() {
                            received_bank_assignments.push((msg.path.clone(), *src_id));
                        }
                    } else if msg.path == "/xremote" {
                        // send selidx 72 to simulate DCA 1 select
                        let select_msg = osc_lib::OscMessage {
                            path: "/-stat/selidx".to_string(),
                            args: vec![OscArg::Int(72)],
                        };
                        let bytes = osc_lib::OscMessage::serialize_to_bytes(
                            &select_msg.path,
                            [&OscArg::Int(72)],
                        )
                        .unwrap();
                        tx_socket.send_to(&bytes, src_addr).await.unwrap();
                    }
                }
            } else if let Some(addr) = client_addr {
                // If it times out, broadcast the select again
                let select_msg = osc_lib::OscMessage {
                    path: "/-stat/selidx".to_string(),
                    args: vec![OscArg::Int(72)],
                };
                let bytes =
                    osc_lib::OscMessage::serialize_to_bytes(&select_msg.path, [&OscArg::Int(72)])
                        .unwrap();
                tx_socket.send_to(&bytes, addr).await.unwrap();
            }
        }
        received_bank_assignments
    });

    // Run the app in a task
    let args = Args {
        ip: ip.clone(),
        dca: Some(1),
    };

    let app_handle = tokio::spawn(async move {
        let _ = run(args).await;
    });

    let assignments = tokio::time::timeout(Duration::from_secs(2), server_handle)
        .await
        .expect("Timeout")
        .unwrap();

    // Verify correct OSC paths for custom bank (/-prefs/custom_bank/{block}/{fader})
    assert_eq!(assignments[0].0, "/-prefs/custom_bank/1/1");
    assert_eq!(assignments[0].1, 1); // ch 1 (src id 1)

    assert_eq!(assignments[1].0, "/-prefs/custom_bank/1/2");
    assert_eq!(assignments[1].1, 2); // ch 2 (src id 2)

    assert_eq!(assignments[2].0, "/-prefs/custom_bank/1/3");
    assert_eq!(assignments[2].1, 33); // auxin 1 (src id 33)

    assert_eq!(assignments[3].0, "/-prefs/custom_bank/1/4");
    assert_eq!(assignments[3].1, 0); // off

    assert_eq!(assignments[7].0, "/-prefs/custom_bank/1/8");

    assert_eq!(assignments[8].0, "/-prefs/custom_bank/2/1");
    assert_eq!(assignments[15].0, "/-prefs/custom_bank/2/8");

    assert_eq!(assignments[16].0, "/-prefs/custom_bank/3/1");
    assert_eq!(assignments[23].0, "/-prefs/custom_bank/3/8");
}
