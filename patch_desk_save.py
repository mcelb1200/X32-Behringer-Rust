import re

with open("tools/x32_desk_save/src/main.rs", "r") as f:
    text = f.read()

text = text.replace("use std::net::UdpSocket;\n", "")
text = text.replace("use x32_lib::{create_socket, error::X32Error};", "use x32_lib::{MixerClient, error::X32Error};\nuse tokio::time::{timeout, Duration};\n")

# Replace struct Args
cli_repl = """struct Args {
    #[arg(short, long, default_value = "192.168.1.64")]
    ip: String,

    #[arg(long, default_value = "auto")]
    transport: String,

    #[arg(long, default_value = "")]
    usb_port: String,

    #[arg(long, default_value = "")]
    aes50_ip: String,"""
text = re.sub(r'struct Args \{[\s\n]*/// X32 console IP address[\s\n]*#\[arg\(short, long, default_value = "192.168.1.64"\)\][\s\n]*ip: String,', cli_repl, text)


# Update fn main
main_old = r"""fn main() -> Result<(), X32Error> {
    let args = Args::parse();

    let file = File::create(&args.file)?;
    let mut file = BufWriter::new(file);

    let socket = create_socket(&args.ip, 500)?;
    println!("Successfully connected to X32 at {}", args.ip);"""

main_new = r"""#[tokio::main]
async fn main() -> Result<(), X32Error> {
    let args = Args::parse();

    let file = File::create(&args.file)?;
    let mut file = BufWriter::new(file);

    let (client, _) = MixerClient::connect_with_transport(
        &args.ip,
        &args.aes50_ip,
        &args.usb_port,
        &args.transport,
        false,
    ).await?;
    let client = std::sync::Arc::new(client);

    println!("Successfully connected to X32 at {}", args.ip);"""

text = text.replace(main_old, main_new)

text = text.replace("save_desk_state(&socket, &mut file)?;", "save_desk_state(&client, &mut file).await?;")
text = text.replace("fn save_desk_state(socket: &UdpSocket, file: &mut BufWriter<File>) -> Result<(), X32Error> {", "async fn save_desk_state(client: &MixerClient, file: &mut BufWriter<File>) -> Result<(), X32Error> {\n    let mut rx = client.subscribe();")

# Replace socket.send and loop recv
fetch_node_old = r"""        let msg = OscMessage::new("/node".to_string(), vec![OscArg::String(node.to_string())]);
        socket.send(&msg.to_bytes()?)?;

        let mut buf = [0; 512];
        let mut retries = 0;
        loop {
            match socket.recv(&mut buf) {
                Ok(len) => {
                    let received_msg = OscMessage::from_bytes(&buf[..len])?;
                    if received_msg.path == "node" {
                        if let Some(OscArg::String(s)) = received_msg.args.first() {
                            // C implementation trick: write string offset by 12 bytes
                            // Wait, OscMessage parses it. Let's just output it directly or reformat.
                            // The C code writes: r_buf + 12. This skips "/node ,s~~~~".
                            // If `s` contains the node name + value, we should print it.
                            writeln!(file, "{}", s)?;
                            break;
                        }
                    }
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut
                    {
                        retries += 1;
                        if retries > 3 {
                            eprintln!("Timeout waiting for node: {}", node);
                            break;
                        }
                        continue;
                    }
                    return Err(X32Error::Io(e));
                }
            }
        }"""
fetch_node_new = r"""        let msg = OscMessage::new("/node".to_string(), vec![OscArg::String(node.to_string())]);
        client.send_message(&msg.path, msg.args).await?;

        if let Ok(Ok(received_msg)) = timeout(Duration::from_millis(500), rx.recv()).await {
            if received_msg.path == "node" || received_msg.path == "/node" {
                if let Some(OscArg::String(s)) = received_msg.args.first() {
                    writeln!(file, "{}", s)?;
                }
            }
        } else {
            eprintln!("Timeout waiting for node: {}", node);
        }"""
text = text.replace(fetch_node_old, fetch_node_new)

with open("tools/x32_desk_save/src/main.rs", "w") as f:
    f.write(text)
