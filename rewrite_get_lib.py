import re

with open("tools/x32_get_lib/src/main.rs", "r") as f:
    text = f.read()

out = """//! `x32_get_lib` is a command-line tool for retrieving library presets from a Behringer X32/M32 mixer.
//!
//! It can fetch Channel, Effects, or Routing presets and save them to local files.
//! This tool allows you to backup your library presets or transfer them between consoles.

use anyhow::Result;
use clap::{Parser, ValueEnum};
use osc_lib::{OscArg, OscMessage};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use tokio::time::{timeout, Duration};
use x32_lib::{MixerClient, error::X32Error};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "192.168.0.64")]
    ip: String,

    #[arg(long, default_value = "auto")]
    transport: String,

    #[arg(long, default_value = "")]
    usb_port: String,

    #[arg(long, default_value = "")]
    aes50_ip: String,

    #[arg(short, long, default_value = ".")]
    output_dir: PathBuf,

    #[arg(short, long, value_enum, default_value_t = LibType::All)]
    type_: LibType,

    #[arg(short, long)]
    verbose: bool,
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
enum LibType {
    Channel,
    Effects,
    Routing,
    All,
}

impl LibType {
    fn as_str(&self) -> &'static str {
        match self {
            LibType::Channel => "ch",
            LibType::Effects => "fx",
            LibType::Routing => "r",
            LibType::All => "all",
        }
    }
    fn extension(&self) -> &'static str {
        match self {
            LibType::Channel => "chn",
            LibType::Effects => "efx",
            LibType::Routing => "rou",
            LibType::All => "",
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    let (client, _) = MixerClient::connect_with_transport(
        &args.ip,
        &args.aes50_ip,
        &args.usb_port,
        &args.transport,
        false,
    ).await?;
    let client = std::sync::Arc::new(client);

    println!("Connected to X32 at {}", args.ip);

    let types = match args.type_ {
        LibType::All => vec![LibType::Channel, LibType::Effects, LibType::Routing],
        t => vec![t],
    };

    let mut rx = client.subscribe();

    for t in types {
        println!("Processing library type: {:?}", t);
        for i in 1..=100 {
            let type_str = t.as_str();
            let addr = format!("/-libs/{}/{:03}/hasdata", type_str, i);
            client.send_message(&addr, vec![]).await?;

            if let Ok(Ok(resp)) = timeout(Duration::from_millis(50), rx.recv()).await {
                if let Some(OscArg::Int(1)) = resp.args.first() {
                    process_lib_slot(&client, t, i, &args.output_dir, args.verbose).await?;
                }
            }
        }
    }

    Ok(())
}

async fn process_lib_slot(
    client: &MixerClient,
    t: LibType,
    id: i32,
    out_dir: &Path,
    _verbose: bool,
) -> Result<()> {
    let type_str = t.as_str();

    let mut rx = client.subscribe();
    let node_arg = format!("-libs/{}/{:03}", type_str, id);
    client.send_message("/node", vec![OscArg::String(node_arg)]).await?;

    let resp = match timeout(Duration::from_millis(500), rx.recv()).await {
        Ok(Ok(m)) => m,
        _ => return Err(X32Error::from("Timeout waiting for node").into()),
    };

    let name = if let Some(OscArg::String(s)) = resp.args.get(1) {
        s.clone()
    } else {
        format!("Preset_{:03}", id)
    };

    println!("  Found preset {}: {}", id, name);

    let filename = format!("{}.{}", name, t.extension());
    let path = out_dir.join(filename);
    let file = File::create(&path)?;
    let mut file = BufWriter::new(file);

    let load_target = match t {
        LibType::Channel => "libchan",
        LibType::Effects => "libfx",
        LibType::Routing => "librout",
        _ => return Ok(()),
    };

    let load_args = match t {
        LibType::Channel => vec![
            OscArg::String(load_target.to_string()),
            OscArg::Int(id - 1),
            OscArg::Int(0),
            OscArg::Int(63),
        ],
        LibType::Effects => vec![
            OscArg::String(load_target.to_string()),
            OscArg::Int(id - 1),
            OscArg::Int(0),
        ],
        LibType::Routing => vec![OscArg::String(load_target.to_string()), OscArg::Int(id - 1)],
        _ => vec![],
    };

    client.send_message("/load", load_args).await?;
    let _ = timeout(Duration::from_millis(200), rx.recv()).await;

    let mut flags = String::from("%000000000 1");
    if let Some(OscArg::String(s)) = resp.args.get(3) {
        if let Some(OscArg::Int(i)) = resp.args.get(4) {
            flags = format!("{} {}", s, i);
        }
    }

    writeln!(file, "#2.1# \\\"{}\\\" {}", name, flags)?;

    let params: Vec<String> = match t {
        LibType::Channel => {
            let mut p = vec![
                "ch/01/config".to_string(),
                "ch/01/delay".to_string(),
                "ch/01/preamp".to_string(),
                "ch/01/gate".to_string(),
                "ch/01/gate/filter".to_string(),
                "ch/01/dyn".to_string(),
                "ch/01/dyn/filter".to_string(),
                "ch/01/eq".to_string(),
            ];
            p.extend((1..=4).map(|i| format!("ch/01/eq/{}", i)));
            p.push("ch/01/mix".to_string());
            p.extend((1..=16).map(|i| format!("ch/01/mix/{:02}", i)));
            p
        }
        LibType::Effects => vec![
            "fx/1/type".to_string(),
            "fx/1/source".to_string(),
            "fx/1/par".to_string(),
        ],
        LibType::Routing => {
            let mut p = vec![
                "config/routing/IN".to_string(),
                "config/routing/AES50A".to_string(),
                "config/routing/AES50B".to_string(),
                "config/routing/CARD".to_string(),
                "config/routing/OUT".to_string(),
                "config/routing/PLAY".to_string(),
            ];
            p.extend((1..=16).flat_map(|i| {
                vec![
                    format!("outputs/main/{:02}", i),
                    format!("outputs/main/{:02}/delay", i),
                ]
            }));
            p.extend((1..=6).map(|i| format!("outputs/aux/{:02}", i)));
            p.extend((1..=16).flat_map(|i| {
                vec![
                    format!("outputs/p16/{:02}", i),
                    format!("outputs/p16/{:02}/iQ", i),
                ]
            }));
            p.extend((1..=2).map(|i| format!("outputs/aes/{:02}", i)));
            p
        }
        _ => vec![],
    };

    for (i, p) in params.iter().enumerate() {
        client.send_message("/node", vec![OscArg::String(p.to_string())]).await?;

        if let Ok(Ok(resp)) = timeout(Duration::from_millis(500), rx.recv()).await {
            if resp.path == "/node" || resp.path == "node" {
                if let Some(OscArg::String(val)) = resp.args.first() {
                    let mut output = val.clone();

                    match t {
                        LibType::Channel => {
                            if let Some(stripped) = output.strip_prefix("ch/01").or_else(|| output.strip_prefix("/ch/01")) {
                                output = stripped.to_string();
                            }
                            if i == 0 {
                                if let Some(last_space) = output.rfind(' ') {
                                    output.truncate(last_space);
                                }
                            }
                            writeln!(file, "{}", output.trim_start())?;
                        }
                        LibType::Effects => {
                            if let Some(stripped) = output.strip_prefix("fx/1/").or_else(|| output.strip_prefix("/fx/1/")) {
                                output = stripped.to_string();
                            }
                            writeln!(file, "{}", output.trim_start())?;
                        }
                        LibType::Routing => {
                            writeln!(file, "{}", output.trim_start())?;
                        }
                        _ => {}
                    }
                }
            }
        } else {
            eprintln!("  Error or timeout on command: /node ,s {}", p);
        }
    }

    if t == LibType::Channel {
        client.send_message("/node", vec![OscArg::String("headamp/000".to_string())]).await?;
        if let Ok(Ok(resp)) = timeout(Duration::from_millis(500), rx.recv()).await {
            if resp.path == "/node" || resp.path == "node" {
                if let Some(OscArg::String(val)) = resp.args.first() {
                    writeln!(file, "{}", val)?;
                }
            }
        } else {
            eprintln!("  Error or timeout on command: /node ,s headamp/000");
        }
    }

    file.flush()?;
    Ok(())
}
"""

with open("tools/x32_get_lib/src/main.rs", "w") as f:
    f.write(out)
