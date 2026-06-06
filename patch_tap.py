import re

with open("tools/x32_tap/src/main.rs", "r") as f:
    text = f.read()

# Replace std::net::UdpSocket and create_socket
text = text.replace("use std::net::UdpSocket;\n", "")
text = text.replace("use x32_lib::create_socket;", "use x32_lib::MixerClient;")
text = text.replace("use x32_lib::{create_socket, get_parameter};", "use x32_lib::{MixerClient, get_parameter_async};")

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
text = re.sub(r'struct Args \{[\s\n]*/// IP address of the X32 console[\s\n]*#\[arg\(short, long, default_value = "192.168.0.64"\)\][\s\n]*ip: String,', cli_repl, text)

# fn main replace create_socket
main_old = r"""    let std_socket = create_socket(&args.ip, 500).context("Failed to create socket")?;
    let async_socket = tokio::net::UdpSocket::from_std(std_socket)
        .context("Failed to convert to async socket")?;
    let socket = Arc::new(async_socket);"""
main_new = r"""    let (client, _) = MixerClient::connect_with_transport(
        &args.ip,
        &args.aes50_ip,
        &args.usb_port,
        &args.transport,
        true, // needs heartbeat? wait, tap uses /xremote? Let's say false or let tap do it.
    ).await?;
    let client = Arc::new(client);"""
text = text.replace(main_old, main_new)

# handle_client
hc_old = "async fn handle_client(\n    socket: Arc<tokio::net::UdpSocket>,\n    args: Args,\n) -> Result<()> {"
hc_new = "async fn handle_client(\n    client: Arc<MixerClient>,\n    args: Args,\n) -> Result<()> {"
text = text.replace(hc_old, hc_new)
text = text.replace("let socket_clone = socket.clone();", "let client_clone = client.clone();")

# send task
# we need to rewrite the interval task.
send_task_old = r"""        let mut interval = interval(Duration::from_millis(if args.auto { 50 } else { 100 }));
        let ping_msg = OscMessage::new("/info".to_string(), vec![]);
        let ping_bytes = ping_msg.to_bytes().expect("Failed to serialize /info");

        loop {
            interval.tick().await;
            if socket_clone.send(&ping_bytes).await.is_err() {
                eprintln!("Failed to send ping.");
            }
        }"""
# Wait, actually x32_tap just sends /info periodically? Why? Probably to keep connection alive or something, or it doesn't matter. It also checks state.
send_task_new = r"""        let mut interval = interval(Duration::from_millis(if args.auto { 50 } else { 100 }));
        loop {
            interval.tick().await;
            if client_clone.send_message("/info", vec![]).await.is_err() {
                eprintln!("Failed to send ping.");
            }
        }"""
text = text.replace(send_task_old, send_task_new)

# async read STDIN task
# it uses `let mut reader = BufReader::new(io::stdin());`
text = text.replace("socket.send(&ping_bytes).await?;", "client.send_message(\"/info\", vec![]).await?;")
text = text.replace("socket.send(&msg.to_bytes()?).await?;", "client.send_message(&msg.path, msg.args).await?;")

text = text.replace("pub async fn run_tap(ip: &str, fx_slot: u8, parameter: u8, value: f32) -> Result<()> {", "pub async fn run_tap(client: &MixerClient, fx_slot: u8, parameter: u8, value: f32) -> Result<()> {")
text = text.replace("let socket = x32_lib::create_socket(ip, 500)?;", "")
text = text.replace("x32_lib::set_parameter_async(&socket", "x32_lib::set_parameter_async(client")

# Fix `set_parameter_async` call
text = re.sub(r'x32_lib::set_parameter_async\([^,]+,\s*([^,]+),\s*([^)]+)\)\.await\?;', r'x32_lib::set_parameter_async(client, \1, \2).await?;', text)

with open("tools/x32_tap/src/main.rs", "w") as f:
    f.write(text)
