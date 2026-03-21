//! `x32_ssavergw` is a command-line tool that acts as a screen saver for the Behringer X32/M32.
//!
//! It monitors the X32 console for activity via OSC. If no activity is detected for a
//! specified amount of time, it saves the current brightness settings for the main LCD
//! screen and LEDs, and then sets them to their lowest values (0.0). When activity
//! is detected again, it restores the original brightness settings.
//!
//! # Credits
//!
//! *   **Original concept and work on the C library:** Patrick-Gilles Maillot
//! *   **Rust implementation by:** [User]

use anyhow::Result;
use clap::Parser;
use osc_lib::{OscArg, OscMessage};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tokio::time::{self, Duration, Instant};

mod state;
use state::AppState;

/// Set Low Light mode for X32 after a period of inactivity.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The IP address of the X32 console.
    #[arg(short, long, default_value = "192.168.0.64")]
    ip: String,

    /// Delay in seconds before entering Low Light mode.
    #[arg(short, long, default_value_t = 5)]
    delay: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let x32_addr: SocketAddr = format!("{}:10023", args.ip).parse()?;
    // We bind with tokio UdpSocket, avoiding `connect` to permit proper `send_to` usage later,
    // as macOS UDP sockets return "Socket is already connected (os error 56)" when using `send_to` on a `connect`ed socket.
    let bind_addr = if x32_addr.is_ipv4() {
        "0.0.0.0:0"
    } else {
        "[::]:0"
    };
    let socket = UdpSocket::bind(bind_addr).await?;

    let state = Arc::new(Mutex::new(AppState::new(args.delay)));

    println!("X32Ssaver - Rust Rewrite");
    println!("Connecting to X32 at {}...", args.ip);

    // Connect phase
    let info_msg = OscMessage {
        path: "/info".to_string(),
        args: vec![],
    };
    let info_bytes = info_msg
        .to_bytes()
        .map_err(|e| anyhow::anyhow!("OSC Error: {}", e))?;
    socket.send_to(&info_bytes, x32_addr).await?;

    let mut connected = false;
    let connect_start = Instant::now();
    let mut buf = [0u8; 1024];

    while connect_start.elapsed() < Duration::from_secs(1) {
        if let Ok(Ok((len, _))) =
            time::timeout(Duration::from_millis(100), socket.recv_from(&mut buf)).await
        {
            let s = String::from_utf8_lossy(&buf[..len]);
            if s.starts_with("/info") {
                state.lock().await.is_connected = true;
                connected = true;
                println!("Connected!");
                break;
            }
        }
    }

    if !connected {
        eprintln!("Connection timeout. Make sure the X32 is powered on and the IP is correct.");
        return Ok(());
    }

    println!("Delay before Low Light: {} seconds", args.delay);
    println!("Press Ctrl+C to exit.");

    let mut interval_xremote = time::interval(Duration::from_secs(9)); // /xremote timeout is 10s
    let mut interval_check = time::interval(Duration::from_millis(100)); // check screen saver state

    // Set up Ctrl+C handler to restore settings on exit
    let state_clone = Arc::clone(&state);
    let socket_clone = Arc::new(socket);
    let socket_ctrlc = Arc::clone(&socket_clone);
    let addr_clone = x32_addr;

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        println!("Exiting... restoring screen brightness.");
        let mut st = state_clone.lock().await;
        if st.is_dimmed {
            restore_brightness(&socket_ctrlc, addr_clone, &st)
                .await
                .unwrap_or_else(|e| eprintln!("Failed to restore brightness: {}", e));
            st.is_dimmed = false;
        }
        std::process::exit(0);
    });

    let xremote_msg = OscMessage::new("/xremote".to_string(), vec![]);
    let xremote_bytes = xremote_msg.to_bytes().unwrap_or_else(|_| b"/xremote\0\0\0\0".to_vec());

    loop {
        tokio::select! {
            _ = interval_xremote.tick() => {
                let _ = socket_clone.send_to(&xremote_bytes, x32_addr).await;
            }

            _ = interval_check.tick() => {
                let mut st = state.lock().await;
                if st.should_dim() {
                    // Enter screen saver
                    if !st.is_dimmed {
                        if let Err(e) = save_and_dim(&socket_clone, x32_addr, &mut *st).await {
                            eprintln!("Error saving/dimming: {}", e);
                        } else {
                            println!("Entered Low Light mode.");
                        }
                    }
                } else if st.should_restore() {
                    // Restore screen saver
                    if st.is_dimmed {
                        if let Err(e) = restore_brightness(&socket_clone, x32_addr, &*st).await {
                            eprintln!("Error restoring brightness: {}", e);
                        } else {
                            println!("Restored normal brightness.");
                            st.is_dimmed = false;
                        }
                    }
                }
            }

            Ok((len, _addr)) = socket_clone.recv_from(&mut buf) => {
                let data = &buf[..len];
                let s = String::from_utf8_lossy(data);

                // If the message is not just a response to our own /xremote or screen dimming
                // Note: /xremote responses are actually just state changes from the desk.
                // We consider any packet other than our own polling responses as activity.
                // Actually, just receiving any state change (not requested by us) is activity.
                // The C code resets the timer on ANY receive EXCEPT the responses to its own requests
                // during dimming.

                let is_bright_resp = s.starts_with("/-prefs/bright") || s.starts_with("/-prefs/ledbright");
                if !is_bright_resp {
                    let mut st = state.lock().await;
                    st.mark_activity();
                }
            }
        }
    }
}

async fn save_and_dim(socket: &UdpSocket, addr: SocketAddr, state: &mut AppState) -> Result<()> {
    // 1. Get current LCD bright
    let req_lcd = OscMessage {
        path: "/-prefs/bright".to_string(),
        args: vec![],
    };
    let req_lcd_bytes = req_lcd
        .to_bytes()
        .map_err(|e| anyhow::anyhow!("OSC Error: {}", e))?;
    socket.send_to(&req_lcd_bytes, addr).await?;

    // Wait for response, ignoring background traffic
    let mut buf = [0u8; 1024];
    let start_wait = Instant::now();
    while start_wait.elapsed() < Duration::from_millis(500) {
        if let Ok(Ok((len, _))) =
            time::timeout(Duration::from_millis(100), socket.recv_from(&mut buf)).await
        {
            if let Ok(msg) = OscMessage::from_bytes(&buf[..len]) {
                if msg.path == "/-prefs/bright" {
                    if let Some(OscArg::Float(f)) = msg.args.first() {
                        state.saved_lcd_bright = *f;
                        break;
                    }
                }
            }
        }
    }

    // 2. Get current LED bright
    let req_led = OscMessage {
        path: "/-prefs/ledbright".to_string(),
        args: vec![],
    };
    let req_led_bytes = req_led
        .to_bytes()
        .map_err(|e| anyhow::anyhow!("OSC Error: {}", e))?;
    socket.send_to(&req_led_bytes, addr).await?;

    let start_wait = Instant::now();
    while start_wait.elapsed() < Duration::from_millis(500) {
        if let Ok(Ok((len, _))) =
            time::timeout(Duration::from_millis(100), socket.recv_from(&mut buf)).await
        {
            if let Ok(msg) = OscMessage::from_bytes(&buf[..len]) {
                if msg.path == "/-prefs/ledbright" {
                    if let Some(OscArg::Float(f)) = msg.args.first() {
                        state.saved_led_bright = *f;
                        break;
                    }
                }
            }
        }
    }

    // 3. Set both to lowest values (0.0)
    let set_lcd = OscMessage {
        path: "/-prefs/bright".to_string(),
        args: vec![OscArg::Float(0.0)],
    };
    let set_lcd_bytes = set_lcd
        .to_bytes()
        .map_err(|e| anyhow::anyhow!("OSC Error: {}", e))?;
    socket.send_to(&set_lcd_bytes, addr).await?;

    let set_led = OscMessage {
        path: "/-prefs/ledbright".to_string(),
        args: vec![OscArg::Float(0.0)],
    };
    let set_led_bytes = set_led
        .to_bytes()
        .map_err(|e| anyhow::anyhow!("OSC Error: {}", e))?;
    socket.send_to(&set_led_bytes, addr).await?;

    state.is_dimmed = true;
    Ok(())
}

async fn restore_brightness(socket: &UdpSocket, addr: SocketAddr, state: &AppState) -> Result<()> {
    let set_lcd = OscMessage {
        path: "/-prefs/bright".to_string(),
        args: vec![OscArg::Float(state.saved_lcd_bright)],
    };
    let set_lcd_bytes = set_lcd
        .to_bytes()
        .map_err(|e| anyhow::anyhow!("OSC Error: {}", e))?;
    socket.send_to(&set_lcd_bytes, addr).await?;

    let set_led = OscMessage {
        path: "/-prefs/ledbright".to_string(),
        args: vec![OscArg::Float(state.saved_led_bright)],
    };
    let set_led_bytes = set_led
        .to_bytes()
        .map_err(|e| anyhow::anyhow!("OSC Error: {}", e))?;
    socket.send_to(&set_led_bytes, addr).await?;

    Ok(())
}
