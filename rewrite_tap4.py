import re

with open("tools/x32_tap/src/main.rs", "r") as f:
    text = f.read()

# Add tokio dep import
text = text.replace("use std::net::UdpSocket;\n", "")
text = text.replace("use x32_lib::create_socket;", "use x32_lib::MixerClient;\nuse tokio::time::{timeout, Duration, interval};\nuse tokio::io::{AsyncBufReadExt, BufReader};\nuse std::sync::Arc;")

# Modify Args
cli_repl = """struct Args {
    #[arg(short, long, default_value = "192.168.0.64")]
    ip: String,

    #[arg(long, default_value = "auto")]
    transport: String,

    #[arg(long, default_value = "")]
    usb_port: String,

    #[arg(long, default_value = "")]
    aes50_ip: String,"""
text = re.sub(r'struct Args \{[\s\n]*/// The IP address of the X32 mixer\.[\s\n]*#\[arg\(short, long, default_value = "192.168.0.64"\)\][\s\n]*ip: String,', cli_repl, text)

# main connection
main_old = r"""fn main() -> Result<()> {
    let args = Args::parse();

    if args.slot < 1 || args.slot > 4 {
        return Err(anyhow!("FX slot must be between 1 and 4."));
    }

    if args.channel < 1 || args.channel > 32 {
        return Err(anyhow!("Channel must be between 1 and 32."));
    }

    if args.threshold <= 0.0 || args.threshold >= 1.0 {
        return Err(anyhow!("Threshold must be between 0.0 and 1.0."));
    }

    println!("Connecting to X32 at {}...", args.ip);
    let socket = create_socket(&args.ip, 500)?;
    socket.set_read_timeout(Some(std::time::Duration::from_millis(500)))?;

    // Check connection with /info
    let info_msg = OscMessage::new("/info".to_string(), vec![]);
    socket.send(&info_msg.to_bytes()?)?;

    let mut buf = [0u8; 512];
    if let Ok(_) = socket.recv(&mut buf) {
        println!("Connected!");
    } else {
        println!("Warning: Did not receive response to /info. Proceeding anyway.");
    }

    handle_client(socket, args)?;

    Ok(())
}"""
main_new = r"""#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    if args.slot < 1 || args.slot > 4 {
        return Err(anyhow!("FX slot must be between 1 and 4."));
    }

    if args.channel < 1 || args.channel > 32 {
        return Err(anyhow!("Channel must be between 1 and 32."));
    }

    if args.threshold <= 0.0 || args.threshold >= 1.0 {
        return Err(anyhow!("Threshold must be between 0.0 and 1.0."));
    }

    println!("Connecting to X32 at {}...", args.ip);
    let (client, _) = MixerClient::connect_with_transport(
        &args.ip,
        &args.aes50_ip,
        &args.usb_port,
        &args.transport,
        false,
    ).await?;
    let client = Arc::new(client);

    let mut rx = client.subscribe();
    client.send_message("/info", vec![]).await?;

    if let Ok(Ok(_)) = timeout(Duration::from_millis(500), rx.recv()).await {
        println!("Connected!");
    } else {
        println!("Warning: Did not receive response to /info. Proceeding anyway.");
    }

    handle_client(client, args).await?;

    Ok(())
}"""
text = text.replace(main_old, main_new)

# handle_client
hc_old = r"""fn handle_client(socket: std::net::UdpSocket, args: Args) -> Result<()> {
    let mut last_tap: Option<Instant> = None;
    let mut t2_ms = 0.0;
    let mut last_t2 = 0.0;

    let fx_addr = format!("/fx/{}/par/{:02}", args.slot, args.parameter);

    // Test initial query
    if let Err(e) = x32_lib::get_parameter(&socket, &fx_addr) {
        eprintln!("Warning: Failed to query initial parameter state: {}", e);
    }

    println!("Ready to tap. Press Enter to tap, 'q' to quit.");
    if args.auto {
        println!("Auto mode enabled. Monitoring CH{:02} with threshold {:.2}", args.channel, args.threshold);
    }

    let mut buf = [0u8; 1024];
    let mut last_meters_req = Instant::now() - std::time::Duration::from_secs(10);
    let mut gate_open = false;

    // Use non-blocking mode for the main loop to read stdin and socket interleaved
    socket.set_nonblocking(true)?;

    loop {
        // --- 1. Auto mode socket reading ---
        if args.auto {
            // Subscribe to /meters/6 every 9 seconds
            if last_meters_req.elapsed().as_secs() >= 9 {
                let msg = OscMessage::new("/meters".to_string(), vec![
                    OscArg::String("/meters/6".to_string()),
                    OscArg::Int(0),
                    OscArg::Int(0),
                    OscArg::Int(args.channel as i32 - 1),
                ]);
                socket.send(&msg.to_bytes()?)?;
                last_meters_req = Instant::now();
            }

            // Read all pending packets
            while let Ok(len) = socket.recv(&mut buf) {
                if let Ok(msg) = OscMessage::from_bytes(&buf[..len]) {
                    if msg.path == "/meters/6" {
                        if let Some(OscArg::Blob(b)) = msg.args.first() {
                            // The blob data format for /meters/6 contains float32 values
                            // It has a header of 4 bytes (size), so the first float is at offset 4.
                            // However, we only requested 1 channel (the one we want).
                            // Let's safely extract it.
                            if let Some(chunk) = b.get(4..8) {
                                let mut val_bytes = [0u8; 4];
                                val_bytes.copy_from_slice(chunk);
                                let meter_val = f32::from_le_bytes(val_bytes);

                                // Simple gate logic
                                if meter_val > args.threshold && !gate_open {
                                    gate_open = true;
                                    process_tap(&mut last_tap, &mut t2_ms, &mut last_t2, &socket, args.slot, args.parameter, true)?;
                                } else if meter_val < args.threshold - 0.05 {
                                    gate_open = false;
                                }
                            }
                        }
                    }
                }
            }
        }

        // --- 2. Manual Stdin reading ---
        // Using crossbeam-channel or mpsc to read stdin asynchronously in standard Rust
        // is complex. Since we have a non-blocking socket, we can just use a trick or
        // rely on tokio. But here we have standard io.
        // We will just do a very quick poll using a background thread and a channel.
"""
hc_new = r"""async fn handle_client(client: Arc<MixerClient>, args: Args) -> Result<()> {
    let mut last_tap: Option<Instant> = None;
    let mut t2_ms = 0.0;
    let mut last_t2 = 0.0;

    let fx_addr = format!("/fx/{}/par/{:02}", args.slot, args.parameter);

    // Test initial query
    if let Err(e) = x32_lib::get_parameter_async(&client, &fx_addr).await {
        eprintln!("Warning: Failed to query initial parameter state: {}", e);
    }

    println!("Ready to tap. Press Enter to tap, 'q' to quit.");
    if args.auto {
        println!("Auto mode enabled. Monitoring CH{:02} with threshold {:.2}", args.channel, args.threshold);
    }

    let mut last_meters_req = Instant::now() - Duration::from_secs(10);
    let mut gate_open = false;

    let mut rx = client.subscribe();
    let mut reader = BufReader::new(tokio::io::stdin());
    let mut line = String::new();

    loop {
        if args.auto {
            if last_meters_req.elapsed().as_secs() >= 9 {
                let msg = OscMessage::new("/meters".to_string(), vec![
                    OscArg::String("/meters/6".to_string()),
                    OscArg::Int(0),
                    OscArg::Int(0),
                    OscArg::Int(args.channel as i32 - 1),
                ]);
                client.send_message(&msg.path, msg.args).await?;
                last_meters_req = Instant::now();
            }
        }

        tokio::select! {
            res = rx.recv() => {
                if let Ok(msg) = res {
                    if msg.path == "/meters/6" {
                        if let Some(OscArg::Blob(b)) = msg.args.first() {
                            if let Some(chunk) = b.get(4..8) {
                                let mut val_bytes = [0u8; 4];
                                val_bytes.copy_from_slice(chunk);
                                let meter_val = f32::from_le_bytes(val_bytes);

                                if meter_val > args.threshold && !gate_open {
                                    gate_open = true;
                                    process_tap(&mut last_tap, &mut t2_ms, &mut last_t2, &client, args.slot, args.parameter, true).await?;
                                } else if meter_val < args.threshold - 0.05 {
                                    gate_open = false;
                                }
                            }
                        }
                    }
                }
            }
            res = reader.read_line(&mut line) => {
                if let Ok(0) = res {
                    break;
                }
                if let Ok(_) = res {
                    let trimmed = line.trim();
                    if trimmed == "q" || trimmed == "quit" {
                        break;
                    }
                    if !args.auto {
                        process_tap(&mut last_tap, &mut t2_ms, &mut last_t2, &client, args.slot, args.parameter, false).await?;
                    }
                    line.clear();
                }
            }
        }
    }
"""

# Wait, `handle_client` in HEAD doesn't use `std::sync::mpsc` exactly like that. Let me look at it first.
# Ah, instead of blind replacing, I will just rewrite handle_client and process_tap manually.

with open("tools/x32_tap/src/main.rs", "w") as f:
    # I will replace the whole handle_client and process_tap
    pass
