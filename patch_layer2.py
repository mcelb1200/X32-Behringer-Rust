import re

with open("tools/x32_custom_layer/src/main.rs", "r") as f:
    content = f.read()

# Replace std::net::UdpSocket
content = content.replace("use std::net::UdpSocket;\n", "")
content = content.replace("use x32_lib::{\n    create_socket,\n    error::{Result, X32Error},\n};", "use x32_lib::{MixerClient, error::{Result, X32Error}};\nuse tokio::time::{timeout, Duration};")

# Update Cli
cli_replacement = """#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
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
content = re.sub(r'#\[derive\(Parser\)\]\n#\[command\(author, version, about, long_about = None\)\]\nstruct Cli \{\n    #\[clap\(subcommand\)\]\n    command: Commands,\n\}', cli_replacement, content)

# Remove ip from Commands
content = re.sub(r'/// The IP address of the X32 mixer\s+ip: String,\s+', '', content)

# Update fn main
main_old = r"""fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Set { ip, assignments } => handle_set_command(ip, assignments),
        Commands::Save { ip, file } => handle_save_command(ip, file),
        Commands::Restore { ip, file } => handle_restore_command(ip, file),
        Commands::Reset { ip, channels } => handle_reset_command(ip, channels),
        Commands::List { ip } => handle_list_command(ip),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}"""

main_new = r"""#[tokio::main]
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
}"""
content = content.replace(main_old, main_new)

# Update signatures and replace socket
content = re.sub(r'fn handle_set_command\(ip: &str, assignments_str: &\[String\]\) -> Result<\(\)> \{[\s\S]*?let socket = create_socket\(ip, 200\)\?;', 'async fn handle_set_command(client: &MixerClient, assignments_str: &[String]) -> Result<()> {', content)
content = re.sub(r'fn handle_save_command\(ip: &str, file_path: &str\) -> Result<\(\)> \{[\s\S]*?let socket = create_socket\(ip, 200\)\?;', 'async fn handle_save_command(client: &MixerClient, file_path: &str) -> Result<()> {', content)
content = re.sub(r'fn handle_restore_command\(ip: &str, file_path: &str\) -> Result<\(\)> \{[\s\S]*?let socket = create_socket\(ip, 200\)\?;', 'async fn handle_restore_command(client: &MixerClient, file_path: &str) -> Result<()> {', content)
content = re.sub(r'fn handle_reset_command\(ip: &str, channels_str: &str\) -> Result<\(\)> \{[\s\S]*?let socket = create_socket\(ip, 200\)\?;', 'async fn handle_reset_command(client: &MixerClient, channels_str: &str) -> Result<()> {', content)
content = re.sub(r'fn handle_list_command\(ip: &str\) -> Result<\(\)> \{[\s\S]*?let socket = create_socket\(ip, 200\)\?;', 'async fn handle_list_command(client: &MixerClient) -> Result<()> {', content)

# update ask_parameter / get_node_state etc
content = content.replace("socket.send(&OscMessage::new(node.to_string(), vec![]).to_bytes()?)?;", "client.send_message(node, vec![]).await?;")
content = content.replace("socket.send(&msg.to_bytes()?)?;", "client.send_message(&msg.path, msg.args.clone()).await?;")
content = content.replace("socket.send(&OscMessage::from_str(&line)?.to_bytes()?)?;", "{ let m = OscMessage::from_str(&line)?; client.send_message(&m.path, m.args).await?; }")
content = content.replace("get_node_state(&socket, node)?", "get_node_state(client, node).await?")
content = content.replace("get_node_state(&socket, &node)?", "get_node_state(client, &node).await?")
content = content.replace("get_source_name(&socket, *src)?", "get_source_name(client, *src).await?")
content = content.replace("get_source_name(&socket, channel)?", "get_source_name(client, channel).await?")
content = content.replace("get_source_name(&socket, i)?", "get_source_name(client, i).await?")

get_node_old = r"""fn get_node_state(socket: &UdpSocket, node: &str) -> Result<String> {
    let msg = OscMessage::new(node.to_string(), vec![]);
    socket.send(&msg.to_bytes()?)?;
    let mut buf = [0; 512];
    loop {
        let (len, _) = socket.recv_from(&mut buf)?;
        let received_msg = OscMessage::from_bytes(&buf[..len])?;
        if received_msg.path == node {
            return format_node_state(&received_msg.args);
        }
    }
}"""

get_node_new = r"""async fn get_node_state(client: &MixerClient, node: &str) -> Result<String> {
    let mut rx = client.subscribe();
    client.send_message(node, vec![]).await?;

    let t = Duration::from_secs(2);
    let start = std::time::Instant::now();
    while start.elapsed() < t {
        if let Ok(Ok(msg)) = timeout(t - start.elapsed(), rx.recv()).await {
            if msg.path == node {
                return format_node_state(&msg.args);
            }
        }
    }
    Err(X32Error::from("Timeout".to_string()))
}"""
content = content.replace(get_node_old, get_node_new)

get_src_old = r"""fn get_source_name(socket: &UdpSocket, channel: u8) -> Result<String> {
    let node = format!("/ch/{:02}/config", channel);
    let msg = OscMessage::new(node.clone(), vec![]);
    socket.send(&msg.to_bytes()?)?;

    let mut buf = [0; 512];
    loop {
        let (len, _) = socket.recv_from(&mut buf)?;
        let received_msg = OscMessage::from_bytes(&buf[..len])?;
        if received_msg.path == node {
            if let Some(OscArg::Int(id)) = received_msg.args.get(2) {
                return Ok(map_source_id_to_name(*id).to_string());
            } else {
                return Err(X32Error::from("Unexpected config format".to_string()));
            }
        }
    }
}"""
get_src_new = r"""async fn get_source_name(client: &MixerClient, channel: u8) -> Result<String> {
    let node = format!("/ch/{:02}/config", channel);
    let mut rx = client.subscribe();
    client.send_message(&node, vec![]).await?;

    let t = Duration::from_secs(2);
    let start = std::time::Instant::now();
    while start.elapsed() < t {
        if let Ok(Ok(msg)) = timeout(t - start.elapsed(), rx.recv()).await {
            if msg.path == node {
                if let Some(OscArg::Int(id)) = msg.args.get(2) {
                    return Ok(map_source_id_to_name(*id).to_string());
                } else {
                    return Err(X32Error::from("Unexpected config format".to_string()));
                }
            }
        }
    }
    Err(X32Error::from("Timeout".to_string()))
}"""
content = content.replace(get_src_old, get_src_new)

# handle_restore_command and handle_reset_command use `socket.recv` loop, we need to change it
# since we can't easily replace the whole block, I will just write a specific regex or replace
content = re.sub(r'        match socket\.recv\(&mut buf\) \{[\s\S]*?            \}\n        \}\n', r'        let _ = timeout(Duration::from_millis(10), client.subscribe().recv()).await;', content)

with open("tools/x32_custom_layer/src/main.rs", "w") as f:
    f.write(content)
