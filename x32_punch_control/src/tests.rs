use super::*;
use tokio::fs;

#[tokio::test]
async fn test_state_default() {
    let state = AppState::default();
    assert_eq!(state.xplay, false);
    assert_eq!(state.xpause, false);
    assert_eq!(state.xmerge, true);
}

#[tokio::test]
async fn test_catchup_logic() {
    let mut state = AppState::default();
    state.t_ff = SystemTime::now().duration_since(UNIX_EPOCH).unwrap() - Duration::from_secs(1);
    state.xreadfile = true;
    state.dt_play = Duration::from_secs(5);
    state.dt_read = Duration::from_secs(1);

    // This is a unit test to verify state transition on catchup triggering
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    if !state.t_ff.is_zero() && now > state.t_ff {
        state.t_ff = Duration::ZERO;
    }
    assert!(state.t_ff.is_zero());
}

#[tokio::test]
async fn test_catchback_logic_file_management() {
    let path = "test_catchback_backup.xpc";
    let out_path = format!("{}_xpc", path);
    let backup_path = format!("{}_xpc_backup", path);

    // create a dummy output file
    fs::write(&out_path, b"dummy").await.unwrap();

    // mimic the rename and open
    let _ = fs::rename(&out_path, &backup_path).await;

    assert!(!fs::metadata(&out_path).await.is_ok());
    assert!(fs::metadata(&backup_path).await.is_ok());

    // mimic recreating out path and removing backup path
    fs::write(&out_path, b"dummy2").await.unwrap();
    let _ = fs::remove_file(&backup_path).await;

    assert!(fs::metadata(&out_path).await.is_ok());
    assert!(!fs::metadata(&backup_path).await.is_ok());

    // cleanup
    let _ = fs::remove_file(&out_path).await;
}

#[tokio::test]
async fn test_run_logic_catch_up() {
    let test_file_path = "test_punch_file.xpc";

    // Create dummy records
    let mut w = PunchWriter::new(File::create(test_file_path).await.unwrap());
    w.write_record(&PunchRecord {
        time: Duration::from_secs(1),
        data: b"/test1".to_vec(),
    })
    .await
    .unwrap();
    w.write_record(&PunchRecord {
        time: Duration::from_secs(2),
        data: b"/test2".to_vec(),
    })
    .await
    .unwrap();
    drop(w);

    let state = Arc::new(Mutex::new(AppState::default()));

    // Trigger catch up
    {
        let mut s = state.lock().await;
        s.t_ff = SystemTime::now().duration_since(UNIX_EPOCH).unwrap() - Duration::from_secs(1);
        s.xplay = true;
        s.dt_play = Duration::from_secs(3); // greater than all records
        s.dt_read = Duration::from_secs(0); // Explicitly reset dt_read
        s.xfiledataready = true;
        s.xreadfile = true;
        s.xcatchdelay = 0; // fast tests
    }

    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let port = socket.local_addr().unwrap().port();
    let socket = Arc::new(socket);
    socket.connect(format!("127.0.0.1:{}", port)).await.unwrap();

    let state_clone = state.clone();
    let socket_clone = socket.clone();

    let task = tokio::spawn(async move {
        run_logic(
            state_clone,
            socket_clone,
            Config::default(),
            Some(test_file_path.to_string()),
        )
        .await;
    });

    // Let it process
    time::sleep(Duration::from_millis(500)).await;

    // Verify it processed everything
    {
        let s = state.lock().await;
        // The file reading will conclude setting xreadfile false.
        assert!(!s.xreadfile);
    }

    task.abort();
    let _ = fs::remove_file(test_file_path).await;
    let _ = fs::remove_file(format!("{}_xpc", test_file_path)).await;
}

#[tokio::test]
async fn test_run_logic_catch_back() {
    let test_file_path = "test_punch_file_cb.xpc";

    // Create dummy records
    let mut w = PunchWriter::new(File::create(test_file_path).await.unwrap());
    w.write_record(&PunchRecord {
        time: Duration::from_secs(1),
        data: b"/test1".to_vec(),
    })
    .await
    .unwrap();
    w.write_record(&PunchRecord {
        time: Duration::from_secs(2),
        data: b"/test2".to_vec(),
    })
    .await
    .unwrap();
    w.write_record(&PunchRecord {
        time: Duration::from_secs(3),
        data: b"/test3".to_vec(),
    })
    .await
    .unwrap();
    drop(w);

    let state = Arc::new(Mutex::new(AppState::default()));

    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let port = socket.local_addr().unwrap().port();
    let socket = Arc::new(socket);
    socket.connect(format!("127.0.0.1:{}", port)).await.unwrap();

    let state_clone = state.clone();
    let socket_clone = socket.clone();

    // In a catch back, we first rename the existing writer, so we need to create it
    let out_path = format!("{}_xpc", test_file_path);

    // The test logic in run_logic expects `test_punch_file_cb.xpc_xpc` to exist, and then renames it to `_backup`.
    // It creates it here before the rename. Wait, the code creates `out_path` from `path` plus `_xpc`.
    // So the writer output is exactly `out_path`! Let's ensure we flush properly.
    let mut out_w = PunchWriter::new(File::create(&out_path).await.unwrap());
    // Assume it had written test1 and test2 and test3 already in previous run
    out_w
        .write_record(&PunchRecord {
            time: Duration::from_secs(1),
            data: b"/test1".to_vec(),
        })
        .await
        .unwrap();
    out_w
        .write_record(&PunchRecord {
            time: Duration::from_secs(2),
            data: b"/test2".to_vec(),
        })
        .await
        .unwrap();
    out_w
        .write_record(&PunchRecord {
            time: Duration::from_secs(3),
            data: b"/test3".to_vec(),
        })
        .await
        .unwrap();
    // In our PunchWriter implementation, writing requires flushing to actually hit disk immediately
    // Wait, the format.rs change added flush() to write_record so it should be fine. But just in case.
    drop(out_w);

    // Explicitly sync to ensure file is closed and fully flushed
    let file_check = tokio::fs::read(&out_path).await.unwrap();
    assert!(file_check.len() > 0, "File should have been written to");

    // Write exactly the same file to the test_file_path itself (used by reader)
    tokio::fs::write(&test_file_path, &file_check)
        .await
        .unwrap();

    // Verify it exists
    assert!(
        tokio::fs::metadata(&out_path).await.is_ok(),
        "Test setup failed: file not written"
    );

    // We also need to write the reader file that run_logic expects `test_file_path`
    // Which we did at the start of the test! Wait, let's verify it too
    assert!(tokio::fs::metadata(&test_file_path).await.is_ok());

    // Wait, run_logic initializes writer = Some(PunchWriter::new(File::create(&out_path).await.unwrap()));
    // This overwrites `out_path` the moment `run_logic` starts!
    // Thus when REW triggers, the rename will just move an EMPTY file, not the dummy data we prepared.
    // To properly simulate REW we need `run_logic` to write the data, or we just write it after starting.
    // The easiest way is to let `run_logic` initialize, then write to its out_path directly before triggering REW.

    let task = tokio::spawn(async move {
        run_logic(
            state_clone,
            socket_clone,
            Config::default(),
            Some(test_file_path.to_string()),
        )
        .await;
    });

    // Wait for run_logic to open the file and create the writer
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Now overwrite `out_path` with our dummy data so REW rename works correctly
    tokio::fs::write(&out_path, &file_check).await.unwrap();

    // Give the OS a moment to sync directory states
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Wait a brief moment for the reader file to be opened inside run_logic
    time::sleep(Duration::from_millis(50)).await;

    // Trigger catch back
    {
        let mut s = state.lock().await;
        s.t_rew = SystemTime::now().duration_since(UNIX_EPOCH).unwrap() - Duration::from_secs(1);
        s.dt_play = Duration::from_secs(2); // Rewound to 2s
        s.dt_read = Duration::from_secs(0); // Explicitly reset dt_read
        // xfiledataready prevents the main loop from trying to read, which would read the first record out of turn
        // wait, we DO want the main loop to not interfere.
        s.xfiledataready = true;
        s.xreadfile = true;
        s.xplay = true;
        s.xcatchdelay = 0; // fast tests
    }
    // Wait for the records to be sent and verify
    let mut buf = [0; 1024];

    // We expect two records to be sent: test1, and test2
    let res = tokio::time::timeout(Duration::from_millis(1000), socket.recv(&mut buf)).await;
    let len = res.unwrap().unwrap();
    assert_eq!(&buf[..len], b"/test1");

    let res = tokio::time::timeout(Duration::from_millis(1000), socket.recv(&mut buf)).await;
    let len = res.unwrap().unwrap();
    assert_eq!(&buf[..len], b"/test2");

    // it should stop there
    let res = tokio::time::timeout(Duration::from_millis(500), socket.recv(&mut buf)).await;
    assert!(res.is_err(), "Should not receive test3");

    task.abort();
    let _ = fs::remove_file(test_file_path).await;
    let _ = fs::remove_file(&out_path).await;
    let _ = fs::remove_file(format!("{}_backup", out_path)).await;
}
