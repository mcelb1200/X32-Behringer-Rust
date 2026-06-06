import re

with open("tools/x32_set_preset/src/main.rs", "r") as f:
    text = f.read()

text = text.replace("use x32_lib::create_socket;", "use x32_lib::MixerClient;\nuse tokio::time::{timeout, Duration};\nuse std::sync::Arc;")
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

con_old = r"""    let socket = create_socket(&args.ip, 500).context("Failed to connect to X32")?;"""
con_new = r"""    let (client, _) = MixerClient::connect_with_transport(
        &args.ip,
        &args.aes50_ip,
        &args.usb_port,
        &args.transport,
        false,
    ).await.context("Failed to connect to X32")?;
    let client = Arc::new(client);"""
text = text.replace(con_old, con_new)

# handle process_preset
text = text.replace("process_preset(&socket,", "process_preset(&client,")
text = text.replace("fn process_preset(socket: &std::net::UdpSocket,", "async fn process_preset(client: &MixerClient,")
# process_preset call in main needs await
text = re.sub(r'process_preset\(&client,([^)]+)\)\?;', r'process_preset(&client,\1).await?;', text)

# socket.send
text = re.sub(r'socket\.send\(&msg\.to_bytes\(\)\?\)\.context\("([^"]+)"\)\?;', r'client.send_message(&msg.path, msg.args.clone()).await.context("\1")?;', text)

# get_node_value
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
        Err(anyhow::anyhow!("Timeout waiting for node response"))
    }
}"""
text = text.replace(get_old, get_new)

text = re.sub(r'get_node_value\(&socket, ([^)]+)\)\?', r'get_node_value(client, \1).await?', text)

with open("tools/x32_set_preset/src/main.rs", "w") as f:
    f.write(text)
