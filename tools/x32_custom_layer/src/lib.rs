//! `x32_custom_layer` is a command-line tool for creating and managing custom channel layers on X32/M32 mixers.
//!
//! It provides functionality to:
//! - **Set**: Assign any source channel (1-32 or Aux 1-8) to any destination channel strip.
//! - **Save**: Save the current custom layer configuration to a file.
//! - **Restore**: Restore a previously saved custom layer.
//! - **Reset**: Reset specific channels to their default "1:1" mapping (e.g., channel 1 source is input 1).
//! - **List**: Display the current source assignments for all channels.
//!
//! # Credits
//!
//! *   **Original concept and work on the C library:** Patrick-Gilles Maillot
//! *   **Additional concepts by:** mcelb1200
//! *   **Rust implementation by:** mcelb1200

use clap::{Parser, Subcommand};
use osc_lib::{OscArg, OscMessage};
use std::collections::HashMap;
use std::fmt::Write as FmtWrite;
use std::fs::File;
use std::io::{BufRead, BufWriter, Read, Write};
use std::str::FromStr;
use tokio::time::{Duration, timeout};
use x32_lib::{
    MixerClient,
    error::{Result, X32Error},
};

/// Header for the custom layer snippet file.
const SNIP_HEAD: &str = "#2.1# \"CustLayer\" 8191 -1 255 0 1\n";

/// OSC nodes to query for a standard channel (1-32).
const SCH_NODES: [&str; 35] = [
    "/headamp/000",
    "/ch/01/config",
    "/ch/01/delay",
    "/ch/01/eq",
    "/ch/01/eq/1",
    "/ch/01/eq/2",
    "/ch/01/eq/3",
    "/ch/01/eq/4",
    "/ch/01/gate",
    "/ch/01/gate/filter",
    "/ch/01/dyn",
    "/ch/01/dyn/filter",
    "/ch/01/insert",
    "/ch/01/mix",
    "/ch/01/mix/01",
    "/ch/01/mix/02",
    "/ch/01/mix/03",
    "/ch/01/mix/04",
    "/ch/01/mix/05",
    "/ch/01/mix/06",
    "/ch/01/mix/07",
    "/ch/01/mix/08",
    "/ch/01/mix/09",
    "/ch/01/mix/10",
    "/ch/01/mix/11",
    "/ch/01/mix/12",
    "/ch/01/mix/13",
    "/ch/01/mix/14",
    "/ch/01/mix/15",
    "/ch/01/mix/16",
    "/ch/01/mix/m",
    "/ch/01/grp",
    "/ch/01/grp/dca",
    "/ch/01/grp/mute",
    "/ch/01/preamp",
];

/// OSC nodes to query for an Aux channel (1-8).
const ACH_NODES: [&str; 29] = [
    "/headamp/000",
    "/auxin/01/config",
    "/auxin/01/eq",
    "/auxin/01/eq/1",
    "/auxin/01/eq/2",
    "/auxin/01/eq/3",
    "/auxin/01/eq/4",
    "/auxin/01/mix",
    "/auxin/01/mix/01",
    "/auxin/01/mix/02",
    "/auxin/01/mix/03",
    "/auxin/01/mix/04",
    "/auxin/01/mix/05",
    "/auxin/01/mix/06",
    "/auxin/01/mix/07",
    "/auxin/01/mix/08",
    "/auxin/01/mix/09",
    "/auxin/01/mix/10",
    "/auxin/01/mix/11",
    "/auxin/01/mix/12",
    "/auxin/01/mix/13",
    "/auxin/01/mix/14",
    "/auxin/01/mix/15",
    "/auxin/01/mix/16",
    "/auxin/01/mix/m",
    "/auxin/01/grp",
    "/auxin/01/grp/dca",
    "/auxin/01/grp/mute",
    "/auxin/01/preamp",
];

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long, default_value = "192.168.0.64")]
    pub ip: String,

    #[arg(long, default_value = "auto")]
    pub transport: String,

    #[arg(long, default_value = "")]
    pub usb_port: String,

    #[arg(long, default_value = "")]
    pub aes50_ip: String,

    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Set { assignments: Vec<String> },
    Save { file: String },
    Restore { file: String },
    Reset { channels: String },
    List,
}

struct Assignment {
    dest: u8,
    src: u8,
}

fn parse_assignments(assignments_str: &[String]) -> Result<Vec<Assignment>> {
    let mut assignments = Vec::new();
    for a in assignments_str {
        let parts: Vec<&str> = a.split(':').collect();
        if parts.len() != 2 {
            return Err(X32Error::Custom(format!(
                "Invalid assignment format: {}",
                a
            )));
        }
        let dest = u8::from_str(parts[0])
            .map_err(|_| X32Error::Custom(format!("Invalid destination channel: {}", parts[0])))?;
        let src = u8::from_str(parts[1])
            .map_err(|_| X32Error::Custom(format!("Invalid source channel: {}", parts[1])))?;

        if dest == 0 || dest > 40 {
            return Err(X32Error::Custom(format!(
                "Destination channel {} out of range (1-40)",
                dest
            )));
        }
        if src == 0 || src > 40 {
            return Err(X32Error::Custom(format!(
                "Source channel {} out of range (1-40)",
                src
            )));
        }
        assignments.push(Assignment { dest, src });
    }
    Ok(assignments)
}

pub async fn run(cli: Cli) -> anyhow::Result<()> {
    let (client, _) = match MixerClient::connect_with_transport(
        &cli.ip,
        &cli.aes50_ip,
        &cli.usb_port,
        &cli.transport,
        false,
    )
    .await
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error connecting: {}", e);
            std::process::exit(1);
        }
    };

    let result = match &cli.command {
        Commands::Set { assignments } => handle_set_command(&client, assignments).await,
        Commands::Save { file } => handle_save_command(&client, file).await,
        Commands::Restore { file } => handle_restore_command(&client, file).await,
        Commands::Reset { channels } => handle_reset_command(&client, channels).await,
        Commands::List => handle_list_command(&client).await,
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}

async fn handle_set_command(client: &MixerClient, assignments_str: &[String]) -> Result<()> {
    let assignments = parse_assignments(assignments_str)?;
    let mut saved_strips: HashMap<u8, Vec<String>> = HashMap::new();

    println!("Saving states of source channels...");
    for a in &assignments {
        if let std::collections::hash_map::Entry::Vacant(e) = saved_strips.entry(a.src) {
            // ⚡ Bolt: Pre-allocate capacity for strip data vector to avoid N dynamic reallocations
            let mut strip_data = if a.src <= 32 {
                Vec::with_capacity(SCH_NODES.len())
            } else {
                Vec::with_capacity(ACH_NODES.len())
            };
            if a.src <= 32 {
                // ⚡ Bolt: Hoist string formatting outside the node loop to prevent O(N) allocations
                let src_repl = format!("/{:02}/", a.src);
                for &node in SCH_NODES.iter() {
                    let formatted_node = node.replace("/01/", &src_repl);
                    strip_data.push(get_node_state(client, &formatted_node).await?);
                }
            } else {
                // ⚡ Bolt: Hoist string formatting outside the node loop to prevent O(N) allocations
                let src_repl = format!("/{:02}/", a.src - 32);
                for &node in ACH_NODES.iter() {
                    let formatted_node = node.replace("/01/", &src_repl);
                    strip_data.push(get_node_state(client, &formatted_node).await?);
                }
            }
            e.insert(strip_data);
        }
    }

    println!("Applying states to destination channels...");
    for a in &assignments {
        let strip_data = saved_strips.get(&a.src).unwrap();
        if a.dest <= 32 {
            // ⚡ Bolt: Hoist string formatting outside the node loop to prevent O(N) allocations
            let dest_repl = format!("/{:02}/", a.dest);
            for (i, &node) in SCH_NODES.iter().enumerate() {
                let dest_node = node.replace("/01/", &dest_repl);
                let mut state_to_apply = strip_data[i].clone();
                if let Some(pos) = state_to_apply.find(' ') {
                    state_to_apply.replace_range(..pos, &dest_node);
                }

                let msg = OscMessage::from_str(&state_to_apply)?;
                client.send_message(&msg.path, msg.args).await?;
            }
            let config_node = format!("/ch/{:02}/config", a.dest);
            let config_val = if a.src <= 32 {
                a.src - 1
            } else {
                a.src - 33 + 32
            };
            client
                .send_message(
                    &config_node,
                    vec![
                        OscArg::String(format!("C{:02}", a.src)),
                        OscArg::Int(22), // custom color
                        OscArg::Int(config_val as i32),
                        OscArg::Int(0),
                    ],
                )
                .await?;
        } else {
            // ⚡ Bolt: Hoist string formatting outside the node loop to prevent O(N) allocations
            let dest_repl = format!("/{:02}/", a.dest - 32);
            for (i, &node) in ACH_NODES.iter().enumerate() {
                let dest_node = node.replace("/01/", &dest_repl);
                let mut state_to_apply = strip_data[i].clone();
                if let Some(pos) = state_to_apply.find(' ') {
                    state_to_apply.replace_range(..pos, &dest_node);
                }
                let msg = OscMessage::from_str(&state_to_apply)?;
                client.send_message(&msg.path, msg.args).await?;
            }
            let config_node = format!("/auxin/{:02}/config", a.dest - 32);
            let config_val = if a.src <= 32 {
                a.src - 1
            } else {
                a.src - 33 + 32
            };
            client
                .send_message(
                    &config_node,
                    vec![
                        OscArg::String(format!("A{:02}", a.src - 32)),
                        OscArg::Int(22), // custom color
                        OscArg::Int(config_val as i32),
                        OscArg::Int(0),
                    ],
                )
                .await?;
        }
    }
    println!("Set command completed.");
    Ok(())
}

async fn handle_save_command(client: &MixerClient, file_path: &str) -> Result<()> {
    let file = File::create(file_path)?;
    let mut writer = BufWriter::new(file);

    writer.write_all(SNIP_HEAD.as_bytes())?;

    for i in 1..=32 {
        // ⚡ Bolt: Hoist string formatting outside the node loop to prevent O(N) allocations
        let src_repl = format!("/{:02}/", i);
        for &node in SCH_NODES.iter() {
            let formatted_node = node.replace("/01/", &src_repl);
            let line = get_node_state(client, &formatted_node).await?;
            writer.write_all(line.as_bytes())?;
            writer.write_all(b"\n")?;
        }
    }
    for i in 1..=8 {
        // ⚡ Bolt: Hoist string formatting outside the node loop to prevent O(N) allocations
        let src_repl = format!("/{:02}/", i);
        for &node in ACH_NODES.iter() {
            let formatted_node = node.replace("/01/", &src_repl);
            let line = get_node_state(client, &formatted_node).await?;
            writer.write_all(line.as_bytes())?;
            writer.write_all(b"\n")?;
        }
    }
    println!("Layer saved to {}", file_path);
    Ok(())
}

fn format_node_state(args: &[OscArg]) -> Result<String> {
    if let Some(OscArg::String(node)) = args.first() {
        let mut result = node.clone();
        // ⚡ Bolt: Use `write!` to append directly into the pre-allocated `result` buffer.
        // This avoids creating intermediate, dynamically allocated `String` objects
        // inside the loop, preventing costly memory allocations on the hot path.
        for arg in args.iter().skip(1) {
            match arg {
                OscArg::Float(f) => {
                    write!(result, " {:.4}", f).unwrap();
                }
                OscArg::Int(i) => {
                    write!(result, " {}", i).unwrap();
                }
                OscArg::String(s) => {
                    write!(result, " \"{}\"", s).unwrap();
                }
                OscArg::Blob(b) => {
                    // C format: loop through bytes, print as %02x
                    write!(result, " ").unwrap();
                    for &byte in b {
                        write!(result, "{:02x}", byte).unwrap();
                    }
                }
            }
        }
        Ok(result)
    } else {
        Err(X32Error::Custom("Unexpected node state format".to_string()))
    }
}

async fn get_node_state(client: &MixerClient, node: &str) -> Result<String> {
    let mut rx = client.subscribe();
    client
        .send_message("/node", vec![OscArg::String(node.to_string())])
        .await?;
    let start = std::time::Instant::now();
    let timeout_dur = Duration::from_secs(2);
    while start.elapsed() < timeout_dur {
        if let Ok(Ok(msg)) = timeout(timeout_dur - start.elapsed(), rx.recv()).await {
            if msg.path == "/node" {
                if let Some(OscArg::String(response_node)) = msg.args.first() {
                    if response_node == node {
                        return format_node_state(&msg.args);
                    }
                }
            }
        }
    }
    Err(X32Error::Custom(format!(
        "Timeout waiting for node {}",
        node
    )))
}

async fn handle_restore_command(client: &MixerClient, file_path: &str) -> Result<()> {
    let file = File::open(file_path)?;

    if file.metadata()?.len() > 1024 * 1024 {
        return Err(X32Error::Custom("File too large".to_string()));
    }

    let mut content = String::new();
    file.take(1024 * 1024 + 1).read_to_string(&mut content)?;
    if content.len() > 1024 * 1024 {
        return Err(X32Error::Custom("File too large".to_string()));
    }
    let mut reader = std::io::Cursor::new(content);

    println!("Restoring configuration from {}...", file_path);

    let mut rx = client.subscribe();

    loop {
        let mut byte_buf = Vec::new();
        match reader.by_ref().take(4096).read_until(b'\n', &mut byte_buf) {
            Ok(0) => break,
            Err(e) => return Err(e.into()),
            Ok(len) => {
                if len == 4096 && !byte_buf.ends_with(b"\n") {
                    let mut discard = Vec::with_capacity(1024);
                    loop {
                        discard.clear();
                        match reader.by_ref().take(1024).read_until(b'\n', &mut discard) {
                            Ok(0) => break,
                            Err(e) => return Err(e.into()),
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
            }
        }

        let line_str = match std::str::from_utf8(&byte_buf) {
            Ok(s) => s,
            Err(_) => {
                eprintln!("Invalid UTF-8 sequence in file, discarded.");
                continue;
            }
        };

        let line = line_str.trim();
        if line.starts_with('/') {
            let msg = OscMessage::from_str(line)?;
            client.send_message(&msg.path, msg.args).await?;
            let _ = timeout(Duration::from_millis(5), rx.recv()).await;
        }
    }
    println!("Restore completed.");
    Ok(())
}

async fn handle_reset_command(client: &MixerClient, channels_str: &str) -> Result<()> {
    let channels_to_reset = parse_channel_range(channels_str)?;
    let mut rx = client.subscribe();

    for &ch in &channels_to_reset {
        let config_node = if ch <= 32 {
            format!("/ch/{:02}/config", ch)
        } else {
            format!("/auxin/{:02}/config", ch - 32)
        };

        let src_val = if ch <= 32 {
            ch as i32 - 1
        } else {
            ch as i32 - 33 + 32
        };

        client
            .send_message(
                &config_node,
                vec![
                    OscArg::String(format!(
                        "{}{:02}",
                        if ch <= 32 { "CH" } else { "A" },
                        if ch <= 32 { ch } else { ch - 32 }
                    )),
                    OscArg::Int(1), // default color
                    OscArg::Int(src_val),
                    OscArg::Int(0),
                ],
            )
            .await?;

        let _ = timeout(Duration::from_millis(5), rx.recv()).await;
    }
    println!("Reset completed for channels: {:?}", channels_to_reset);
    Ok(())
}

fn parse_channel_range(range_str: &str) -> Result<Vec<u8>> {
    let mut channels = Vec::new();
    for part in range_str.split(',') {
        let part = part.trim();
        if let Some(pos) = part.find('-') {
            let start = u8::from_str(&part[..pos]).map_err(|_| {
                X32Error::Custom(format!("Invalid start channel: {}", &part[..pos]))
            })?;
            let end = u8::from_str(&part[pos + 1..]).map_err(|_| {
                X32Error::Custom(format!("Invalid end channel: {}", &part[pos + 1..]))
            })?;
            if start > end || start == 0 || end > 40 {
                return Err(X32Error::Custom(format!("Invalid range: {}", part)));
            }
            for i in start..=end {
                channels.push(i);
            }
        } else {
            let ch = u8::from_str(part)
                .map_err(|_| X32Error::Custom(format!("Invalid channel: {}", part)))?;
            if ch == 0 || ch > 40 {
                return Err(X32Error::Custom(format!("Channel {} out of range", ch)));
            }
            channels.push(ch);
        }
    }
    channels.sort_unstable();
    channels.dedup();
    Ok(channels)
}

async fn handle_list_command(client: &MixerClient) -> Result<()> {
    println!("Current Channel Assignments:");
    println!("----------------------------");
    for i in 1..=32 {
        let src = get_source_name(client, i).await?;
        println!("CH{:02} <- {}", i, src);
    }
    for i in 1..=8 {
        let src = get_source_name(client, i + 32).await?;
        println!("AUX{:02} <- {}", i, src);
    }
    Ok(())
}

async fn get_source_name(client: &MixerClient, channel: u8) -> Result<String> {
    let expected_response_prefix = if channel <= 32 {
        format!("/ch/{:02}/config", channel)
    } else {
        format!("/auxin/{:02}/config", channel - 32)
    };

    let mut rx = client.subscribe();
    client
        .send_message(&expected_response_prefix, vec![])
        .await?;

    let start = std::time::Instant::now();
    let timeout_dur = Duration::from_secs(2);
    while start.elapsed() < timeout_dur {
        if let Ok(Ok(msg)) = timeout(timeout_dur - start.elapsed(), rx.recv()).await {
            if msg.path.starts_with(&expected_response_prefix) {
                if let Some(OscArg::Int(source_id)) = msg.args.get(2) {
                    return Ok(map_source_id_to_name(*source_id).to_string());
                }
            }
        }
    }
    Err(X32Error::Custom(
        "Timeout waiting for source config".to_string(),
    ))
}

fn map_source_id_to_name(id: i32) -> &'static str {
    match id {
        0..=31 => {
            const CH_NAMES: [&str; 32] = [
                "IN01", "IN02", "IN03", "IN04", "IN05", "IN06", "IN07", "IN08", "IN09", "IN10",
                "IN11", "IN12", "IN13", "IN14", "IN15", "IN16", "IN17", "IN18", "IN19", "IN20",
                "IN21", "IN22", "IN23", "IN24", "IN25", "IN26", "IN27", "IN28", "IN29", "IN30",
                "IN31", "IN32",
            ];
            CH_NAMES[id as usize]
        }
        32..=39 => {
            const AUX_NAMES: [&str; 8] = [
                "AUX1", "AUX2", "AUX3", "AUX4", "AUX5", "AUX6", "AUX7", "AUX8",
            ];
            AUX_NAMES[(id - 32) as usize]
        }
        40..=55 => {
            const FX_NAMES: [&str; 16] = [
                "FX1L", "FX1R", "FX2L", "FX2R", "FX3L", "FX3R", "FX4L", "FX4R", "FX5L", "FX5R",
                "FX6L", "FX6R", "FX7L", "FX7R", "FX8L", "FX8R",
            ];
            FX_NAMES[(id - 40) as usize]
        }
        56..=71 => {
            const BUS_NAMES: [&str; 16] = [
                "BUS01", "BUS02", "BUS03", "BUS04", "BUS05", "BUS06", "BUS07", "BUS08", "BUS09",
                "BUS10", "BUS11", "BUS12", "BUS13", "BUS14", "BUS15", "BUS16",
            ];
            BUS_NAMES[(id - 56) as usize]
        }
        _ => "OFF",
    }
}
