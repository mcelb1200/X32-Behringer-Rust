import re

with open("tools/x32_fade/src/main.rs", "r") as f:
    text = f.read()

text = text.replace("use std::thread;\n", "")
text = text.replace("use x32_lib::{create_socket, get_fader_level};", "use x32_lib::{MixerClient, get_parameter_async};\nuse tokio::time::sleep;\n")

# Modify Args
cli_repl = """struct Args {
    #[arg(short, long)]
    ip: Option<String>,

    #[arg(long, default_value = "auto")]
    transport: String,

    #[arg(long, default_value = "")]
    usb_port: String,

    #[arg(long, default_value = "")]
    aes50_ip: String,"""
text = re.sub(r'struct Args \{[\s\n]*/// IP address of the X32 console\.[\s\n]*#\[arg\(short, long\)\][\s\n]*ip: Option<String>,', cli_repl, text)

# Update Config struct with transport options maybe? The instructions said "CLI Configuration" meaning Args/Cli, but x32_fade has a config. I will add them just in case or just use Args.

# Modify fn main
main_old = r"""fn main() -> Result<()> {"""
main_new = r"""#[tokio::main]
async fn main() -> Result<()> {"""
text = text.replace(main_old, main_new)

# Connect block
con_old = r"""        let socket = create_socket(ip, 1000)?;"""
con_new = r"""        let (client, _) = MixerClient::connect_with_transport(
            ip,
            &args.aes50_ip,
            &args.usb_port,
            &args.transport,
            false,
        ).await?;
        let client = std::sync::Arc::new(client);"""
text = text.replace(con_old, con_new)

text = text.replace("fade(\n                    &socket", "fade(\n                    &client")

text = re.sub(r'fade\([\s\S]*?\)\?;', lambda m: m.group(0).replace(')?;', ').await?;'), text)

# Update fade
fade_old = "fn fade(\n    socket: &std::net::UdpSocket,"
fade_new = "async fn fade(\n    client: &MixerClient,"
text = text.replace(fade_old, fade_new)

text = text.replace("get_fader_level(socket, fader_addr)?", "get_parameter_async(client, fader_addr).await?")
text = text.replace("let buf = msg.to_bytes()?;\n            socket.send(&buf)?;", "client.send_message(&msg.path, msg.args).await?;")

text = text.replace("thread::sleep(step_interval);", "sleep(step_interval).await;")

with open("tools/x32_fade/src/main.rs", "w") as f:
    f.write(text)
