import re

with open("tools/x32_tap/src/main.rs", "r") as f:
    text = f.read()

# Replace std::net::UdpSocket
text = text.replace("use std::net::UdpSocket;\n", "")
text = text.replace("use x32_lib::create_socket;", "use x32_lib::MixerClient;\nuse tokio::time::{timeout, Duration};\nuse std::sync::Arc;")

# Add transport fields to Args
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
main_old = r"""    println!("Connecting to X32 at {}...", args.ip);
    let std_socket = create_socket(&args.ip, 500).context("Failed to create socket")?;
    std_socket.set_nonblocking(true)?;
    let socket = tokio::net::UdpSocket::from_std(std_socket)?;

    // Check connection with /info
    let info_msg = OscMessage::new("/info".to_string(), vec![]);
    socket.send(&info_msg.to_bytes()?).await?;

    let mut buf = [0u8; 512];
    match tokio::time::timeout(std::time::Duration::from_millis(500), socket.recv(&mut buf)).await {
        Ok(Ok(_)) => println!("Connected!"),
        Ok(Err(e)) => return Err(anyhow!("Failed to connect to X32: {}", e)),
        Err(_) => {
            return Err(anyhow!(
                "Connection to X32 timed out. Is the IP address correct?"
            ));
        }
    }

    let socket = Arc::new(socket);

    handle_client(socket, args).await?;"""

main_new = r"""    println!("Connecting to X32 at {}...", args.ip);
    let (client, _) = MixerClient::connect_with_transport(
        &args.ip,
        &args.aes50_ip,
        &args.usb_port,
        &args.transport,
        false,
    ).await.context("Failed to connect to X32")?;
    let client = Arc::new(client);
    let mut rx = client.subscribe();

    client.send_message("/info", vec![]).await?;

    match timeout(Duration::from_millis(500), rx.recv()).await {
        Ok(Ok(_)) => println!("Connected!"),
        Ok(Err(e)) => return Err(anyhow::anyhow!("Failed to connect to X32: {}", e)),
        Err(_) => {
            return Err(anyhow::anyhow!(
                "Connection to X32 timed out. Is the IP address correct?"
            ));
        }
    }

    handle_client(client, args).await?;"""
text = text.replace(main_old, main_new)

text = text.replace("async fn handle_client(socket: Arc<tokio::net::UdpSocket>, args: Args) -> Result<()> {", "async fn handle_client(client: Arc<MixerClient>, args: Args) -> Result<()> {\n    let mut rx = client.subscribe();")

# fx_type resolution block
fx_old = r"""    let type_req = OscMessage::new(
        format!("/fx/{}/type", args.slot),
        vec![OscArg::String(format!("/fx/{}/type", args.slot))],
    );
    socket.send(&type_req.to_bytes()?).await?;

    // Wait for response
    #[allow(clippy::collapsible_match)]
    if let Ok(res) =
        tokio::time::timeout(std::time::Duration::from_millis(500), socket.recv(&mut buf)).await
    {
        if let Ok(len) = res {
            if let Ok(msg) = OscMessage::from_bytes(&buf[..len]) {
                if msg.path == type_req.path {
                    if let Some(OscArg::Int(t)) = msg.args.first() {
                        fx_type = *t;
                    }
                }
            }
        }
    }"""
fx_new = r"""    let type_req = OscMessage::new(
        format!("/fx/{}/type", args.slot),
        vec![OscArg::String(format!("/fx/{}/type", args.slot))],
    );
    client.send_message(&type_req.path, type_req.args).await?;

    if let Ok(Ok(msg)) = timeout(Duration::from_millis(500), rx.recv()).await {
        // Since we sent type_req, we format the path again
        let req_path = format!("/fx/{}/type", args.slot);
        if msg.path == req_path {
            if let Some(OscArg::Int(t)) = msg.args.first() {
                fx_type = *t as i32;
            }
        }
    }"""
text = text.replace(fx_old, fx_new)

text = text.replace("socket.send(&xremote_msg.to_bytes()?).await", "client.send_message(&xremote_msg.path, xremote_msg.args.clone()).await")
text = text.replace("socket.send(&meter_req.to_bytes()?).await", "client.send_message(&meter_req.path, meter_req.args.clone()).await")

# loop recv block
loop_old = r"""            if let Ok(Ok(len)) =
                tokio::time::timeout(std::time::Duration::from_millis(100), socket.recv(&mut buf))
                    .await
            {
                if let Ok(msg) = OscMessage::from_bytes(&buf[..len]) {
                    if msg.path == "/meters/6" {
                        if let Some(OscArg::Blob(data)) = msg.args.first() {
                            // C code reads float at offset 12 of the data (which is byte 28 from start of packet).
                            // A blob in /meters/6 contains 4 floats: 4 * 4 = 16 bytes.
                            if data.len() >= 16 {
                                let mut f_bytes = [0u8; 4];
                                // Rust OSC blobs usually come out as raw bytes. The float at offset 12:
                                if let Some(slice) = data.get(12..16) {
                                    f_bytes.copy_from_slice(slice);
                                } else {
                                    continue;
                                }
                                // X32 sends floats in Little Endian in blobs.
                                let level = f32::from_le_bytes(f_bytes);

                                if level > args.threshold {
                                    if !was_above_threshold {
                                        let tap_time = Instant::now();
                                        if let Some(last) = last_tap {
                                            let delta = tap_time.duration_since(last);
                                            let delta_ms = delta.as_millis() as f32;

                                            // Minimum resolution is 60ms to avoid rapid-fire updates
                                            if delta_ms > 60.0 {
                                                let f_val = (delta_ms / 3000.0).clamp(0.0, 1.0);
                                                let tempo_ms = (f_val * 3000.0) as i32;
                                                println!(
                                                    "Auto Tap: {}ms (level: {:.2})",
                                                    tempo_ms, level
                                                );

                                                let update_msg = OscMessage::new(
                                                    address.clone(),
                                                    vec![OscArg::Float(f_val)],
                                                );
                                                if let Err(e) =
                                                    socket.send(&update_msg.to_bytes()?).await
                                                {
                                                    eprintln!(
                                                        "Failed to update FX parameter: {}",
                                                        e
                                                    );
                                                }
                                                last_tap = Some(tap_time);
                                            }
                                        } else {
                                            println!("First auto tap... (level: {:.2})", level);
                                            last_tap = Some(tap_time);
                                        }
                                        was_above_threshold = true;
                                    }
                                } else {
                                    was_above_threshold = false;
                                }
                            }
                        }
                    }
                }
            }"""
loop_new = r"""            if let Ok(Ok(msg)) = timeout(Duration::from_millis(100), rx.recv()).await {
                if msg.path == "/meters/6" {
                    if let Some(OscArg::Blob(data)) = msg.args.first() {
                        if data.len() >= 16 {
                            let mut f_bytes = [0u8; 4];
                            if let Some(slice) = data.get(12..16) {
                                f_bytes.copy_from_slice(slice);
                            } else {
                                continue;
                            }
                            let level = f32::from_le_bytes(f_bytes);

                            if level > args.threshold {
                                if !was_above_threshold {
                                    let tap_time = Instant::now();
                                    if let Some(last) = last_tap {
                                        let delta = tap_time.duration_since(last);
                                        let delta_ms = delta.as_millis() as f32;

                                        if delta_ms > 60.0 {
                                            let f_val = (delta_ms / 3000.0).clamp(0.0, 1.0);
                                            let tempo_ms = (f_val * 3000.0) as i32;
                                            println!(
                                                "Auto Tap: {}ms (level: {:.2})",
                                                tempo_ms, level
                                            );

                                            let update_msg = OscMessage::new(
                                                address.clone(),
                                                vec![OscArg::Float(f_val)],
                                            );
                                            if let Err(e) =
                                                client.send_message(&update_msg.path, update_msg.args.clone()).await
                                            {
                                                eprintln!(
                                                    "Failed to update FX parameter: {}",
                                                    e
                                                );
                                            }
                                            last_tap = Some(tap_time);
                                        }
                                    } else {
                                        println!("First auto tap... (level: {:.2})", level);
                                        last_tap = Some(tap_time);
                                    }
                                    was_above_threshold = true;
                                }
                            } else {
                                was_above_threshold = false;
                            }
                        }
                    }
                }
            }"""
text = text.replace(loop_old, loop_new)

text = text.replace("if let Err(e) = socket.send(&msg.to_bytes()?).await {", "if let Err(e) = client.send_message(&msg.path, msg.args.clone()).await {")
text = text.replace("let mut buf = [0u8; 512];\n", "")
text = text.replace("let mut buf = [0; 512];\n", "")

with open("tools/x32_tap/src/main.rs", "w") as f:
    f.write(text)
