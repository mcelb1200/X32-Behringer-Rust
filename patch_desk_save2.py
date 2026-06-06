import re

with open("tools/x32_desk_save/src/main.rs", "r") as f:
    text = f.read()

main_old = r"""fn main() -> Result<(), X32Error> {
    let args = Args::parse();

    let commands = if args.desk_save {
        nodes::DS_NODE.iter().map(|s| s.to_string()).collect()
    } else if args.scene {
        nodes::SC_NODE.iter().map(|s| s.to_string()).collect()
    } else if args.routing {
        nodes::RO_NODE.iter().map(|s| s.to_string()).collect()
    } else if let Some(pattern_file) = args.pattern_file {
        let file = File::open(pattern_file)?;
        let reader = std::io::BufReader::new(file);
        reader.lines().filter_map(|l| l.ok()).collect()
    } else {
        return Err(X32Error::Custom("No mode selected".to_string()));
    };

    let socket = create_socket(&args.ip, 500)?;
    println!("Successfully connected to X32 at {}", args.ip);

    let data = get_desk_data(&socket, &commands)?;"""

main_new = r"""#[tokio::main]
async fn main() -> Result<(), X32Error> {
    let args = Args::parse();

    let commands: Vec<String> = if args.desk_save {
        nodes::DS_NODE.iter().map(|s| s.to_string()).collect()
    } else if args.scene {
        nodes::SC_NODE.iter().map(|s| s.to_string()).collect()
    } else if args.routing {
        nodes::RO_NODE.iter().map(|s| s.to_string()).collect()
    } else if let Some(pattern_file) = args.pattern_file {
        let file = File::open(pattern_file)?;
        let reader = std::io::BufReader::new(file);
        reader.lines().filter_map(|l| l.ok()).collect()
    } else {
        return Err(X32Error::Custom("No mode selected".to_string()));
    };

    let (client, _) = MixerClient::connect_with_transport(
        &args.ip,
        &args.aes50_ip,
        &args.usb_port,
        &args.transport,
        false,
    ).await?;
    let client = std::sync::Arc::new(client);
    println!("Successfully connected to X32 at {}", args.ip);

    let data = get_desk_data(&client, &commands).await?;"""
text = text.replace(main_old, main_new)

# Update get_desk_data signature
get_desk_old = r"""fn get_desk_data(socket: &UdpSocket, commands: &[String]) -> Result<Vec<String>, X32Error> {"""
get_desk_new = r"""async fn get_desk_data(client: &MixerClient, commands: &[String]) -> Result<Vec<String>, X32Error> {
    let mut rx = client.subscribe();"""
text = text.replace(get_desk_old, get_desk_new)

# Replace socket.send inside get_desk_data
# Original uses socket.recv
text = re.sub(r'socket\.send\(&msg\.to_bytes\(\)\?\)\?;[\s\n]*let mut buf = \[0; 512\];[\s\n]*let mut retries = 0;[\s\n]*loop \{[\s\n]*match socket\.recv\(&mut buf\) \{[\s\S]*?\}',
r'''client.send_message(&msg.path, msg.args).await?;
        if let Ok(Ok(received_msg)) = timeout(Duration::from_millis(500), rx.recv()).await {
            if received_msg.path == "node" || received_msg.path == "/node" {
                if let Some(OscArg::String(s)) = received_msg.args.first() {
                    data.push(s.clone());
                }
            }
        } else {
            eprintln!("Timeout waiting for node: {}", node);
        }''', text)


with open("tools/x32_desk_save/src/main.rs", "w") as f:
    f.write(text)
