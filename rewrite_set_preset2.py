import re

with open("tools/x32_set_preset/src/main.rs", "r") as f:
    text = f.read()

# Replace std::net::UdpSocket
text = text.replace("use std::net::UdpSocket;\n", "")
text = text.replace("use x32_lib::create_socket;", "use x32_lib::MixerClient;\nuse tokio::time::{timeout, Duration};")

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

# fn main and connect logic
main_old = r"""fn main() -> Result<()> {
    let args = Args::parse();

    // Parse Preset Type
    let ext = args
        .file
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let ptype = match ext.as_str() {
        "chn" => PresetType::Channel,
        "efx" => PresetType::Effect,
        "rou" => PresetType::Routing,
        _ => PresetType::Unknown,
    };

    if ptype == PresetType::Unknown {
        return Err(anyhow!(
            "Unsupported file extension: {}. Expected .chn, .efx, or .rou",
            ext
        ));
    }

    // Parse Target String
    let target_prefix = if let Some(t) = &args.target {
        Some(parse_target(t, &ptype)?)
    } else {
        None
    };

    // Connect to X32
    println!("Connecting to X32 at {}...", args.ip);
    let socket = create_socket(&args.ip, 500)?;"""

main_new = r"""#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let ext = args
        .file
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let ptype = match ext.as_str() {
        "chn" => PresetType::Channel,
        "efx" => PresetType::Effect,
        "rou" => PresetType::Routing,
        _ => PresetType::Unknown,
    };

    if ptype == PresetType::Unknown {
        return Err(anyhow!(
            "Unsupported file extension: {}. Expected .chn, .efx, or .rou",
            ext
        ));
    }

    let target_prefix = if let Some(t) = &args.target {
        Some(parse_target(t, &ptype)?)
    } else {
        None
    };

    println!("Connecting to X32 at {}...", args.ip);
    let (client, _) = MixerClient::connect_with_transport(
        &args.ip,
        &args.aes50_ip,
        &args.usb_port,
        &args.transport,
        false,
    ).await?;
    let client = std::sync::Arc::new(client);"""
text = text.replace(main_old, main_new)

# socket.send replacements in main
text = re.sub(r'socket\.send\(&msg\.to_bytes\(\)\?\)\?;', r'client.send_message(&msg.path, msg.args.clone()).await?;', text)

# update get_node_value
get_old = r"""fn get_node_value(socket: &std::net::UdpSocket, path: &str) -> Result<OscMessage> {
    let msg = OscMessage::new("/node".to_string(), vec![OscArg::String(path.to_string())]);
    socket.send(&msg.to_bytes()?).context("Failed to request node")?;

    let mut buf = [0u8; 1024];
    socket
        .set_read_timeout(Some(std::time::Duration::from_millis(500)))
        .context("Failed to set read timeout")?;

    let len = socket
        .recv(&mut buf)
        .context("Failed to receive node response (Timeout?)")?;

    let response = OscMessage::from_bytes(&buf[..len]).context("Failed to parse OSC response")?;
    Ok(response)
}"""
get_new = r"""async fn get_node_value(client: &MixerClient, path: &str) -> Result<OscMessage> {
    let mut rx = client.subscribe();
    let msg = OscMessage::new("/node".to_string(), vec![OscArg::String(path.to_string())]);
    client.send_message(&msg.path, msg.args).await.context("Failed to request node")?;

    if let Ok(Ok(msg)) = timeout(Duration::from_millis(500), rx.recv()).await {
        Ok(msg)
    } else {
        Err(anyhow!("Timeout waiting for node response"))
    }
}"""
text = text.replace(get_old, get_new)

# handle process_preset
text = text.replace("process_preset(&socket,", "process_preset(&client,")
text = text.replace("fn process_preset(socket: &std::net::UdpSocket,", "async fn process_preset(client: &MixerClient,")
text = text.replace("process_preset(&client, args.target.clone(), args.safe_eq, args.safe_dyn, args.safe_mca, args.safe_preamp, args.safe_config, args.file, args.verbose)?;", "process_preset(&client, args.target.clone(), args.safe_eq, args.safe_dyn, args.safe_mca, args.safe_preamp, args.safe_config, args.file, args.verbose).await?;")
text = re.sub(r'get_node_value\(&client, ([^)]+)\)\?', r'get_node_value(client, \1).await?', text)

with open("tools/x32_set_preset/src/main.rs", "w") as f:
    f.write(text)
