import re

with open("tools/x32_set_lib/src/main.rs", "r") as f:
    text = f.read()

text = text.replace("use x32_lib::create_socket;", "use x32_lib::{MixerClient, error::X32Error};\nuse tokio::time::{timeout, Duration};\nuse std::io::Read;")
text = text.replace("use std::net::UdpSocket;\n", "")

# Modify Args
cli_repl = """struct Args {
    #[arg(short, long, default_value = "192.168.1.62")]
    ip: String,

    #[arg(long, default_value = "auto")]
    transport: String,

    #[arg(long, default_value = "")]
    usb_port: String,

    #[arg(long, default_value = "")]
    aes50_ip: String,"""
text = re.sub(r'struct Args \{[\s\n]*/// IP address of the X32 console[\s\n]*#\[arg\(short, long, default_value = "192.168.1.62"\)\][\s\n]*ip: String,', cli_repl, text)

# main signature
main_old = r"""fn main() -> Result<()> {
    let args = Args::parse();
    let socket = create_socket(&args.ip, 500)?;"""
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
    let client = std::sync::Arc::new(client);"""
text = text.replace(main_old, main_new)

text = text.replace("process_file(&socket, file_path, args.verbose)?;", "process_file(&client, file_path, args.verbose).await?;")
text = text.replace("fn process_file(socket: &UdpSocket, path: &Path, verbose: bool) -> Result<()> {", "async fn process_file(client: &MixerClient, path: &Path, verbose: bool) -> Result<()> {")

text = text.replace("socket.send(&msg.to_bytes()?)?;", "client.send_message(&msg.path, msg.args.clone()).await?;")
# the original also does socket.recv
text = re.sub(r'let _ = socket\.recv\(&mut buf\);', 'let mut rx = client.subscribe(); let _ = timeout(Duration::from_millis(100), rx.recv()).await;', text)

with open("tools/x32_set_lib/src/main.rs", "w") as f:
    f.write(text)
