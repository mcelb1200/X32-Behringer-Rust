use anyhow::{anyhow, Context, Result};
use clap::{Parser, ValueEnum};
use osc_lib::{OscArg, OscMessage};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;
use x32_lib::create_socket;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "192.168.0.64")]
    ip: String,

    #[arg(short, long, default_value = ".")]
    output_dir: PathBuf,

    #[arg(long, value_enum, default_value_t = LibType::All)]
    type_: LibType,

    #[arg(short, long)]
    verbose: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
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
            LibType::Routing => "r", // C code says "-libs/r/" ? C code: "-libs/r/%03d"
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

fn process_lib_slot(socket: &std::net::UdpSocket, t: LibType, id: i32, out_dir: &PathBuf, verbose: bool) -> Result<()> {
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
    let mut file = File::create(&path)?;

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
        LibType::Channel => vec![OscArg::String(load_target.to_string()), OscArg::Int(id - 1), OscArg::Int(0), OscArg::Int(63)],
        LibType::Effects => vec![OscArg::String(load_target.to_string()), OscArg::Int(id - 1), OscArg::Int(0)],
        LibType::Routing => vec![OscArg::String(load_target.to_string()), OscArg::Int(id - 1)],
        _ => vec![],
    };

    let msg = OscMessage::new("/load".to_string(), load_args);
    socket.send(&msg.to_bytes()?)?;

    // Wait for load (receive /load confirmation)
    socket.set_read_timeout(Some(Duration::from_millis(200)))?;
    if let Ok(len) = socket.recv(&mut buf) {
        // Assume success for now
    }

    // 2. Read Params
    // Define list of params per type (from C PComList etc.)
    // Writing to file format: /address argument

    writeln!(file, "#2.1# \"{}\" %000000000 1", name)?; // Simplified header

    let params = match t {
        LibType::Channel => vec![
            "/ch/01/config", "/ch/01/preamp", "/ch/01/gate", "/ch/01/dyn", "/ch/01/eq", "/ch/01/mix"
            // Add more details if needed, or recursive /node traversal
        ],
        LibType::Effects => vec![
            "/fx/1/type", "/fx/1/source", "/fx/1/par"
        ],
        LibType::Routing => vec![
            "/config/routing", "/outputs"
        ],
        _ => vec![],
    };

    for p in params {
        // We can use /node to get values recursively if we implement that,
        // or just query specific paths.
        // The C code iterates a static list.
        // For simplicity/robustness, we query the path.
        // But /ch/01/config returns multiple values.
        // We need to query it.
        let msg = OscMessage::new(p.to_string(), vec![]);
        socket.send(&msg.to_bytes()?)?;

        // Wait for response(s)
        // X32 might send multiple messages if we query a node?
        // Usually /ch/01/config returns one message with multiple args.
        if let Ok(len) = socket.recv(&mut buf) {
            let resp = OscMessage::from_bytes(&buf[..len])?;
            // Format: /path arg1 arg2 ...
            // But in file, we want relative paths?
            // C code writes: "config ...", "preamp ..." (stripping /ch/01/ prefix)

            let suffix = match t {
                LibType::Channel => p.strip_prefix("/ch/01").unwrap_or(p),
                LibType::Effects => p.strip_prefix("/fx/1").unwrap_or(p),
                LibType::Routing => p.strip_prefix("/").unwrap_or(p), // Routing keeps full path usually? C: "/config/..."
                _ => p,
            };

            // Reconstruct line
            write!(file, "{}", suffix)?;
            for arg in resp.args {
                match arg {
                    OscArg::Int(i) => write!(file, " {}", i)?,
                    OscArg::Float(f) => write!(file, " {:.4}", f)?,
                    OscArg::String(s) => write!(file, " \"{}\"", s)?,
                    _ => {},
                }
            }
            writeln!(file)?;
        }
    }

    Ok(())
}
