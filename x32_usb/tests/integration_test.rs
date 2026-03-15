use std::sync::mpsc::{channel, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use x32_emulator::server;
use x32_emulator::Mixer;

fn run_server_with_seeder<F>(port: u16, seeder: F) -> (JoinHandle<()>, Sender<()>)
where
    F: FnOnce(&mut Mixer) + Send + 'static,
{
    let (tx, rx) = channel();
    let handle = thread::spawn(move || {
        server::run(
            &format!("127.0.0.1:{}", port),
            Some(Box::new(seeder)),
            Some(rx),
        )
        .unwrap();
    });
    thread::sleep(Duration::from_millis(200));
    (handle, tx)
}

#[test]
fn test_not_connected() {
    // We don't start a server for this test, to simulate a connection failure.
    let bin = escargot::CargoBuild::new().bin("x32_usb").run().unwrap();
    let mut cmd = bin.command();
    cmd.arg("--ip").arg("127.0.0.1:10047").arg("ls");

    let output = cmd.output().unwrap();
    assert!(!output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Not connected to X32."));
}

#[test]
fn test_ls_command() {
    let (handle, tx) = run_server_with_seeder(10048, |mixer| {
        mixer.seed_from_lines(vec![
            "/-stat/usbmounted,i\t1",
            "/-usb/dir/maxpos,i\t3",
            "/-usb/dir/001/name,s\t[..]",
            "/-usb/dir/002/name,s\t[System Volume Information]",
            "/-usb/dir/003/name,s\ttrack01.wav",
        ]);
    });

    let bin = escargot::CargoBuild::new().bin("x32_usb").run().unwrap();
    let mut cmd = bin.command();
    cmd.arg("--ip").arg("127.0.0.1:10048").arg("ls");

    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        stdout,
        "FileEntry { index: 1, name: \"[..]\", file_type: Parent }\n\
         FileEntry { index: 2, name: \"[System Volume Information]\", file_type: Volume }\n\
         FileEntry { index: 3, name: \"track01.wav\", file_type: Wav }\n"
    );

    tx.send(()).unwrap();
    handle.join().unwrap();
}

#[test]
fn test_file_operations() {
    let (handle, tx) = run_server_with_seeder(10049, |mixer| {
        mixer.seed_from_lines(vec![
            "/-stat/usbmounted,i\t1",
            "/-usb/dir/maxpos,i\t4",
            "/-usb/dir/001/name,s\t[..]",
            "/-usb/dir/002/name,s\t[MyScenes]",
            "/-usb/dir/003/name,s\tmyscene.scn",
            "/-usb/dir/004/name,s\ttrack02.wav",
        ]);
    });

    let bin = escargot::CargoBuild::new().bin("x32_usb").run().unwrap();
    let mut cmd = bin.command();
    cmd.arg("--ip")
        .arg("127.0.0.1:10049")
        .arg("cd")
        .arg("MyScenes");

    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout, "Changed directory to [MyScenes]\n");

    let mut cmd = bin.command();
    cmd.arg("--ip")
        .arg("1.2.3.4")
        .arg("load")
        .arg("myscene.scn");

    let output = cmd.output().unwrap();
    assert!(!output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Not connected to X32."));

    let mut cmd = bin.command();
    cmd.arg("--ip").arg("127.0.0.1:10049").arg("play").arg("4");

    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout, "Playing file: track02.wav\n");

    tx.send(()).unwrap();
    handle.join().unwrap();
}
