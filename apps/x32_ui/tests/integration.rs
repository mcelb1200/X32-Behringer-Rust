//! End-to-end integration tests for the `x32_ui` server and Virtual Soundcheck workflow.

use osc_lib::{OscArg, OscMessage};
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UdpSocket};
use tokio::sync::Mutex;
use x32_core::Mixer;
use x32_ui::server::spawn_server;
use x32_ui::AppContext;

async fn spawn_test_emulator() -> (SocketAddr, Arc<Mutex<Mixer>>) {
    let sock = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let addr = sock.local_addr().unwrap();
    let mut mixer = Mixer::new();

    // Seed sample console settings via dispatch
    let dummy_peer: SocketAddr = "127.0.0.1:40000".parse().unwrap();
    let seeds = [
        ("/ch/01/config/name", OscArg::String("Kick".to_string())),
        ("/ch/01/config/color", OscArg::Int(1)),
        ("/ch/02/config/name", OscArg::String("Snare".to_string())),
        ("/ch/02/config/color", OscArg::Int(3)),
        ("/ch/03/config/name", OscArg::String("Keys L".to_string())),
        ("/ch/04/config/name", OscArg::String("Keys R".to_string())),
        ("/config/chlink/3-4", OscArg::Int(1)),
        ("/mtx/01/config/name", OscArg::String("Lobby".to_string())),
        ("/mtx/01/mix/fader", OscArg::Float(0.5)),
        ("/-prefs/clockrate", OscArg::Int(0)), // 48000 Hz
        ("/config/routing/routswitch", OscArg::Int(0)), // REC
        ("/config/routing/IN/1-8", OscArg::Int(4)), // AES50 A1-8
        ("/config/routing/IN/9-16", OscArg::Int(5)), // AES50 A9-16
        ("/config/routing/IN/17-24", OscArg::Int(2)),
        ("/config/routing/IN/25-32", OscArg::Int(3)),
        ("/config/routing/PLAY/1-8", OscArg::Int(0)),
        ("/config/routing/CARD/1-8", OscArg::Int(0)),
    ];
    for (path, arg) in seeds {
        let bytes = OscMessage::serialize_to_bytes(path, [&arg]).unwrap();
        let _ = mixer.dispatch(&bytes, dummy_peer).unwrap();
    }

    let mixer_arc = Arc::new(Mutex::new(mixer));
    let mixer_bg = mixer_arc.clone();

    tokio::spawn(async move {
        let mut buf = [0u8; 4096];
        loop {
            if let Ok((len, peer)) = sock.recv_from(&mut buf).await {
                let responses = {
                    let mut m = mixer_bg.lock().await;
                    m.dispatch(&buf[..len], peer).unwrap_or_default()
                };
                for (target, pkt) in responses {
                    let _ = sock.send_to(&pkt, target).await;
                }
            }
        }
    });

    (addr, mixer_arc)
}

async fn http_request(
    addr: SocketAddr,
    method: &str,
    path: &str,
    body: Option<&str>,
) -> (u16, String) {
    let mut stream = TcpStream::connect(addr).await.unwrap();
    let payload = body.unwrap_or("");
    let req = format!(
        "{method} {path} HTTP/1.1\r\n\
         Host: {addr}\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\r\n\
         {payload}",
        payload.len()
    );
    stream.write_all(req.as_bytes()).await.unwrap();
    stream.flush().await.unwrap();

    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).await.unwrap();
    let text = String::from_utf8_lossy(&raw).to_string();
    let (head, resp_body) = text.split_once("\r\n\r\n").unwrap_or((&text, ""));
    let status = head
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);
    (status, resp_body.to_string())
}

#[tokio::test]
async fn test_consolidated_ui_and_virtual_soundcheck_e2e() {
    let tmp = TempDir::new().unwrap();
    let (emu_addr, emu_mixer) = spawn_test_emulator().await;
    let ctx = Arc::new(AppContext::with_profile_path(
        tmp.path().join("x32_studio_profile.json"),
    ));
    let (http_addr, server_handle) = spawn_server("127.0.0.1:0".parse().unwrap(), ctx.clone())
        .await
        .unwrap();

    // 1. UI HTML and health check
    let (st, html) = http_request(http_addr, "GET", "/", None).await;
    assert_eq!(st, 200);
    assert!(html.contains("X32 Control Suite"));

    let (st, health) = http_request(http_addr, "GET", "/api/health", None).await;
    assert_eq!(st, 200);
    assert!(health.contains("\"ok\""));

    // 2. Tools catalog returns all 48 tools
    let (st, tools_body) = http_request(http_addr, "GET", "/api/tools", None).await;
    assert_eq!(st, 200);
    let tools_json: serde_json::Value = serde_json::from_str(&tools_body).unwrap();
    assert_eq!(tools_json["tools"].as_array().unwrap().len(), 48);

    // 3. Connect to the test emulator
    let connect_payload = serde_json::json!({
        "ip": emu_addr.to_string(),
        "transport": "osc"
    })
    .to_string();
    let (st, conn_body) = http_request(http_addr, "POST", "/api/connect", Some(&connect_payload)).await;
    assert_eq!(st, 200, "connect failed: {}", conn_body);
    let conn_json: serde_json::Value = serde_json::from_str(&conn_body).unwrap();
    assert_eq!(conn_json["connected"], true);

    // 4. Capture console state for Virtual Soundcheck
    let (st, cap_body) =
        http_request(http_addr, "POST", "/api/vsc/capture", Some(r#"{"refresh":true,"include_aux_card":true}"#)).await;
    assert_eq!(st, 200);
    let cap_json: serde_json::Value = serde_json::from_str(&cap_body).unwrap();
    assert_eq!(cap_json["state"]["channels"][0]["name"], "Kick");
    assert_eq!(cap_json["state"]["channels"][1]["name"], "Snare");
    assert_eq!(cap_json["state"]["channels"][2]["linked_stereo"], true);
    assert_eq!(cap_json["state"]["matrices"][0]["name"], "Lobby");
    assert_eq!(cap_json["routing_plan"]["card_blocks"], serde_json::json!([4, 5, 2, 3]));
    assert_eq!(cap_json["routing_plan"]["play_blocks"], serde_json::json!([16, 17, 18, 19]));

    // 4b. Control Matrix 01 fader & mute via HTTP and verify on emulator
    let (st, _) = http_request(
        http_addr,
        "POST",
        "/api/mixer/fader",
        Some(r#"{"kind":"mtx","index":1,"value":0.65}"#),
    )
    .await;
    assert_eq!(st, 200);
    let (st, _) = http_request(
        http_addr,
        "POST",
        "/api/mixer/mute",
        Some(r#"{"kind":"mtx","index":1,"on":false}"#),
    )
    .await;
    assert_eq!(st, 200);
    tokio::time::sleep(std::time::Duration::from_millis(40)).await;
    {
        let mut m = emu_mixer.lock().await;
        let peer: SocketAddr = "127.0.0.1:40001".parse().unwrap();
        let query = |path: &str| OscMessage::serialize_to_bytes(path, std::iter::empty::<&OscArg>()).unwrap();
        let r_fader = m.dispatch(&query("/mtx/01/mix/fader"), peer).unwrap();
        let r_on = m.dispatch(&query("/mtx/01/mix/on"), peer).unwrap();
        if let OscArg::Float(f) = OscMessage::from_bytes(&r_fader[0].1).unwrap().args[0] {
            assert!((f - 0.65).abs() < 1e-4);
        } else {
            panic!("Expected float fader response for /mtx/01/mix/fader");
        }
        assert_eq!(OscMessage::from_bytes(&r_on[0].1).unwrap().args[0], OscArg::Int(0));
    }

    // 5. Backup & Apply VSC routing + switch to PLAY
    let backup_path = tmp.path().join("pre_vsc.json");
    let apply_payload = serde_json::json!({
        "include_aux_card": true,
        "backup_path": backup_path.to_string_lossy(),
        "switch_to_play": true
    })
    .to_string();
    let (st, apply_body) =
        http_request(http_addr, "POST", "/api/vsc/apply-routing", Some(&apply_payload)).await;
    assert_eq!(st, 200, "apply failed: {}", apply_body);
    assert!(backup_path.exists());

    // Confirm emulator received CARD/1-8 = 4, PLAY/1-8 = 16, and routswitch = 1 (PLAY)
    tokio::time::sleep(std::time::Duration::from_millis(60)).await;
    {
        let mut m = emu_mixer.lock().await;
        let peer: SocketAddr = "127.0.0.1:40001".parse().unwrap();
        let query = |path: &str| OscMessage::serialize_to_bytes(path, std::iter::empty::<&OscArg>()).unwrap();
        let r_card = m.dispatch(&query("/config/routing/CARD/1-8"), peer).unwrap();
        let r_play = m.dispatch(&query("/config/routing/PLAY/1-8"), peer).unwrap();
        let r_sw = m.dispatch(&query("/config/routing/routswitch"), peer).unwrap();
        assert_eq!(OscMessage::from_bytes(&r_card[0].1).unwrap().args[0], OscArg::Int(4));
        assert_eq!(OscMessage::from_bytes(&r_play[0].1).unwrap().args[0], OscArg::Int(16));
        assert_eq!(OscMessage::from_bytes(&r_sw[0].1).unwrap().args[0], OscArg::Int(1));
    }

    // 6. Generate & adapt REAPER .rpp (from docs/X32Template.rpp) + .X32Reaper.ini
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let tpl_path = repo_root.join("docs/X32Template.rpp");
    let out_rpp = tmp.path().join("VSC_Show.rpp");
    let out_ini = tmp.path().join(".X32Reaper.ini");

    let mut full_opts = cap_json["session_options"].clone();
    full_opts["include_empty_channels"] = serde_json::json!(true);
    let gen_payload = serde_json::json!({
        "session_options": full_opts,
        "template_path": tpl_path.to_string_lossy(),
        "output_rpp_path": out_rpp.to_string_lossy(),
        "output_ini_path": out_ini.to_string_lossy()
    })
    .to_string();
    let (st, gen_body) =
        http_request(http_addr, "POST", "/api/vsc/generate-rpp", Some(&gen_payload)).await;
    assert_eq!(st, 200, "generate failed: {}", gen_body);
    let rpp_disk = std::fs::read_to_string(&out_rpp).unwrap();
    assert!(rpp_disk.contains("NAME \"Kick %1 %1\""));
    assert!(rpp_disk.contains("NAME \"Keys L (ST) %3 %3\""));
    assert!(rpp_disk.contains("Bass.wav")); // Preserved from X32Template.rpp!

    let parsed_ini = x32_reaper::config::Config::load(&out_ini).unwrap();
    assert_eq!(parsed_ini.trk_min, 1);
    assert_eq!(parsed_ini.trk_max, 31); // 32 channels with 3-4 stereo-linked -> 31 tracks

    // 7. Restore original pre-VSC routing from backup
    let restore_payload = serde_json::json!({
        "path": backup_path.to_string_lossy()
    })
    .to_string();
    let (st, _) =
        http_request(http_addr, "POST", "/api/vsc/restore-backup", Some(&restore_payload)).await;
    assert_eq!(st, 200);
    tokio::time::sleep(std::time::Duration::from_millis(60)).await;
    {
        let mut m = emu_mixer.lock().await;
        let peer: SocketAddr = "127.0.0.1:40001".parse().unwrap();
        let query = |path: &str| OscMessage::serialize_to_bytes(path, std::iter::empty::<&OscArg>()).unwrap();
        let r_play = m.dispatch(&query("/config/routing/PLAY/1-8"), peer).unwrap();
        let r_sw = m.dispatch(&query("/config/routing/routswitch"), peer).unwrap();
        assert_eq!(OscMessage::from_bytes(&r_play[0].1).unwrap().args[0], OscArg::Int(0));
        assert_eq!(OscMessage::from_bytes(&r_sw[0].1).unwrap().args[0], OscArg::Int(0));
    }

    ctx.mixer.disconnect().await;
    server_handle.abort();
}

#[tokio::test]
async fn test_console_first_digital_split_event_recorder_and_hw_buttons_e2e() {
    let tmp = TempDir::new().unwrap();
    let (emu_addr, emu_mixer) = spawn_test_emulator().await;

    // Spawn mock REAPER OSC UDP listener
    let reaper_sock = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let reaper_port = reaper_sock.local_addr().unwrap().port();
    let reaper_msgs: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let reaper_msgs_bg = reaper_msgs.clone();
    tokio::spawn(async move {
        let mut buf = [0u8; 2048];
        loop {
            if let Ok((len, _)) = reaper_sock.recv_from(&mut buf).await {
                if let Ok(msg) = OscMessage::from_bytes(&buf[..len]) {
                    reaper_msgs_bg.lock().await.push(msg.path);
                }
            }
        }
    });

    let profile_path = tmp.path().join("x32_studio_profile.json");
    let rec_dir = tmp.path().join("recordings");
    let ctx = Arc::new(AppContext::with_profile_path(profile_path));
    let (http_addr, server_handle) = spawn_server("127.0.0.1:0".parse().unwrap(), ctx.clone())
        .await
        .unwrap();

    // Connect to emulator
    let connect_payload = serde_json::json!({
        "ip": emu_addr.to_string(),
        "transport": "osc"
    })
    .to_string();
    let (st, _) = http_request(http_addr, "POST", "/api/connect", Some(&connect_payload)).await;
    assert_eq!(st, 200);

    // Save profile pointing REAPER to our mock UDP port & recordings to TempDir
    let mut prof = ctx.studio.status().await.profile;
    prof.reaper_ip = "127.0.0.1".to_string();
    prof.reaper_send_port = reaper_port;
    prof.recordings_dir = rec_dir.to_string_lossy().to_string();
    prof.pop_guard_mute_ms = 25;
    prof.split_mirror_aes50 = "aes50b".to_string();
    let prof_json = serde_json::to_string(&prof).unwrap();
    let (st, _) = http_request(http_addr, "POST", "/api/studio/profile", Some(&prof_json)).await;
    assert_eq!(st, 200);

    // Apply digital split + program console User Assign Bank C buttons
    let (st, _) = http_request(http_addr, "POST", "/api/studio/apply-split", Some("{}")).await;
    assert_eq!(st, 200);
    let (st, _) = http_request(http_addr, "POST", "/api/studio/program-buttons", Some("{}")).await;
    assert_eq!(st, 200);
    tokio::time::sleep(std::time::Duration::from_millis(60)).await;

    // Verify emulator received AES50B split + User Assign Bank C programming
    {
        let mut m = emu_mixer.lock().await;
        let peer: SocketAddr = "127.0.0.1:40001".parse().unwrap();
        let q = |path: &str| {
            OscMessage::serialize_to_bytes(path, std::iter::empty::<&OscArg>()).unwrap()
        };
        let r_aes = m.dispatch(&q("/config/routing/AES50B/1-8"), peer).unwrap();
        let r_btn5 = m.dispatch(&q("/config/userctrl/C/btn/5"), peer).unwrap();
        assert_eq!(OscMessage::from_bytes(&r_aes[0].1).unwrap().args[0], OscArg::Int(4));
        assert_eq!(
            OscMessage::from_bytes(&r_btn5[0].1).unwrap().args[0],
            OscArg::String("MN16000".to_string())
        );
    }

    // Simulate pressing physical X32 User Assign Button 6 (Record: userpar/18 = 127)
    let hw_sim = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let press_rec =
        OscMessage::serialize_to_bytes("/-stat/userpar/18/value", [&OscArg::Int(127)]).unwrap();
    hw_sim.send_to(&press_rec, emu_addr).await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let status_rec = ctx.studio.status().await;
    assert_eq!(status_rec.recorder.state, "recording");
    assert!(reaper_msgs.lock().await.contains(&"/record".to_string()));

    // Simulate a live fader move & physical Button 9 (Drop Marker: userpar/21 = 127)
    let fader_move =
        OscMessage::serialize_to_bytes("/ch/01/mix/fader", [&OscArg::Float(0.82)]).unwrap();
    hw_sim.send_to(&fader_move, emu_addr).await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let press_mkr =
        OscMessage::serialize_to_bytes("/-stat/userpar/21/value", [&OscArg::Int(127)]).unwrap();
    hw_sim.send_to(&press_mkr, emu_addr).await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(80)).await;

    let status_mkr = ctx.studio.status().await;
    assert!(status_mkr.recorder.event_count >= 1);
    assert_eq!(status_mkr.recorder.marker_count, 1);
    assert!(reaper_msgs.lock().await.contains(&"/action/40157".to_string()));

    // Simulate pressing physical Button 5 (Live -> DAW Mode: userpar/17 = 127)
    let press_mode =
        OscMessage::serialize_to_bytes("/-stat/userpar/17/value", [&OscArg::Int(127)]).unwrap();
    hw_sim.send_to(&press_mode, emu_addr).await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(160)).await;

    let status_daw = ctx.studio.status().await;
    assert_eq!(format!("{:?}", status_daw.mode), "Daw");
    assert_eq!(status_daw.recorder.state, "idle"); // Recording flushed cleanly!
    {
        let mut m = emu_mixer.lock().await;
        let peer: SocketAddr = "127.0.0.1:40001".parse().unwrap();
        let q = |path: &str| {
            OscMessage::serialize_to_bytes(path, std::iter::empty::<&OscArg>()).unwrap()
        };
        let r_sw = m.dispatch(&q("/config/routing/routswitch"), peer).unwrap();
        let r_col = m.dispatch(&q("/config/userctrl/C/color"), peer).unwrap();
        assert_eq!(OscMessage::from_bytes(&r_sw[0].1).unwrap().args[0], OscArg::Int(1));
        assert_eq!(OscMessage::from_bytes(&r_col[0].1).unwrap().args[0], OscArg::Int(2)); // Green
    }

    // Verify saved .x32events.jsonl file via HTTP endpoint
    let (st, recs_body) = http_request(http_addr, "GET", "/api/studio/recordings", None).await;
    assert_eq!(st, 200);
    let recs_json: serde_json::Value = serde_json::from_str(&recs_body).unwrap();
    assert_eq!(recs_json.as_array().unwrap().len(), 1);
    assert_eq!(recs_json[0]["markers"], 1);

    // Simulate pressing physical Button 5 again (DAW -> Live Mode: userpar/17 = 127)
    hw_sim.send_to(&press_mode, emu_addr).await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(160)).await;

    let status_live = ctx.studio.status().await;
    assert_eq!(format!("{:?}", status_live.mode), "Live");
    {
        let mut m = emu_mixer.lock().await;
        let peer: SocketAddr = "127.0.0.1:40001".parse().unwrap();
        let q = |path: &str| {
            OscMessage::serialize_to_bytes(path, std::iter::empty::<&OscArg>()).unwrap()
        };
        let r_sw = m.dispatch(&q("/config/routing/routswitch"), peer).unwrap();
        let r_col = m.dispatch(&q("/config/userctrl/C/color"), peer).unwrap();
        assert_eq!(OscMessage::from_bytes(&r_sw[0].1).unwrap().args[0], OscArg::Int(0));
        assert_eq!(OscMessage::from_bytes(&r_col[0].1).unwrap().args[0], OscArg::Int(1)); // Red
    }

    ctx.mixer.disconnect().await;
    server_handle.abort();
}

#[tokio::test]
async fn test_offline_scene_import_source_aware_digital_twin_e2e() {
    let tmp = TempDir::new().unwrap();
    let profile_path = tmp.path().join("x32_studio_profile.json");
    let ctx = Arc::new(AppContext::with_profile_path(profile_path));
    let (http_addr, server_handle) = spawn_server("127.0.0.1:0".parse().unwrap(), ctx.clone())
        .await
        .unwrap();

    // Synthetic X32 v4 scene covering source patching, shared physical inputs,
    // Aux/Bus/Matrix links, AES50 active routing, and UOUT1-8 on Card block 1.
    let synthetic_scene = "\
#4.0# \"Synthetic Twin\" \"\" %000000000 1\n\
/config/routing/IN 0 4 10 11 0\n\
/config/routing/AES50A 16 17 18 19 20 21\n\
/config/routing/CARD 26 1 2 3\n\
/config/routing/PLAY 10 11 12 4\n\
/config/auxlink ON OFF OFF OFF\n\
/config/buslink ON OFF OFF OFF OFF OFF OFF OFF\n\
/config/mtxlink ON OFF OFF\n\
/ch/01/config \"SynthKick\" 1 RD 5\n\
/ch/01/mix ON -3.0 OFF 10 ON +0.0\n\
/ch/02/config \"SynthVoxMain\" 2 BL 12\n\
/ch/02/mix ON -6.0 OFF 10 ON -25.0\n\
/ch/03/config \"SynthVoxDbl\" 3 BL 12\n\
/ch/03/mix OFF -12.0 OFF 10 ON +25.0\n\
/auxin/01/config \"PlaybackL\" 1 CY 33\n\
/bus/01/config \"DrumBus\" 1 RD\n\
/mtx/01/config \"MainMtx\" 1 YE\n";

    let import_body = serde_json::json!({
        "scene_content": synthetic_scene,
        "preserve_card_uout": true
    })
    .to_string();
    let (status, raw_resp) =
        http_request(http_addr, "POST", "/api/vsc/import-scene", Some(&import_body)).await;
    assert_eq!(status, 200, "import-scene failed: {raw_resp}");
    let resp: serde_json::Value = serde_json::from_str(&raw_resp).unwrap();
    assert_eq!(resp["state"]["model"], "X32");

    // Verify source-aware patching and link flags populated in snapshot
    assert_eq!(resp["state"]["channels"][0]["name"], "SynthKick");
    assert_eq!(resp["state"]["channels"][0]["source"], 5);
    assert_eq!(resp["state"]["channels"][1]["source"], 12);
    assert_eq!(resp["state"]["channels"][2]["source"], 12);
    assert_eq!(resp["state"]["auxins"][0]["linked_stereo"], true);
    assert_eq!(resp["state"]["auxins"][1]["linked_stereo"], true);
    assert_eq!(resp["state"]["buses"][0]["linked_stereo"], true);
    assert_eq!(resp["state"]["matrices"][0]["linked_stereo"], true);

    // UOUT preservation warning + preserved Card block 1 (26 = UOUT1-8)
    let plan_warnings = resp["routing_plan"]["warnings"].as_array().unwrap();
    assert!(!plan_warnings.is_empty(), "expected UOUT warning");
    assert_eq!(resp["routing_plan"]["card_blocks"][0], 26);

    // Also verify digital split protects active AES50A outputs
    let parsed_state: x32_ui::mixer::MixerSnapshotState =
        serde_json::from_value(resp["state"].clone()).unwrap();
    let split = x32_ui::studio_mode::compute_digital_split_plan_with_options(
        &parsed_state.routing,
        false,
        "aes50a",
        true,
    );
    assert!(
        split
            .warnings
            .iter()
            .any(|w| w.contains("AES50-A is already in use")),
        "expected AES50-A active protection warning, got {:?}",
        split.warnings
    );

    // Verify deduplicated session options & digital-twin .rpp generation
    assert_eq!(resp["session_options"]["include_empty_channels"], false);
    let rpp_out = tmp.path().join("SyntheticTwin.rpp");
    let mut sess_opts = resp["session_options"].clone();
    sess_opts["project_name"] = serde_json::json!("SyntheticTwin");
    sess_opts["deduplicate_shared_sources"] = serde_json::json!(true);
    sess_opts["digital_twin_mode"] = serde_json::json!(true);
    sess_opts["include_aux_tracks"] = serde_json::json!(true);
    sess_opts["include_bus_tracks"] = serde_json::json!(true);
    sess_opts["include_matrix_tracks"] = serde_json::json!(true);
    let gen_body = serde_json::json!({
        "session_options": sess_opts,
        "output_rpp_path": rpp_out.to_string_lossy(),
    })
    .to_string();
    let (status, raw_gen) =
        http_request(http_addr, "POST", "/api/vsc/generate-rpp", Some(&gen_body)).await;
    assert_eq!(status, 200, "generate-rpp failed: {raw_gen}");
    let gen_resp: serde_json::Value = serde_json::from_str(&raw_gen).unwrap();
    assert_eq!(gen_resp["ok"], true);

    let rpp = std::fs::read_to_string(&rpp_out).unwrap();
    assert!(rpp.contains("NAME \"SynthKick %1 %1\""));
    assert!(rpp.contains("NAME \"SynthVoxMain %2 "));
    assert!(
        !rpp.contains("SynthVoxDbl"),
        "shared source ch03 should be deduplicated"
    );
    assert!(rpp.contains("HWOUT 1028 0 "), "ch01 source 5 -> mono hw 1024 + 4");
    assert!(rpp.contains("HWOUT 1035 0 "), "ch02 source 12 -> mono hw 1024 + 11");
    assert!(rpp.contains("PlaybackL"), "aux track should be emitted");
    assert!(rpp.contains("DrumBus"));
    assert!(rpp.contains("MainMtx"));

    server_handle.abort();
}

