import re

# x32_set_preset
with open("tools/x32_set_preset/src/main.rs", "r") as f:
    text = f.read()

text = text.replace("use x32_lib::create_socket;", "use x32_lib::MixerClient;\nuse tokio::time::{timeout, Duration};")
text = text.replace("use std::net::UdpSocket;", "")
text = text.replace("fn main() -> Result<()> {", "#[tokio::main]\nasync fn main() -> Result<()> {")

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
con_old = r"""    let socket = create_socket(&args.ip, 500).context("Failed to connect to X32")?;"""
con_new = r"""    let (client, _) = MixerClient::connect_with_transport(
        &args.ip,
        &args.aes50_ip,
        &args.usb_port,
        &args.transport,
        false,
    ).await.context("Failed to connect to X32")?;
    let client = std::sync::Arc::new(client);"""
text = text.replace(con_old, con_new)
text = text.replace("socket.send(&msg.to_bytes()?).context(\"Failed to send ping\")?;", "client.send_message(&msg.path, msg.args.clone()).await.context(\"Failed to send ping\")?;")
text = text.replace("socket.send(&msg.to_bytes()?).context(\"Failed to send OSC message\")?;", "client.send_message(&msg.path, msg.args.clone()).await.context(\"Failed to send OSC message\")?;")

# get_node_value
get_old = "fn get_node_value(socket: &std::net::UdpSocket, path: &str) -> Result<OscMessage> {"
get_new = "async fn get_node_value(client: &MixerClient, path: &str) -> Result<OscMessage> {\n    let mut rx = client.subscribe();"
text = text.replace(get_old, get_new)

send_node_old = r"""    socket.send(&msg.to_bytes()?).context("Failed to request node")?;

    let mut buf = [0u8; 1024];
    socket
        .set_read_timeout(Some(std::time::Duration::from_millis(500)))
        .context("Failed to set read timeout")?;

    let len = socket
        .recv(&mut buf)
        .context("Failed to receive node response (Timeout?)")?;

    let response = OscMessage::from_bytes(&buf[..len]).context("Failed to parse OSC response")?;
    Ok(response)"""
send_node_new = r"""    client.send_message(&msg.path, msg.args).await.context("Failed to request node")?;

    let t = Duration::from_millis(500);
    if let Ok(Ok(msg)) = timeout(t, rx.recv()).await {
        Ok(msg)
    } else {
        Err(anyhow!("Timeout waiting for node response"))
    }"""
text = text.replace(send_node_old, send_node_new)

# await calls
text = re.sub(r'get_node_value\(&socket, ([^)]+)\)\?', r'get_node_value(&client, \1).await?', text)

with open("tools/x32_set_preset/src/main.rs", "w") as f:
    f.write(text)

# x32_tap
with open("tools/x32_tap/src/main.rs", "r") as f:
    text = f.read()

text = text.replace("use std::io::{self, Write};", "use std::io::{self, Write};\nuse std::sync::Arc;")
text = text.replace("use x32_lib::{create_socket, get_parameter};", "use x32_lib::{MixerClient, get_parameter_async};\nuse tokio::time::{timeout, Duration, interval};\nuse tokio::io::{AsyncBufReadExt, BufReader};\nuse tokio::task;")
text = text.replace("use std::net::UdpSocket;", "")

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

text = text.replace("fn main() -> Result<()> {", "#[tokio::main]\nasync fn main() -> Result<()> {")

con_old = r"""    let std_socket = create_socket(&args.ip, 500).context("Failed to create socket")?;
    std_socket.set_nonblocking(true)?;
    let socket = tokio::net::UdpSocket::from_std(std_socket)?;

    // Check connection with /info
    let info_msg = OscMessage::new("/info".to_string(), vec![]);
    let info_bytes = info_msg.to_bytes().context("Failed to serialize /info")?;
    socket.send(&info_bytes).await.context("Failed to send /info")?;

    let mut buf = [0; 1024];
    match tokio::time::timeout(std::time::Duration::from_secs(2), socket.recv(&mut buf)).await {
        Ok(Ok(_)) => println!("Connected."),
        _ => eprintln!("Warning: Did not receive response to /info. The IP address might be incorrect or the console is unreachable."),
    }

    let socket = std::sync::Arc::new(socket);

    handle_client(socket, args).await"""

con_new = r"""    let (client, _) = MixerClient::connect_with_transport(
        &args.ip,
        &args.aes50_ip,
        &args.usb_port,
        &args.transport,
        false,
    ).await?;
    let client = std::sync::Arc::new(client);

    let mut rx = client.subscribe();
    client.send_message("/info", vec![]).await.context("Failed to send /info")?;

    match tokio::time::timeout(std::time::Duration::from_secs(2), rx.recv()).await {
        Ok(Ok(_)) => println!("Connected."),
        _ => eprintln!("Warning: Did not receive response to /info. The IP address might be incorrect or the console is unreachable."),
    }

    handle_client(client, args).await"""
text = text.replace(con_old, con_new)

text = text.replace("async fn handle_client(\n    socket: Arc<tokio::net::UdpSocket>,\n    args: Args,\n) -> Result<()> {", "async fn handle_client(\n    client: Arc<MixerClient>,\n    args: Args,\n) -> Result<()> {")
text = text.replace("socket.send(&info_bytes).await?;", "client.send_message(\"/info\", vec![]).await?;")
text = text.replace("socket.send(&msg.to_bytes()?).await?;", "client.send_message(&msg.path, msg.args).await?;")

# in handle_client there is a loop and it uses tokio::net::UdpSocket methods? Wait!
# It's an async fn. Let's see how it polls stdin and socket.

# tokio::select! is used
# replace `socket.recv(&mut buf)` with `rx.recv()`
recv_old = r"""                let res = socket_clone.recv(&mut buf) => {
                    if let Ok(len) = res {
                        if let Ok(msg) = OscMessage::from_bytes(&buf[..len]) {
                            if msg.path == "/meters/6" {
                                if let Some(OscArg::Blob(b)) = msg.args.first() {
                                    if b.len() >= 4 + (args.channel as usize * 4) {
                                        let offset = 4 + ((args.channel as usize - 1) * 4);
                                        let mut val_bytes = [0u8; 4];
                                        val_bytes.copy_from_slice(&b[offset..offset + 4]);
                                        let meter_val = f32::from_le_bytes(val_bytes);
                                        // Simple gate logic: trigger tap when signal goes above threshold after being below
                                        if meter_val > args.threshold && !gate_open {
                                            gate_open = true;
                                            process_tap(&mut last_tap, &mut t2_ms, &mut last_t2, &client_clone, args.slot, args.parameter, true).await;
                                        } else if meter_val < args.threshold - 0.05 {
                                            gate_open = false;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }"""
recv_new = r"""                let res = rx.recv() => {
                    if let Ok(msg) = res {
                        if msg.path == "/meters/6" {
                            if let Some(OscArg::Blob(b)) = msg.args.first() {
                                if b.len() >= 4 + (args.channel as usize * 4) {
                                    let offset = 4 + ((args.channel as usize - 1) * 4);
                                    let mut val_bytes = [0u8; 4];
                                    val_bytes.copy_from_slice(&b[offset..offset + 4]);
                                    let meter_val = f32::from_le_bytes(val_bytes);
                                    // Simple gate logic: trigger tap when signal goes above threshold after being below
                                    if meter_val > args.threshold && !gate_open {
                                        gate_open = true;
                                        process_tap(&mut last_tap, &mut t2_ms, &mut last_t2, &client_clone, args.slot, args.parameter, true).await;
                                    } else if meter_val < args.threshold - 0.05 {
                                        gate_open = false;
                                    }
                                }
                            }
                        }
                    }
                }"""
text = text.replace(recv_old, recv_new)

# `let mut reader = BufReader::new(io::stdin());`
text = text.replace("let mut reader = tokio::io::BufReader::new(tokio::io::stdin());", "let mut reader = BufReader::new(tokio::io::stdin());")
text = text.replace("let mut reader = BufReader::new(io::stdin());", "let mut reader = BufReader::new(tokio::io::stdin());")

text = text.replace("let socket_clone = socket.clone();", "let client_clone = client.clone();\n    let mut rx = client_clone.subscribe();")
text = text.replace("let ping_msg = OscMessage::new(\"/info\".to_string(), vec![]);", "")
text = text.replace("let ping_bytes = ping_msg.to_bytes().expect(\"Failed to serialize /info\");", "")
text = text.replace("socket_clone.send(&ping_bytes).await", "client_clone.send_message(\"/info\", vec![]).await")

text = text.replace("socket: &Arc<tokio::net::UdpSocket>", "client: &Arc<MixerClient>")
text = text.replace("socket: &MixerClient", "client: &MixerClient")

# Also `process_tap` needs `client`
# I will just replace `socket` with `client` globally inside handle_client parameters and process_tap
text = text.replace("fn process_tap(\n    last_tap: &mut Option<Instant>,\n    t2_ms: &mut f32,\n    last_t2: &mut f32,\n    socket: &Arc<tokio::net::UdpSocket>,", "async fn process_tap(\n    last_tap: &mut Option<Instant>,\n    t2_ms: &mut f32,\n    last_t2: &mut f32,\n    client: &Arc<MixerClient>,")
text = text.replace("process_tap(&mut last_tap, &mut t2_ms, &mut last_t2, &socket_clone, args.slot, args.parameter, true).await;", "process_tap(&mut last_tap, &mut t2_ms, &mut last_t2, &client_clone, args.slot, args.parameter, true).await;")
text = text.replace("process_tap(&mut last_tap, &mut t2_ms, &mut last_t2, &socket_clone, args.slot, args.parameter, false).await;", "process_tap(&mut last_tap, &mut t2_ms, &mut last_t2, &client_clone, args.slot, args.parameter, false).await;")

text = text.replace("pub async fn run_tap(ip: &str, fx_slot: u8, parameter: u8, value: f32) -> Result<()> {", "pub async fn run_tap(client: &MixerClient, fx_slot: u8, parameter: u8, value: f32) -> Result<()> {")
text = text.replace("let socket = x32_lib::create_socket(ip, 500)?;", "")
text = re.sub(r'x32_lib::set_parameter_async\(&socket,([^,]+),\s*([^)]+)\)\.await\?;', r'x32_lib::set_parameter_async(client, \1, \2).await?;', text)

text = text.replace("run_tap(&socket", "run_tap(client")
text = text.replace("run_tap(client, args.slot, args.parameter, value).await", "run_tap(client, args.slot, args.parameter, value).await")

text = text.replace("get_parameter(&socket_clone", "get_parameter_async(&client_clone")

with open("tools/x32_tap/src/main.rs", "w") as f:
    f.write(text)
