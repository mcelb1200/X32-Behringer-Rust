use anyhow::{Context, Result};
use clap::Parser;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use x32_lib::MixerClient;

/// XAir_Command - a simple udp client for XR12, 16 or 18 sending commands and getting answers
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = "usage: xair_command [OPTIONS]")]
struct Args {
    /// X32 console ipv4 address
    #[arg(short, long, default_value = "192.168.0.64")]
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
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    println!(" XAir_Command - Rust Rewrite - (c)2014-18 Patrick-Gilles Maillot");
    print!("Connecting to XR18.");

    let port = std::env::var("XAIR_PORT").unwrap_or_else(|_| "10024".to_string());
    let _addr: SocketAddr = format!("{}:{}", args.ip, port)
        .parse()
        .context("Invalid IP address")?;

    let client = MixerClient::connect(&args.ip, true).await?;

    let mut do_keyboard = args.keyboard != 0;
    let s_delay = Arc::new(Mutex::new(args.time));
    let verbose = Arc::new(Mutex::new(args.verbose != 0));
    let xremote_on = Arc::new(Mutex::new(false));

    let mut rx = client.subscribe();
    let verbose_recv = verbose.clone();
    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if *verbose_recv.lock().await {
                println!("X-> {}", msg);
            }
        }
    });

    let mut keep_on = true;
    let mut _batch_aborted = false;

    if let Some(file_path) = &args.file {
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
        use std::io::Read;
        let reader = BufReader::new(file.take(1024 * 1024));

        for line_res in reader.lines() {
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
                client.stop_heartbeat();
                *xremote_on.lock().await = false;
            } else if line == "xremote on" {
                client.start_heartbeat();
                *xremote_on.lock().await = true;
            } else if !line.is_empty() {
                use std::str::FromStr;
                if let Ok(msg) = osc_lib::OscMessage::from_str(line) {
                    if let Ok(_bytes) = msg.to_bytes() {
                        let delay = *s_delay.lock().await;
                        if delay > 0 {
                            tokio::time::sleep(Duration::from_millis(delay as u64)).await;
                        }
                        client.send_message(&msg.path, msg.args).await?;
                    }
                } else {
                    eprintln!("Failed to parse line: {}", line);
                }
            }
        }
        println!("---end of batch mode file");
    }

    if do_keyboard {
        use std::io::{BufRead, Read};
        let stdin = std::io::stdin();
        let mut stdin_lock = stdin.lock();
        let mut keep_on = true;

        loop {
            if !keep_on {
                break;
            }

            let mut line_buf = String::new();
            let mut handle = stdin_lock.by_ref().take(4096);
            if handle.read_line(&mut line_buf).is_err() || line_buf.is_empty() {
                break;
            }
            if !line_buf.ends_with('\n') && line_buf.len() == 4096 {
                // If it doesn't end with a newline and hit the length limit, the line was too long.
                // Clear the rest of the line from stdin to avoid processing partial commands.
                let mut discard = Vec::with_capacity(1024);
                loop {
                    discard.clear();
                    let mut chunk_handle = stdin_lock.by_ref().take(1024);
                    match chunk_handle.read_until(b'\n', &mut discard) {
                        Ok(0) | Err(_) => break,
                        Ok(_) => {
                            if discard.ends_with(b"\n") {
                                break;
                            }
                        }
                    }
                }
                eprintln!("Input line too long, discarded.");
                continue;
            }
            let line = line_buf.trim();

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
                client.stop_heartbeat();
                *xremote_on.lock().await = false;
            } else if line == "xremote on" {
                client.start_heartbeat();
                *xremote_on.lock().await = true;
            } else if !line.is_empty() {
                use std::str::FromStr;
                if let Ok(msg) = osc_lib::OscMessage::from_str(line) {
                    if let Ok(_bytes) = msg.to_bytes() {
                        let delay = *s_delay.lock().await;
                        if delay > 0 {
                            tokio::time::sleep(Duration::from_millis(delay as u64)).await;
                        }
                        client.send_message(&msg.path, msg.args).await?;
                    }
                } else {
                    eprintln!("Failed to parse line: {}", line);
                }
            }
        }
    }

    Ok(())
}
