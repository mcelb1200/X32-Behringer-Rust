import re

with open("tools/x32_get_lib/src/main.rs", "r") as f:
    text = f.read()

# Replace std::net::UdpSocket
text = text.replace("use std::net::UdpSocket;", "")
text = text.replace("use x32_lib::create_socket;", "use x32_lib::{MixerClient, error::X32Error};\nuse tokio::time::{timeout, Duration};")

# Add transport fields to Args
cli_repl = """struct Args {
    #[arg(short, long, default_value = "192.168.1.62")]
    ip: String,

    #[arg(long, default_value = "auto")]
    transport: String,

    #[arg(long, default_value = "")]
    usb_port: String,

    #[arg(long, default_value = "")]
    aes50_ip: String,"""
text = re.sub(r'struct Args \{[\s\n]*/// The IP address of the X32 console[\s\n]*#\[arg\(short, long, default_value = "192.168.1.62"\)\][\s\n]*ip: String,', cli_repl, text)

# Update fn main
main_old = r"""fn main() -> Result<()> {
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
}"""
main_new = r"""#[tokio::main]
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
}"""
text = text.replace(main_old, main_new)

# Update process_lib_slot signature
text = text.replace("fn process_lib_slot(\n    socket: &std::net::UdpSocket,", "async fn process_lib_slot(\n    client: &MixerClient,")
text = text.replace("fn process_lib_slot(\n    socket: &UdpSocket,", "async fn process_lib_slot(\n    client: &MixerClient,")

# Update process_lib_slot body
pl_old = r"""    // Get Node info (name)
    // /node ,s -libs/{type}/{id}
    let node_arg = format!("-libs/{}/{:03}", type_str, id);
    let msg = OscMessage::new("/node".to_string(), vec![OscArg::String(node_arg)]);
    socket.send(&msg.to_bytes()?)?;

    let mut buf = [0u8; 512];
    let len = socket.recv(&mut buf)?;
    let resp = OscMessage::from_bytes(&buf[..len])?;"""
pl_new = r"""    let mut rx = client.subscribe();
    let node_arg = format!("-libs/{}/{:03}", type_str, id);
    client.send_message("/node", vec![OscArg::String(node_arg)]).await?;

    let resp = match timeout(Duration::from_millis(500), rx.recv()).await {
        Ok(Ok(m)) => m,
        _ => return Err(X32Error::from("Timeout waiting for node").into()),
    };"""
text = text.replace(pl_old, pl_new)

text = text.replace("socket.send(&msg.to_bytes()?)?;", "client.send_message(&msg.path, msg.args.clone()).await?;")
text = text.replace("socket.set_read_timeout(Some(Duration::from_millis(200)))?;", "")
text = re.sub(r'    if let Ok\(_len\) = socket\.recv\(&mut buf\) \{\n        // Assume success for now\n    \}', '    let _ = timeout(Duration::from_millis(200), rx.recv()).await;', text)
text = text.replace("socket.set_read_timeout(Some(Duration::from_millis(500)))?;", "")

loop_old = r"""        if let Ok(len) = socket.recv(&mut buf) {
            if let Ok(resp) = OscMessage::from_bytes(&buf[..len]) {
                if resp.path == "node" {
                    if let Some(OscArg::String(val)) = resp.args.first() {
                        let mut output = val.clone();

                        match t {
                            LibType::Channel => {
                                // Strip "/ch/01" from the beginning
                                if let Some(stripped) = output.strip_prefix("ch/01") {
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
                                if let Some(stripped) = output.strip_prefix("fx/1/") {
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
        }"""
loop_new = r"""        if let Ok(Ok(resp)) = timeout(Duration::from_millis(500), rx.recv()).await {
            if resp.path == "node" || resp.path == "/node" {
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
        }"""
text = text.replace(loop_old, loop_new)

headamp_old = r"""        if let Ok(len) = socket.recv(&mut buf) {
            if let Ok(resp) = OscMessage::from_bytes(&buf[..len]) {
                if resp.path == "node" {
                    if let Some(OscArg::String(val)) = resp.args.first() {
                        writeln!(file, "{}", val)?;
                    }
                }
            }
        } else {
            eprintln!("  Error or timeout on command: /node ,s headamp/000");
        }"""
headamp_new = r"""        if let Ok(Ok(resp)) = timeout(Duration::from_millis(500), rx.recv()).await {
            if resp.path == "node" || resp.path == "/node" {
                if let Some(OscArg::String(val)) = resp.args.first() {
                    writeln!(file, "{}", val)?;
                }
            }
        } else {
            eprintln!("  Error or timeout on command: /node ,s headamp/000");
        }"""
text = text.replace(headamp_old, headamp_new)

with open("tools/x32_get_lib/src/main.rs", "w") as f:
    f.write(text)
