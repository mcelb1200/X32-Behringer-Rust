use anyhow::{Context, Result, anyhow};
use clap::Parser;
use osc_lib::{OscArg, OscMessage};
use std::io::{self, Write};
use std::time::Instant;
use x32_lib::{create_socket, get_fx_type};

/// Set the delay time of an X32 effects unit by tapping.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The IP address of the X32 mixer.
    #[arg(short, long, default_value = "192.168.0.64")]
    ip: String,

    /// The FX slot number (1-4) containing the delay effect.
    #[arg(short, long, default_value_t = 1)]
    slot: u8,
    // Future: Auto mode arguments
    // #[arg(short, long)]
    // auto: bool,
}

// Stereo delay FX number (from C source): 10
// Other delay types from C source: 11, 12, 21, 24, 25, 26

fn main() -> Result<()> {
    let args = Args::parse();

    if args.slot < 1 || args.slot > 4 {
        return Err(anyhow!("FX slot must be between 1 and 4."));
    }

    println!("Connecting to X32 at {}...", args.ip);
    let socket = create_socket(&args.ip, 500).context("Failed to create socket")?;

    // Check connection with /info
    let info_msg = OscMessage::new("/info".to_string(), vec![]);
    socket.send(&info_msg.to_bytes()?)?;

    let mut buf = [0u8; 512];
    match socket.recv(&mut buf) {
        Ok(_) => println!("Connected!"),
        Err(e) => return Err(anyhow!("Failed to connect to X32: {}", e)),
    }

    // Verify FX type
    println!("Checking FX slot {}...", args.slot);
    let fx_type = get_fx_type(&socket, args.slot)?;

    // List of known delay FX types from C source
    let valid_delays = [10, 11, 12, 21, 24, 25, 26];
    if !valid_delays.contains(&fx_type) {
        eprintln!(
            "Warning: FX slot {} does not appear to contain a standard delay effect (Type ID: {}).",
            args.slot, fx_type
        );
        eprintln!("Proceeding anyway, but commands may not work as expected.");
    } else {
        println!("Found valid delay effect (Type ID: {}).", fx_type);
    }

    println!("X32Tap - Manual Mode");
    println!("Press <Enter> repeatedly to set tempo.");
    println!("Type 'q' and <Enter> to quit.");

    let mut last_tap: Option<Instant> = None;
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut input_buffer = String::new();

    loop {
        print!("> ");
        stdout.flush()?;
        input_buffer.clear();
        stdin.read_line(&mut input_buffer)?;

        let input = input_buffer.trim();

        if input.eq_ignore_ascii_case("q") {
            break;
        }

        // Treat empty line (just Enter) or any non-command input as a tap
        let now = Instant::now();

        if let Some(last) = last_tap {
            let delta = now.duration_since(last);
            let delta_ms = delta.as_millis() as f32;

            // Calculate parameter value (0.0 - 1.0 represents 0ms - 3000ms)
            let mut f_val = delta_ms / 3000.0;
            if f_val < 0.0 {
                f_val = 0.0;
            }
            if f_val > 1.0 {
                f_val = 1.0;
            }

            let tempo_ms = (f_val * 3000.0) as i32;
            println!("Tempo: {}ms", tempo_ms);

            // Construct OSC message
            // From C source: if type is 0 (DLY=10?) use /par/02, else /par/01?
            // The C code says:
            // if (Xdelay == 0) sprintf(tmpstr, "/fx/%1d/par/02", Xdel_slot); // DLY (10)
            // else sprintf(tmpstr, "/fx/%1d/par/01", Xdel_slot); // Others

            let param_idx = if fx_type == 10 { 2 } else { 1 };
            let address = format!("/fx/{}/par/{:02}", args.slot, param_idx);

            let msg = OscMessage::new(address, vec![OscArg::Float(f_val)]);
            if let Err(e) = socket.send(&msg.to_bytes()?) {
                eprintln!("Failed to send OSC message: {}", e);
            }
        } else {
            println!("First tap...");
        }

        last_tap = Some(now);
    }

    Ok(())
}
