use anyhow::{Context, Result, anyhow};
use clap::Parser;
use osc_lib::{OscArg, OscMessage};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::time::Duration;
use x32_lib::create_socket;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "192.168.0.64")]
    ip: String,

    #[arg(short, long, default_value_t = 1)]
    start_index: i32,

    files: Vec<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let socket = create_socket(&args.ip, 500)?;
    let mut index = args.start_index;

    for path in args.files {
        println!("Processing file: {:?}", path);
        let file = File::open(&path)?;
        let reader = BufReader::new(file);

        // Detect type from extension
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        let (lib_type, prefix) = match ext {
            "chn" => ("libchan", "/ch/01"),
            "efx" => ("libfx", "/fx/1"),
            "rou" => ("librout", ""),
            _ => {
                eprintln!("Skipping unknown file extension: {}", ext);
                continue;
            }
        };

        let mut name = String::from("NewPreset");

        for line in reader.lines() {
            let line = line?;
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if line.starts_with('#') {
                // Parse header for name: #2.1# "Name" ...
                if let Some(start) = line.find('"') {
                    if let Some(end) = line[start + 1..].find('"') {
                        name = line[start + 1..start + 1 + end].to_string();
                    }
                }
                continue;
            }

            // Parse OSC command
            // Line format: /path arg1 arg2
            // C code output: "config ..." -> mapped to "/ch/01/config ..."

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            let addr_suffix = parts[0];
            // If addr starts with /, treat as absolute (Routing), else prepend prefix
            let full_addr = if addr_suffix.starts_with('/') {
                addr_suffix.to_string()
            } else {
                format!("{}{}", prefix, addr_suffix) // e.g. /ch/01/config
            };

            // Parse args
            let mut osc_args = Vec::new();
            for arg_str in &parts[1..] {
                if let Ok(i) = arg_str.parse::<i32>() {
                    osc_args.push(OscArg::Int(i));
                } else if let Ok(f) = arg_str.parse::<f32>() {
                    osc_args.push(OscArg::Float(f));
                } else {
                    osc_args.push(OscArg::String(arg_str.replace("\"", "")));
                }
            }

            let msg = OscMessage::new(full_addr, osc_args);
            socket.send(&msg.to_bytes()?)?;
            // Optional: sleep?
        }

        // Save to Library
        // /save ,sisi "libchan" index "Name" 0
        // Note: C code uses ,sisi.
        let save_args = if lib_type == "librout" {
            // Routing: /save ,sis "librout" index "Name"
            vec![
                OscArg::String(lib_type.to_string()),
                OscArg::Int(index - 1),
                OscArg::String(name),
            ]
        } else {
            vec![
                OscArg::String(lib_type.to_string()),
                OscArg::Int(index - 1),
                OscArg::String(name),
                OscArg::Int(0),
            ]
        };

        let msg = OscMessage::new("/save".to_string(), save_args);
        socket.send(&msg.to_bytes()?)?;

        println!("Saved to slot {}", index);
        index += 1;

        // Wait for save
        std::thread::sleep(Duration::from_millis(200));
    }

    Ok(())
}
