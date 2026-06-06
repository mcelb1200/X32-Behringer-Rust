import re

with open("tools/x32_tap/src/main.rs", "r") as f:
    text = f.read()

# I will replace the main block.
main_old = r"""#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let std_socket = create_socket(&args.ip, 500).context("Failed to create socket")?;
    std_socket.set_nonblocking(true)?;
    let socket = tokio::net::UdpSocket::from_std(std_socket)?;

    handle_client(Arc::new(socket), args).await
}"""
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
    let client = Arc::new(client);

    handle_client(client, args).await
}"""
text = text.replace(main_old, main_new)

with open("tools/x32_tap/src/main.rs", "w") as f:
    f.write(text)
