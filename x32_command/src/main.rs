use anyhow::{Context, Result};
use clap::Parser;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;

/// X32_Command - a simple udp client for X32 sending commands and getting answers
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = "usage: x32_command [OPTIONS]")]
struct Args {
    /// X32 console ipv4 address
    #[arg(short, long)]
    ip: String,

    /// debug option (0/1)
    #[arg(short, long, default_value_t = 0)]
    debug: u8,

    /// verbose option (0/1)
    #[arg(short, long, default_value_t = 1)]
    verbose: u8,

    /// keyboard mode on (0/1)
    #[arg(short, long, default_value_t = 1)]
    keyboard: u8,

    /// delay between batch commands in ms
    #[arg(short, long, default_value_t = 10)]
    time: u32,

    /// sets batch mode on, getting input data from 'file'
    #[arg(short, long)]
    file: Option<String>,

    /// sets batch mode on, getting input data from snippets/tidbits/X32node 'file'
    #[arg(short, long)]
    snippet: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    println!(" X32_Command - Rust Rewrite - (c)2014-20 Patrick-Gilles Maillot");
    print!("Connecting to X32.");

    let port = std::env::var("X32_PORT").unwrap_or_else(|_| "10023".to_string());
    let addr: SocketAddr = format!("{}:{}", args.ip, port)
        .parse()
        .context("Invalid IP address")?;

    let socket = UdpSocket::bind("0.0.0.0:0")
        .await
        .context("Failed to bind socket")?;

    let socket = Arc::new(socket);

    // Connect to X32 (if an IP was provided, which is required)
    let mut buf = [0u8; 512];

    let max_retries = 3;
    let mut retries = 0;

    loop {
        // Send /info request
        if let Err(e) = socket.send_to(b"/info", addr).await {
            eprintln!("Failed to send /info: {}", e);
            break; // Just break so we don't hang in tests if we use dummy IP
        }

        let res = tokio::time::timeout(Duration::from_millis(50), socket.recv_from(&mut buf)).await;
        match res {
            Ok(Ok((len, _src))) => {
                let msg = String::from_utf8_lossy(&buf[..len]);
                if msg.starts_with("/info") {
                    break;
                }
            }
            Ok(Err(e)) => {
                eprintln!("\nPolling for data failed: {}", e);
                return Err(e.into());
            }
            Err(_) => {
                retries += 1;
                if retries >= max_retries {
                    break; // Just break so we don't hang if no X32 is present
                }
                print!(".");
                use std::io::Write;
                let _ = std::io::stdout().flush();
            }
        }
    }

    println!(" Done!");

    // We might need to give the connection loop a moment to settle, or the OS a moment to route the UDP packet
    tokio::time::sleep(Duration::from_millis(10)).await;

    let mut do_keyboard = args.keyboard != 0;
    let s_delay = Arc::new(Mutex::new(args.time));
    let verbose = Arc::new(Mutex::new(args.verbose != 0));
    let xremote_on = Arc::new(Mutex::new(false));

    // Spawn a task to listen for incoming messages
    let socket_recv = socket.clone();
    let verbose_recv = verbose.clone();
    tokio::spawn(async move {
        let mut recv_buf = [0u8; 1024];
        loop {
            if let Ok((len, _src)) = socket_recv.recv_from(&mut recv_buf).await {
                let is_verbose = *verbose_recv.lock().await;
                if is_verbose {
                    // Try to parse the OSC message
                    if let Ok(msg) = osc_lib::OscMessage::from_bytes(&recv_buf[..len]) {
                        println!("X-> {}", msg);
                    } else {
                        let mut hex_str = String::new();
                        static HEX: &[u8; 16] = b"0123456789abcdef";
                        for &byte in &recv_buf[..len] {
                            hex_str.push(HEX[(byte >> 4) as usize] as char);
                            hex_str.push(HEX[(byte & 0x0f) as usize] as char);
                            hex_str.push(' ');
                        }
                        println!("X-> [Raw] {}", hex_str);
                    }
                }
            }
        }
    });

    // Spawn a task for xremote keeping the connection alive
    let socket_xremote = socket.clone();
    let xremote_state = xremote_on.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(9));
        loop {
            interval.tick().await;
            let is_on = *xremote_state.lock().await;
            if is_on {
                let _ = socket_xremote.send_to(b"/xremote", addr).await;
            }
        }
    });

    let mut keep_on = true;
    let mut _batch_aborted = false;

    // We process snippet file first if provided
    let files_to_process = vec![
        args.snippet.as_ref().map(|s| (s, true)), // true indicates snippet mode (longer timeout usually, though we just sleep)
        args.file.as_ref().map(|s| (s, false)),
    ];

    for (file_path, is_snippet) in files_to_process.into_iter().flatten() {
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        let file = File::open(file_path).context(format!("Cannot read file: {}", file_path))?;

        // Security: Prevent OOM from maliciously large or corrupted files.
        // Also protect against special system files (like /dev/zero) that report 0 length
        // but stream infinite data.
        if file.metadata()?.len() > 1024 * 1024 {
            // 1MB limit
            return Err(anyhow::anyhow!("File too large"));
        }
        let mut reader = BufReader::new(file);
        use std::io::Read;

        for line_res in reader.by_ref().take(1024 * 1024).lines() {
            if !keep_on {
                break;
            }

            let line = line_res?;
            let line = line.trim();

            if line.starts_with('#') {
                println!("---comment: {}", line);
            } else if line == "exit" || line == "quit" {
                keep_on = false;
            } else if line == "kill" {
                keep_on = false;
                do_keyboard = false;
            } else if let Some(stripped) = line.strip_prefix("time ") {
                if let Ok(val) = stripped.parse::<u32>() {
                    *s_delay.lock().await = val;
                    println!(":: delay is: {}", val);
                }
            } else if line == "verbose" {
                let v = *verbose.lock().await;
                println!(":: verbose is {}", if v { "on" } else { "off" });
            } else if line == "verbose off" {
                *verbose.lock().await = false;
            } else if line == "verbose on" {
                *verbose.lock().await = true;
            } else if line == "xremote" {
                let x = *xremote_on.lock().await;
                println!(":: xremote is {}", if x { "on" } else { "off" });
            } else if line == "xremote off" {
                *xremote_on.lock().await = false;
            } else if line == "xremote on" {
                *xremote_on.lock().await = true;
            } else if !line.is_empty() {
                use std::str::FromStr;
                // For snippets, we could handle raw nodes strings differently if needed.
                // The C code parses lines using a custom parser. In rust, osc_lib handles string parsing.
                if let Ok(msg) = osc_lib::OscMessage::from_str(line) {
                    if let Ok(bytes) = msg.to_bytes() {
                        let delay = *s_delay.lock().await;
                        if delay > 0 {
                            tokio::time::sleep(Duration::from_millis(delay as u64)).await;
                        } else if is_snippet {
                            // snippet files often need a bit of time between commands
                            tokio::time::sleep(Duration::from_millis(10)).await;
                        }
                        socket.send_to(&bytes, addr).await?;
                    }
                } else {
                    eprintln!("Failed to parse line: {}", line);
                }
            }
        }
        println!("---end of batch mode file");
    }

    if do_keyboard {
        use std::io::BufRead;
        let stdin = std::io::stdin();
        let mut keep_on = true;

        for line_res in stdin.lock().lines() {
            if !keep_on {
                break;
            }

            let line = line_res?;
            let line = line.trim();

            if line.starts_with('#') {
                println!("---comment: {}", line);
            } else if line == "exit" || line == "quit" {
                keep_on = false;
            } else if let Some(stripped) = line.strip_prefix("time ") {
                if let Ok(val) = stripped.parse::<u32>() {
                    *s_delay.lock().await = val;
                    println!(":: delay is: {}", val);
                }
            } else if line == "verbose" {
                let v = *verbose.lock().await;
                println!(":: verbose is {}", if v { "on" } else { "off" });
            } else if line == "verbose off" {
                *verbose.lock().await = false;
            } else if line == "verbose on" {
                *verbose.lock().await = true;
            } else if line == "xremote" {
                let x = *xremote_on.lock().await;
                println!(":: xremote is {}", if x { "on" } else { "off" });
            } else if line == "xremote off" {
                *xremote_on.lock().await = false;
            } else if line == "xremote on" {
                *xremote_on.lock().await = true;
            } else if !line.is_empty() {
                use std::str::FromStr;
                if let Ok(msg) = osc_lib::OscMessage::from_str(line) {
                    if let Ok(bytes) = msg.to_bytes() {
                        let delay = *s_delay.lock().await;
                        if delay > 0 {
                            tokio::time::sleep(Duration::from_millis(delay as u64)).await;
                        }
                        socket.send_to(&bytes, addr).await?;
                    }
                } else {
                    eprintln!("Failed to parse line: {}", line);
                }
            }
        }
    }

    Ok(())
}
