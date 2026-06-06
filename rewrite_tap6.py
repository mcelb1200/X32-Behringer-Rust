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
        _ => println!("Warning: Did not receive response to /info. Proceeding anyway."),
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
        _ => println!("Warning: Did not receive response to /info. Proceeding anyway."),
    }

    handle_client(client, args).await?;"""
text = text.replace(main_old, main_new)

text = text.replace("async fn handle_client(socket: Arc<tokio::net::UdpSocket>, args: Args) -> Result<()> {", "async fn handle_client(client: Arc<MixerClient>, args: Args) -> Result<()> {\n    let mut rx = client.subscribe();")

# socket.send replacements
text = re.sub(r'socket\.send\(&([a-zA-Z0-9_]+)\.to_bytes\(\)\?\)\.await\?;', r'client.send_message(&\1.path, \1.args.clone()).await?;', text)
text = re.sub(r'socket\.send\(&([a-zA-Z0-9_]+)\.to_bytes\(\)\?\)\.await', r'client.send_message(&\1.path, \1.args.clone()).await', text)
text = text.replace("socket.send(&msg.to_bytes()?).await", "client.send_message(&msg.path, msg.args.clone()).await")

# socket.recv replacements
recv_old1 = r"""    if let Ok(res) =
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
recv_new1 = r"""    if let Ok(Ok(msg)) = timeout(Duration::from_millis(500), rx.recv()).await {
        if msg.path == type_req.path {
            if let Some(OscArg::Int(t)) = msg.args.first() {
                fx_type = *t;
            }
        }
    }"""
text = text.replace(recv_old1, recv_new1)

recv_old2 = r"""            if let Ok(Ok(len)) =
                tokio::time::timeout(std::time::Duration::from_millis(100), socket.recv(&mut buf))
                    .await
            {
                if let Ok(msg) = OscMessage::from_bytes(&buf[..len]) {
                    if msg.path == "/meters/6" {"""
recv_new2 = r"""            if let Ok(Ok(msg)) = timeout(Duration::from_millis(100), rx.recv()).await {
                // mock the braces that were lost
                if true {
                    if msg.path == "/meters/6" {"""
text = text.replace(recv_old2, recv_new2)

# Remove `let mut buf = [0u8; 512];`
# Note: we need to replace only in handle_client but since the only match is there, it's fine.
text = text.replace("let mut buf = [0u8; 512];", "")

# Remove `socket.set_nonblocking(true)?;`
text = text.replace("socket.set_nonblocking(true)?;", "")

with open("tools/x32_tap/src/main.rs", "w") as f:
    f.write(text)
