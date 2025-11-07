//! `x32_screen_saver` is a command-line utility that connects to a Behringer X32 or Midas M32
//! digital mixer and activates a screen saver mode after a period of inactivity. When the
//! screen saver is active, the mixer's screen and LED brightness are turned down to their
//! minimum values. Any activity on the mixer will restore the original brightness levels.
//!
//! This tool is a Rust rewrite of the original `X32SsaverGW.c` utility by
//! Patrick-Gilles Maillot.

use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use x32_lib::create_socket;
use x32_lib::error::Result;
use osc_lib::{OscMessage, OscArg};
use std::sync::{Arc, Mutex};
use std::thread;

/// A command-line utility to activate a screen saver on an X32/M32 console after a period of inactivity.
#[derive(Parser, Debug, Serialize, Deserialize)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// IP address of the X32 console
    #[arg(short, long)]
    ip: String,

    /// Delay in seconds before the screen saver activates
    #[arg(short, long, default_value_t = 5)]
    delay: u64,

    /// Save the current configuration to a file
    #[arg(long)]
    save_config: Option<PathBuf>,

    /// Load configuration from a file
    #[arg(long)]
    load_config: Option<PathBuf>,
}

/// The main entry point for the `x32_screen_saver` utility.
///
/// This function parses command-line arguments, connects to the X32, and enters a loop
/// to monitor for activity. If no activity is detected for the specified delay period,
/// the screen saver is activated. When activity resumes, the screen saver is deactivated.
fn main() -> Result<()> {
    let mut args = Args::parse();

    if let Some(load_path) = &args.load_config {
        if let Ok(config_str) = fs::read_to_string(load_path) {
            if let Ok(config) = serde_json::from_str::<Args>(&config_str) {
                args = config;
            }
        }
    }

    if let Some(save_path) = &args.save_config {
        if let Ok(config_str) = serde_json::to_string_pretty(&args) {
            let _ = fs::write(save_path, config_str);
        }
    }

    println!("Connecting to X32 at {}...", args.ip);
    let socket = Arc::new(create_socket(&args.ip, 10)?);
    println!("Connected!");

    let last_activity = Arc::new(Mutex::new(Instant::now()));
    let screen_saver_on = Arc::new(Mutex::new(false));

    // Keepalive thread
    let socket_clone = Arc::clone(&socket);
    thread::spawn(move || {
        loop {
            let xremote_msg = OscMessage::new("/xremote".to_string(), vec![]);
            if socket_clone.send(&xremote_msg.to_bytes().unwrap()).is_err() {
                eprintln!("Failed to send /xremote");
            }
            thread::sleep(Duration::from_secs(9));
        }
    });


    let mut old_lcd_bright = 0.0;
    let mut old_led_bright = 0.0;

    let lcd_low_bright_msg = OscMessage::new("/-prefs/bright".to_string(), vec![OscArg::Float(0.0)]);
    let led_low_bright_msg = OscMessage::new("/-prefs/ledbright".to_string(), vec![OscArg::Float(0.0)]);

    loop {
        let mut buf = [0; 512];
        match socket.recv(&mut buf) {
            Ok(len) => {
                if len > 0 {
                    *last_activity.lock().unwrap() = Instant::now();
                    let mut ss_on = screen_saver_on.lock().unwrap();
                    if *ss_on {
                        println!("Activity detected. Restoring brightness.");
                        let lcd_old_bright_msg = OscMessage::new("/-prefs/bright".to_string(), vec![OscArg::Float(old_lcd_bright)]);
                        let led_old_bright_msg = OscMessage::new("/-prefs/ledbright".to_string(), vec![OscArg::Float(old_led_bright)]);
                        socket.send(&lcd_old_bright_msg.to_bytes()?)?;
                        socket.send(&led_old_bright_msg.to_bytes()?)?;
                        *ss_on = false;
                    }
                }
            }
            Err(e) => {
                if e.kind() != std::io::ErrorKind::WouldBlock && e.kind() != std::io::ErrorKind::TimedOut {
                    return Err(e.into());
                }
            }
        }

        let mut ss_on = screen_saver_on.lock().unwrap();
        if !*ss_on && last_activity.lock().unwrap().elapsed() > Duration::from_secs(args.delay) {
            println!("Inactivity detected. Activating screen saver.");

            // Get current brightness levels
            socket.send(&OscMessage::new("/-prefs/bright".to_string(), vec![]).to_bytes()?)?;
            if let Ok(len) = socket.recv(&mut buf) {
                if let Ok(msg) = OscMessage::from_bytes(&buf[..len]) {
                    if let Some(OscArg::Float(val)) = msg.args.get(0) {
                        old_lcd_bright = *val;
                    }
                }
            }
            socket.send(&OscMessage::new("/-prefs/ledbright".to_string(), vec![]).to_bytes()?)?;
            if let Ok(len) = socket.recv(&mut buf) {
                if let Ok(msg) = OscMessage::from_bytes(&buf[..len]) {
                    if let Some(OscArg::Float(val)) = msg.args.get(0) {
                        old_led_bright = *val;
                    }
                }
            }

            socket.send(&lcd_low_bright_msg.to_bytes()?)?;
            socket.send(&led_low_bright_msg.to_bytes()?)?;
            *ss_on = true;
        }

        thread::sleep(Duration::from_millis(100));
    }
}
