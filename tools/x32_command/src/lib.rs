use anyhow::{Context, Result};
use clap::Parser;
use osc_lib::OscMessage;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::Duration;
use x32_lib::{MixerClient, MixerModel};

/// Resolves target address with model-specific default ports if no port is specified in the IP string.
pub fn resolve_target_addr(ip: &str, model: MixerModel) -> String {
    if ip.contains(':') {
        ip.to_string()
    } else {
        let port = match model {
            MixerModel::X32 => 10023,
            MixerModel::Wing => 2222,
            MixerModel::XR18 | MixerModel::XR16 | MixerModel::XR12 => 10024,
        };
        format!("{}:{}", ip, port)
    }
}

/// Parses a command line for a given mixer model, validating parameters against model limits where applicable.
pub fn parse_line_for_model(line: &str, model: MixerModel) -> Option<OscMessage> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    if let Some((path, args_str)) = line.split_once(' ') {
        let args_trimmed = args_str.trim();
        if !args_trimmed.is_empty() {
            if let Some(msg) = x32_fxparse::parse_parameter(model, path, args_trimmed) {
                return Some(msg);
            }
            let is_param_path = path.starts_with("/ch/")
                || path.starts_with("/bus/")
                || path.starts_with("/auxin/")
                || path.starts_with("/fxrtn/")
                || path.starts_with("/fx/")
                || path.starts_with("/dca/")
                || path.starts_with("/mtx/")
                || path.starts_with("/main/")
                || path.starts_with("/config/");

            if is_param_path {
                return None;
            }
        }
    }

    OscMessage::from_str(line).ok()
}

/// X32_Command - a simple udp client for X32 sending commands and getting answers
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = "usage: x32_command [OPTIONS]")]
pub struct Args {
    /// X32 console ipv4 address
    #[arg(short, long)]
    pub ip: String,

    /// Mixer model (X32, Wing, XR18, XR16, XR12)
    #[arg(short = 'M', long, default_value = "X32")]
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
    let target_addr = resolve_target_addr(&args.ip, args.model);
    print!("Connecting to {} ({:?})...", target_addr, args.model);

    let client = MixerClient::connect(&target_addr, false)
        .await
        .context("Failed to connect to mixer")?;
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
                if let Some(msg) = parse_line_for_model(line, args.model) {
                    let delay = *s_delay.lock().await;
                    if delay > 0 {
                        tokio::time::sleep(Duration::from_millis(delay as u64)).await;
                    } else if is_snippet {
                        tokio::time::sleep(Duration::from_millis(10)).await;
                    }
                    client.send_message(&msg.path, msg.args).await?;
                } else {
                    eprintln!("Failed to parse line for model {:?}: {}", args.model, line);
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
                if let Some(msg) = parse_line_for_model(line, args.model) {
                    let delay = *s_delay.lock().await;
                    if delay > 0 {
                        tokio::time::sleep(Duration::from_millis(delay as u64)).await;
                    }
                    client.send_message(&msg.path, msg.args).await?;
                } else {
                    eprintln!("Failed to parse line for model {:?}: {}", args.model, line);
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
    fn test_resolve_target_addr() {
        assert_eq!(
            resolve_target_addr("192.168.1.64", MixerModel::X32),
            "192.168.1.64:10023"
        );
        assert_eq!(
            resolve_target_addr("192.168.1.64", MixerModel::Wing),
            "192.168.1.64:2222"
        );
        assert_eq!(
            resolve_target_addr("192.168.1.64", MixerModel::XR18),
            "192.168.1.64:10024"
        );
        assert_eq!(
            resolve_target_addr("192.168.1.64", MixerModel::XR16),
            "192.168.1.64:10024"
        );
        assert_eq!(
            resolve_target_addr("192.168.1.64", MixerModel::XR12),
            "192.168.1.64:10024"
        );
        assert_eq!(
            resolve_target_addr("192.168.1.64:1234", MixerModel::X32),
            "192.168.1.64:1234"
        );
    }

    #[test]
    fn test_parse_line_for_model_valid() {
        let msg_x32 = parse_line_for_model("/ch/01/mix/fader -10.0", MixerModel::X32);
        assert!(msg_x32.is_some());
        let msg = msg_x32.unwrap();
        assert_eq!(msg.path, "/ch/01/mix/fader");

        let msg_wing = parse_line_for_model("/ch/40/mix/fader 0.0", MixerModel::Wing);
        assert!(msg_wing.is_some());

        let msg_xr18 = parse_line_for_model("/ch/16/mix/fader 0.0", MixerModel::XR18);
        assert!(msg_xr18.is_some());
    }

    #[test]
    fn test_parse_line_for_model_out_of_bounds() {
        // XR12 has 12 channels max, channel 16 parameter should fail parameter validation
        let msg_xr12 = parse_line_for_model("/ch/16/mix/fader 0.0", MixerModel::XR12);
        assert!(msg_xr12.is_none());

        // XR18 has 16 channels max, channel 32 should fail
        let msg_xr18 = parse_line_for_model("/ch/32/mix/fader 0.0", MixerModel::XR18);
        assert!(msg_xr18.is_none());
    }

    #[test]
    fn test_parse_line_for_model_queries_and_system() {
        // Queries (no arguments)
        let query = parse_line_for_model("/ch/01/mix/fader", MixerModel::X32);
        assert!(query.is_some());
        assert_eq!(query.unwrap().path, "/ch/01/mix/fader");

        // System command
        let sys = parse_line_for_model("/xremote", MixerModel::XR18);
        assert!(sys.is_some());
        assert_eq!(sys.unwrap().path, "/xremote");
    }

    #[test]
    fn test_cli_args_model_parsing() {
        let args = Args::parse_from(&["x32_command", "--ip", "10.0.0.1", "-M", "Wing"]);
        assert_eq!(args.ip, "10.0.0.1");
        assert_eq!(args.model, MixerModel::Wing);

        let args_xr18 = Args::parse_from(&["x32_command", "--ip", "10.0.0.2", "--model", "XR18"]);
        assert_eq!(args_xr18.model, MixerModel::XR18);
    }
}
