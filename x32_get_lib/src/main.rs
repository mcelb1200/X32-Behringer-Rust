//! `x32_get_lib` is a command-line tool for retrieving library presets from a Behringer X32/M32 mixer.
//!
//! It can fetch Channel, Effects, or Routing presets and save them to local files.
//! This tool allows you to backup your library presets or transfer them between consoles.
//!
//! # Credits
//!
//! *   **Original concept and work on the C library:** Patrick-Gilles Maillot
//! *   **Additional concepts by:** [User]
//! *   **Rust implementation by:** [User]

use anyhow::Result;
use clap::{Parser, ValueEnum};
use osc_lib::{OscArg, OscMessage};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;
use x32_lib::create_socket;

/// Command-line arguments for the `x32_get_lib` tool.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// IP address of the X32 console.
    #[arg(short, long, default_value = "192.168.0.64")]
    ip: String,

    /// Output directory for saved files.
    #[arg(short, long, default_value = ".")]
    output_dir: PathBuf,

    /// Type of library data to retrieve.
    #[arg(long, value_enum, default_value_t = LibType::All)]
    type_: LibType,

    /// Enable verbose output.
    #[arg(short, long)]
    verbose: bool,
}

/// Enumeration of library types supported by the X32.
#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
enum LibType {
    /// Channel presets.
    Channel,
    /// Effects presets.
    Effects,
    /// Routing presets.
    Routing,
    /// All types.
    All,
}

impl LibType {
    /// Returns the OSC path segment corresponding to the library type.
    fn as_str(&self) -> &'static str {
        match self {
            LibType::Channel => "ch",
            LibType::Effects => "fx",
            LibType::Routing => "r", // C code says "-libs/r/" ? C code: "-libs/r/%03d"
            LibType::All => "all",
        }
    }
    /// Returns the file extension for the library type.
    fn extension(&self) -> &'static str {
        match self {
            LibType::Channel => "chn",
            LibType::Effects => "efx",
            LibType::Routing => "rou",
            LibType::All => "",
        }
    }
}

/// The main entry point for the application.
fn main() -> Result<()> {
    let args = Args::parse();
    let socket = create_socket(&args.ip, 500)?;

    println!("Connected to X32 at {}", args.ip);

    let types = match args.type_ {
        LibType::All => vec![LibType::Channel, LibType::Effects, LibType::Routing],
        t => vec![t],
    };

    for t in types {
        println!("Processing library type: {:?}", t);
        for i in 1..=100 {
            let type_str = t.as_str();
            // Check hasdata: /-libs/{type}/{id}/hasdata
            let addr = format!("/-libs/{}/{:03}/hasdata", type_str, i);
            let msg = OscMessage::new(addr.clone(), vec![]);
            socket.send(&msg.to_bytes()?)?;

            let mut buf = [0u8; 512];
            if let Ok(len) = socket.recv(&mut buf) {
                let resp = OscMessage::from_bytes(&buf[..len])?;
                if let Some(OscArg::Int(1)) = resp.args.first() {
                    // Has data
                    process_lib_slot(&socket, t, i, &args.output_dir, args.verbose)?;
                }
            }
        }
    }

    Ok(())
}

/// Processes a single library slot, retrieving its data and saving it to a file.
///
/// # Arguments
///
/// * `socket` - The UDP socket connected to the mixer.
/// * `t` - The type of library preset.
/// * `id` - The preset ID (1-100).
/// * `out_dir` - The directory to save the file to.
/// * `verbose` - Whether to print verbose output.
fn process_lib_slot(
    socket: &std::net::UdpSocket,
    t: LibType,
    id: i32,
    out_dir: &Path,
    _verbose: bool,
) -> Result<()> {
    let type_str = t.as_str();

    // Get Node info (name)
    // /node ,s -libs/{type}/{id}
    let node_arg = format!("-libs/{}/{:03}", type_str, id);
    let msg = OscMessage::new("/node".to_string(), vec![OscArg::String(node_arg)]);
    socket.send(&msg.to_bytes()?)?;

    let mut buf = [0u8; 512];
    let len = socket.recv(&mut buf)?;
    let resp = OscMessage::from_bytes(&buf[..len])?;

    // Parse name from response: node ,s ... "Name" ...
    // The C code parses raw string.
    // Rust OscMessage args: String("-libs/ch/001"), String("Name"), ...
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

    // Write Header (from C: #2.1#...)
    // Actually, we should fetch the data.
    // Logic:
    // 1. /load ,si ... (load to edit buffer)
    // 2. Read params from edit buffer
    // 3. Write to file

    // 1. Load
    // /load ,siii "libchan" id 0 63
    let load_target = match t {
        LibType::Channel => "libchan",
        LibType::Effects => "libfx",
        LibType::Routing => "librout", // Check C code
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

    let msg = OscMessage::new("/load".to_string(), load_args);
    socket.send(&msg.to_bytes()?)?;

    // Wait for load (receive /load confirmation)
    socket.set_read_timeout(Some(Duration::from_millis(200)))?;
    if let Ok(_len) = socket.recv(&mut buf) {
        // Assume success for now
    }

    // 2. Read Params
    // The C code expects a specific header format, using #2.1# + name + flags (flags are available in the node response)
    // We didn't parse flags from node resp properly above, but the existing code wrote #2.1# "name" %000000000 1
    // We'll write the raw node response data after the #2.1# to be identical to C.

    // Get the name string and flags from the node response
    let mut flags = String::from("%000000000 1");
    if let Some(OscArg::String(s)) = resp.args.get(3) {
        // usually flag
        if let Some(OscArg::Int(i)) = resp.args.get(4) {
            flags = format!("{} {}", s, i);
        }
    }

    writeln!(file, "#2.1# \"{}\" {}", name, flags)?;

    let p_com_list = [
        "ch/01/config",
        "ch/01/delay",
        "ch/01/preamp",
        "ch/01/gate",
        "ch/01/gate/filter",
        "ch/01/dyn",
        "ch/01/dyn/filter",
        "ch/01/eq",
        "ch/01/eq/1",
        "ch/01/eq/2",
        "ch/01/eq/3",
        "ch/01/eq/4",
        "ch/01/mix",
        "ch/01/mix/01",
        "ch/01/mix/02",
        "ch/01/mix/03",
        "ch/01/mix/04",
        "ch/01/mix/05",
        "ch/01/mix/06",
        "ch/01/mix/07",
        "ch/01/mix/08",
        "ch/01/mix/09",
        "ch/01/mix/10",
        "ch/01/mix/11",
        "ch/01/mix/12",
        "ch/01/mix/13",
        "ch/01/mix/14",
        "ch/01/mix/15",
        "ch/01/mix/16",
    ];

    let p_cfx_list = ["fx/1/type", "fx/1/source", "fx/1/par"];

    let p_cro_list = [
        "config/routing/IN",
        "config/routing/AES50A",
        "config/routing/AES50B",
        "config/routing/CARD",
        "config/routing/OUT",
        "config/routing/PLAY",
        "outputs/main/01",
        "outputs/main/01/delay",
        "outputs/main/02",
        "outputs/main/02/delay",
        "outputs/main/03",
        "outputs/main/03/delay",
        "outputs/main/04",
        "outputs/main/04/delay",
        "outputs/main/05",
        "outputs/main/05/delay",
        "outputs/main/06",
        "outputs/main/06/delay",
        "outputs/main/07",
        "outputs/main/07/delay",
        "outputs/main/08",
        "outputs/main/08/delay",
        "outputs/main/09",
        "outputs/main/09/delay",
        "outputs/main/10",
        "outputs/main/10/delay",
        "outputs/main/11",
        "outputs/main/11/delay",
        "outputs/main/12",
        "outputs/main/12/delay",
        "outputs/main/13",
        "outputs/main/13/delay",
        "outputs/main/14",
        "outputs/main/14/delay",
        "outputs/main/15",
        "outputs/main/15/delay",
        "outputs/main/16",
        "outputs/main/16/delay",
        "outputs/aux/01",
        "outputs/aux/02",
        "outputs/aux/03",
        "outputs/aux/04",
        "outputs/aux/05",
        "outputs/aux/06",
        "outputs/p16/01",
        "outputs/p16/01/iQ",
        "outputs/p16/02",
        "outputs/p16/02/iQ",
        "outputs/p16/03",
        "outputs/p16/03/iQ",
        "outputs/p16/04",
        "outputs/p16/04/iQ",
        "outputs/p16/05",
        "outputs/p16/05/iQ",
        "outputs/p16/06",
        "outputs/p16/06/iQ",
        "outputs/p16/07",
        "outputs/p16/07/iQ",
        "outputs/p16/08",
        "outputs/p16/08/iQ",
        "outputs/p16/09",
        "outputs/p16/09/iQ",
        "outputs/p16/10",
        "outputs/p16/10/iQ",
        "outputs/p16/11",
        "outputs/p16/11/iQ",
        "outputs/p16/12",
        "outputs/p16/12/iQ",
        "outputs/p16/13",
        "outputs/p16/13/iQ",
        "outputs/p16/14",
        "outputs/p16/14/iQ",
        "outputs/p16/15",
        "outputs/p16/15/iQ",
        "outputs/p16/16",
        "outputs/p16/16/iQ",
        "outputs/aes/01",
        "outputs/aes/02",
    ];

    let params: &[&str] = match t {
        LibType::Channel => &p_com_list,
        LibType::Effects => &p_cfx_list,
        LibType::Routing => &p_cro_list,
        _ => &[],
    };

    socket.set_read_timeout(Some(Duration::from_millis(500)))?;

    for (i, p) in params.iter().enumerate() {
        let msg = OscMessage::new("/node".to_string(), vec![OscArg::String(p.to_string())]);
        socket.send(&msg.to_bytes()?)?;

        if let Ok(len) = socket.recv(&mut buf) {
            if let Ok(resp) = OscMessage::from_bytes(&buf[..len]) {
                if resp.path == "node" {
                    if let Some(OscArg::String(val)) = resp.args.first() {
                        let mut output = val.clone();

                        match t {
                            LibType::Channel => {
                                // Strip "/ch/01" from the beginning
                                if let Some(stripped) = output.strip_prefix("/ch/01") {
                                    output = stripped.to_string();
                                }

                                // Remove the "source" element of /config
                                if i == 0 {
                                    if let Some(last_space) = output.rfind(' ') {
                                        output.truncate(last_space);
                                    }
                                }
                                writeln!(file, "{}", output.trim_start())?;
                            }
                            LibType::Effects => {
                                // Strip "/fx/1/" from the beginning
                                if let Some(stripped) = output.strip_prefix("/fx/1/") {
                                    output = stripped.to_string();
                                }
                                writeln!(file, "{}", output.trim_start())?;
                            }
                            LibType::Routing => {
                                // The C code writes: r_buf + 12 which strips "/node...,s~~" meaning it keeps the leading '/' or just writes the value.
                                // Actually C code for routing sends: /node ,s config/routing/IN
                                // Returns: node ,s "/config/routing/IN ~~~"
                                // The C code writes it directly.
                                writeln!(file, "{}", output.trim_start())?;
                            }
                            _ => {}
                        }
                    }
                }
            }
        } else {
            eprintln!("  Error or timeout on command: /node ,s {}", p);
        }
    }

    if t == LibType::Channel {
        let msg = OscMessage::new(
            "/node".to_string(),
            vec![OscArg::String("headamp/000".to_string())],
        );
        socket.send(&msg.to_bytes()?)?;
        if let Ok(len) = socket.recv(&mut buf) {
            if let Ok(resp) = OscMessage::from_bytes(&buf[..len]) {
                if resp.path == "node" {
                    if let Some(OscArg::String(val)) = resp.args.first() {
                        writeln!(file, "{}", val)?;
                    }
                }
            }
        } else {
            eprintln!("  Error or timeout on command: /node ,s headamp/000");
        }
    }

    file.flush()?;

    Ok(())
}
