use anyhow::{Context, Result};
use clap::Parser;
use osc_lib::OscMessage;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::Duration;
use x32_lib::{MixerClient, MixerModel};

/// X32_Command - a simple udp client for X32 sending commands and getting answers
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = "usage: x32_command [OPTIONS]")]
pub struct Args {
    /// X32 console ipv4 address
    #[arg(short, long)]
    pub ip: String,

    /// Mixer model: X32, Wing, XR18, XR16, XR12
    #[arg(
        short = 'M',
        long,
        default_value = "X32",
        help = "Mixer model: X32, Wing, XR18, XR16, XR12"
    )]
    pub model: MixerModel,

    /// debug option (0/1)
    #[arg(short, long, default_value_t = 0)]
    pub debug: u8,

    /// verbose option (0/1)
    #[arg(short, long, default_value_t = 1)]
    pub verbose: u8,

    /// keyboard mode on (0/1)
    #[arg(short, long, default_value_t = 1)]
    pub keyboard: u8,

    /// delay between batch commands in ms
    #[arg(short, long, default_value_t = 10)]
    pub time: u32,

    /// sets batch mode on, getting input data from 'file'
    #[arg(short, long)]
    pub file: Option<String>,

    /// sets batch mode on, getting input data from snippets/tidbits/X32node 'file'
    #[arg(short, long)]
    pub snippet: Option<String>,
}

pub async fn run(args: Args) -> Result<()> {
    println!(" X32_Command - Rust Rewrite - (c)2014-20 Patrick-Gilles Maillot");
    let target = args.model.resolve_target_address(&args.ip);
    print!("Connecting to {} ({})...", args.model, target);

    let client = MixerClient::connect(&target, false)
        .await
        .with_context(|| format!("Failed to connect to {}", target))?;
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
        use std::io::BufRead;

        let file =
            File::open(file_path).with_context(|| format!("Cannot read file: {}", file_path))?;
        if file.metadata()?.len() > 1024 * 1024 {
            return Err(anyhow::anyhow!("File too large"));
        }
        use std::io::Read;
        let mut content = String::new();
        file.take(1024 * 1024 + 1).read_to_string(&mut content)?;
        if content.len() > 1024 * 1024 {
            return Err(anyhow::anyhow!("File too large"));
        }
        let reader = std::io::Cursor::new(content);

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
                println!(":: xremote toggle");
            } else if line == "xremote off" {
                client.stop_heartbeat();
            } else if line == "xremote on" {
                client.start_heartbeat();
            } else if !line.is_empty() {
                if let Ok(msg) = OscMessage::from_str(line) {
                    if !args.model.is_path_valid(&msg.path) {
                        eprintln!(
                            "Warning: path '{}' is out of bounds for model {}",
                            msg.path, args.model
                        );
                        continue;
                    }
                    let delay = *s_delay.lock().await;
                    if delay > 0 {
                        tokio::time::sleep(Duration::from_millis(delay as u64)).await;
                    } else if is_snippet {
                        tokio::time::sleep(Duration::from_millis(10)).await;
                    }
                    client.send(msg).await?;
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

            let mut byte_buf = Vec::new();
            let mut handle = stdin_lock.by_ref().take(4096);
            match handle.read_until(b'\n', &mut byte_buf) {
                Ok(0) => break, // EOF
                Err(e) => {
                    eprintln!("Error reading input: {}", e);
                    break;
                }
                Ok(len) => {
                    if len == 4096 && !byte_buf.ends_with(b"\n") {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Input line too long",
                        )
                        .into());
                    }
                }
            }

            let line_str = match std::str::from_utf8(&byte_buf) {
                Ok(s) => s,
                Err(_) => {
                    eprintln!("Invalid UTF-8 sequence in input, discarded.");
                    continue;
                }
            };

            let line = line_str.trim();

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
                    if !args.model.is_path_valid(&msg.path) {
                        eprintln!(
                            "Warning: path '{}' is out of bounds for model {}",
                            msg.path, args.model
                        );
                        continue;
                    }
                    let delay = *s_delay.lock().await;
                    if delay > 0 {
                        tokio::time::sleep(Duration::from_millis(delay as u64)).await;
                    }
                    client.send(msg).await?;
                } else {
                    eprintln!("Failed to parse line: {}", line);
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_default_model() {
        let args = Args::try_parse_from(["x32_command", "-i", "192.168.1.10"]).unwrap();
        assert_eq!(args.model, MixerModel::X32);
        assert_eq!(
            args.model.resolve_target_address(&args.ip),
            "192.168.1.10:10023"
        );
    }

    #[test]
    fn test_args_wing_model() {
        let args =
            Args::try_parse_from(["x32_command", "-i", "192.168.1.10", "-M", "Wing"]).unwrap();
        assert_eq!(args.model, MixerModel::Wing);
        assert_eq!(
            args.model.resolve_target_address(&args.ip),
            "192.168.1.10:2222"
        );
    }

    #[test]
    fn test_args_xr_models() {
        let args_xr18 =
            Args::try_parse_from(["x32_command", "-i", "10.0.0.5", "--model", "XR18"]).unwrap();
        assert_eq!(args_xr18.model, MixerModel::XR18);
        assert_eq!(
            args_xr18.model.resolve_target_address(&args_xr18.ip),
            "10.0.0.5:10024"
        );

        let args_xr12 =
            Args::try_parse_from(["x32_command", "-i", "10.0.0.5:9999", "-M", "XR12"]).unwrap();
        assert_eq!(args_xr12.model, MixerModel::XR12);
        // Explicit port is preserved
        assert_eq!(
            args_xr12.model.resolve_target_address(&args_xr12.ip),
            "10.0.0.5:9999"
        );
    }
}
