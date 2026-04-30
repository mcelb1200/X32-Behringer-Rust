use anyhow::{Context, Result};
use clap::Parser;
use osc_lib::OscMessage;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::Duration;
use x32_lib::MixerClient;

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

    let client = MixerClient::connect(&args.ip, false)
        .await
        .context("Failed to connect to X32")?;
    println!(" Done!");

    let mut do_keyboard = args.keyboard != 0;
    let s_delay = Arc::new(Mutex::new(args.time));
    let verbose = Arc::new(Mutex::new(args.verbose != 0));

    // Subscribe to messages for verbose output
    let mut rx = client.subscribe();
    let verbose_clone = verbose.clone();
    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if *verbose_clone.lock().await {
                println!("X-> {}", msg);
            }
        }
    });

    let mut keep_on = true;

    // We process snippet file first if provided
    let files_to_process = vec![
        args.snippet.as_ref().map(|s| (s, true)),
        args.file.as_ref().map(|s| (s, false)),
    ];

    for (file_path, is_snippet) in files_to_process.into_iter().flatten() {
        use std::fs::File;
        use std::io::{BufRead, BufReader, Read};

        let file = File::open(file_path).context(format!("Cannot read file: {}", file_path))?;
        if file.metadata()?.len() > 1024 * 1024 {
            return Err(anyhow::anyhow!("File too large"));
        }
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
                // We don't have an easy way to query current heartbeat status from MixerClient
                // but we can just use the toggle commands.
                println!(":: xremote toggle");
            } else if line == "xremote off" {
                client.stop_heartbeat();
            } else if line == "xremote on" {
                client.start_heartbeat();
            } else if !line.is_empty() {
                if let Ok(msg) = OscMessage::from_str(line) {
                    let delay = *s_delay.lock().await;
                    if delay > 0 {
                        tokio::time::sleep(Duration::from_millis(delay as u64)).await;
                    } else if is_snippet {
                        tokio::time::sleep(Duration::from_millis(10)).await;
                    }
                    client.send_message(&msg.path, msg.args).await?;
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

        loop {
            if !keep_on {
                break;
            }

            let mut line_buf = String::new();
            let mut handle = stdin_lock.by_ref().take(4096);
            if handle.read_line(&mut line_buf).is_err() || line_buf.is_empty() {
                break;
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
                println!(":: xremote toggle");
            } else if line == "xremote off" {
                client.stop_heartbeat();
            } else if line == "xremote on" {
                client.start_heartbeat();
            } else if !line.is_empty() {
                if let Ok(msg) = OscMessage::from_str(line) {
                    let delay = *s_delay.lock().await;
                    if delay > 0 {
                        tokio::time::sleep(Duration::from_millis(delay as u64)).await;
                    }
                    client.send_message(&msg.path, msg.args).await?;
                } else {
                    eprintln!("Failed to parse line: {}", line);
                }
            }
        }
    }

    Ok(())
}
