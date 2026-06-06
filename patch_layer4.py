import re

with open("tools/x32_custom_layer/src/main.rs", "r") as f:
    text = f.read()

# Replace std::net::UdpSocket
text = text.replace("use std::net::UdpSocket;\n", "")
text = text.replace("use x32_lib::{\n    create_socket,\n    error::{Result, X32Error},\n};", "use x32_lib::{MixerClient, error::{Result, X32Error}};\nuse tokio::time::{timeout, Duration};")

# Add transport fields to Cli
cli_repl = """struct Cli {
    #[arg(short, long, default_value = "192.168.0.64")]
    ip: String,

    #[arg(long, default_value = "auto")]
    transport: String,

    #[arg(long, default_value = "")]
    usb_port: String,

    #[arg(long, default_value = "")]
    aes50_ip: String,

    #[clap(subcommand)]
    command: Commands,
}"""
text = re.sub(r'struct Cli \{[\s\n]*#\[clap\(subcommand\)\][\s\n]*command: Commands,[\s\n]*\}', cli_repl, text)

# Remove `ip: String` from all Commands variants
text = re.sub(r'/// The IP address of the X32 mixer\n\s+ip: String,\n', '', text)

# Update main function
main_new = """#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let (client, _) = match MixerClient::connect_with_transport(
        &cli.ip,
        &cli.aes50_ip,
        &cli.usb_port,
        &cli.transport,
        false,
    ).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error connecting: {}", e);
            std::process::exit(1);
        }
    };

    let result = match &cli.command {
        Commands::Set { assignments } => handle_set_command(&client, assignments).await,
        Commands::Save { file } => handle_save_command(&client, file).await,
        Commands::Restore { file } => handle_restore_command(&client, file).await,
        Commands::Reset { channels } => handle_reset_command(&client, channels).await,
        Commands::List => handle_list_command(&client).await,
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn query_full_message(client: &MixerClient, path: &str, wait_for_node: Option<&str>) -> Result<OscMessage> {
    let mut rx = client.subscribe();
    client.send_message(path, vec![OscArg::String(wait_for_node.unwrap_or("").to_string())]).await?;
    let start = std::time::Instant::now();
    let timeout_dur = Duration::from_secs(2);
    while start.elapsed() < timeout_dur {
        if let Ok(Ok(msg)) = timeout(timeout_dur - start.elapsed(), rx.recv()).await {
            if let Some(expected_node) = wait_for_node {
                if msg.path == path {
                    if let Some(OscArg::String(response_node)) = msg.args.first() {
                        if response_node == expected_node {
                            return Ok(msg);
                        }
                    }
                }
            } else if msg.path.starts_with(path) {
                return Ok(msg);
            }
        }
    }
    Err(X32Error::from("Timeout waiting for response".to_string()))
}
"""
text = re.sub(r'fn main\(\) \{[\s\S]*?std::process::exit\(1\);\n    \}\n\}', main_new, text)

# Handlers signatures
text = re.sub(r'fn handle_set_command\(ip: &str, assignments_str: &\[String\]\) -> Result<\(\)> \{[\s\n]*let socket = create_socket\(ip, 200\)\?;', 'async fn handle_set_command(client: &MixerClient, assignments_str: &[String]) -> Result<()> {', text)
text = re.sub(r'fn handle_save_command\(ip: &str, file_path: &str\) -> Result<\(\)> \{[\s\n]*let socket = create_socket\(ip, 200\)\?;', 'async fn handle_save_command(client: &MixerClient, file_path: &str) -> Result<()> {', text)
text = re.sub(r'fn handle_restore_command\(ip: &str, file_path: &str\) -> Result<\(\)> \{[\s\n]*let socket = create_socket\(ip, 200\)\?;', 'async fn handle_restore_command(client: &MixerClient, file_path: &str) -> Result<()> {', text)
text = re.sub(r'fn handle_reset_command\(ip: &str, channels_str: &str\) -> Result<\(\)> \{[\s\n]*let socket = create_socket\(ip, 200\)\?;', 'async fn handle_reset_command(client: &MixerClient, channels_str: &str) -> Result<()> {', text)
text = re.sub(r'fn handle_list_command\(ip: &str\) -> Result<\(\)> \{[\s\n]*let socket = create_socket\(ip, 200\)\?;', 'async fn handle_list_command(client: &MixerClient) -> Result<()> {', text)

# get_node_state
get_node_old = r"""fn get_node_state(socket: &UdpSocket, node: &str) -> Result<String> {
    let msg = OscMessage::new("/node".to_string(), vec![OscArg::String(node.to_string())]);
    socket.send(&msg.to_bytes()?)?;

    let mut buf = [0; 512];
    for _ in 0..10 {
        // Retry loop
        match socket.recv(&mut buf) {
            Ok(len) => {
                if len == 0 {
                    continue;
                }
                let response = OscMessage::from_bytes(&buf[..len])?;
                if response.path == "/node" {
                    if let Some(OscArg::String(response_node)) = response.args.first() {
                        if response_node == node {
                            return format_node_state(&response.args);
                        }
                    }
                }
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut
                {
                    continue; // Timeout, try again
                }
                return Err(e.into()); // Other error
            }
        }
    }
    Err(X32Error::from(format!(
        "Timeout waiting for node {}",
        node
    )))
}"""
get_node_new = r"""async fn get_node_state(client: &MixerClient, node: &str) -> Result<String> {
    let msg = query_full_message(client, "/node", Some(node)).await?;
    format_node_state(&msg.args)
}"""
text = text.replace(get_node_old, get_node_new)

# get_source_name
get_src_old = r"""fn get_source_name(socket: &UdpSocket, channel: u8) -> Result<String> {
    let expected_response_prefix = format!("/ch/{:02}/config", channel);

    let msg = OscMessage::new(expected_response_prefix.clone(), vec![]);
    socket.send(&msg.to_bytes()?)?;

    let mut buf = [0; 512];
    for _ in 0..10 {
        // Retry loop
        match socket.recv(&mut buf) {
            Ok(len) => {
                if len == 0 {
                    continue;
                }
                let response = OscMessage::from_bytes(&buf[..len])?;
                if response.path.starts_with(&expected_response_prefix) {
                    if let Some(OscArg::Int(source_id)) = response.args.first() {
                        return Ok(map_source_id_to_name(*source_id).to_string());
                    }
                }
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut
                {
                    continue;
                }
                return Err(e.into());
            }
        }
    }
    Err(X32Error::from("Timeout waiting for source config".to_string()))
}"""
get_src_new = r"""async fn get_source_name(client: &MixerClient, channel: u8) -> Result<String> {
    let expected_response_prefix = format!("/ch/{:02}/config", channel);
    let mut rx = client.subscribe();
    client.send_message(&expected_response_prefix, vec![]).await?;

    let start = std::time::Instant::now();
    let timeout_dur = Duration::from_secs(2);
    while start.elapsed() < timeout_dur {
        if let Ok(Ok(msg)) = timeout(timeout_dur - start.elapsed(), rx.recv()).await {
            if msg.path.starts_with(&expected_response_prefix) {
                if let Some(OscArg::Int(source_id)) = msg.args.first() {
                    return Ok(map_source_id_to_name(*source_id).to_string());
                }
            }
        }
    }
    Err(X32Error::from("Timeout waiting for source config".to_string()))
}"""
text = text.replace(get_src_old, get_src_new)

# socket.send replacements
text = re.sub(r'socket\.send\(&OscMessage::new\(([^,]+), vec!\[\]\)\.to_bytes\(\)\?\)\?;', r'client.send_message(&\1, vec![]).await?;', text)
text = re.sub(r'socket\.send\(&msg\.to_bytes\(\)\?\)\?;', r'client.send_message(&msg.path, msg.args.clone()).await?;', text)
text = re.sub(r'socket\.send\(&OscMessage::from_str\(&line\)\?\.to_bytes\(\)\?\)\?;', r'{ let m = OscMessage::from_str(&line)?; client.send_message(&m.path, m.args).await?; }', text)
text = re.sub(r'socket\.send\(&OscMessage::from_str\(line\)\?\.to_bytes\(\)\?\)\?;', r'{ let m = OscMessage::from_str(line)?; client.send_message(&m.path, m.args).await?; }', text)

# Fix await for get_node_state and get_source_name
text = text.replace("get_node_state(&socket, node)?", "get_node_state(client, node).await?")
text = text.replace("get_node_state(&socket, &node)?", "get_node_state(client, &node).await?")
text = text.replace("get_source_name(&socket, *src)?", "get_source_name(client, *src).await?")
text = text.replace("get_source_name(&socket, channel)?", "get_source_name(client, channel).await?")
text = text.replace("get_source_name(&socket, i)?", "get_source_name(client, i).await?")

with open("tools/x32_custom_layer/src/main.rs", "w") as f:
    f.write(text)
