import re

with open("tools/x32_get_lib/src/main.rs", "r") as f:
    text = f.read()

# Add tokio dep import
text = text.replace("use x32_lib::{\n    create_socket,\n    error::{Result, X32Error},\n};", "use x32_lib::{MixerClient, error::{Result, X32Error}};\nuse tokio::time::{timeout, Duration};\n")
text = text.replace("use x32_lib::{create_socket, error::Result};", "use x32_lib::{MixerClient, error::Result, error::X32Error};\nuse tokio::time::{timeout, Duration};\n")

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
text = re.sub(r'struct Args \{[\s\n]*/// The IP address of the X32 console[\s\n]*#\[arg\(short, long, default_value = "192.168.1.62"\)\][\s\n]*ip: String,', cli_repl, text)

# Modify main signature
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
    let client = std::sync::Arc::new(client);
"""
text = text.replace(main_old, main_new)

# Modify main processing
text = text.replace("socket.send(&msg.to_bytes()?)?;", "client.send_message(&msg.path, msg.args).await?;")
text = text.replace("if let Ok(len) = socket.recv(&mut buf) {", "if let Ok(Ok(resp)) = timeout(Duration::from_millis(50), rx.recv()).await {")
text = text.replace("let resp = OscMessage::from_bytes(&buf[..len])?;", "")
text = text.replace("process_lib_slot(&socket, t, i, &args.output_dir, args.verbose)?;", "process_lib_slot(&client, t, i, &args.output_dir, args.verbose).await?;")
# wait, main needs `let mut rx = client.subscribe();` before the loop.
text = re.sub(r'for t in types \{', 'let mut rx = client.subscribe();\n    for t in types {', text)

# Modify process_lib_slot signature
ps_old = "fn process_lib_slot(\n    socket: &std::net::UdpSocket,\n    t: LibType,\n    id: i32,\n    out_dir: &Path,\n    _verbose: bool,\n) -> Result<()> {"
ps_new = "async fn process_lib_slot(\n    client: &MixerClient,\n    t: LibType,\n    id: i32,\n    out_dir: &Path,\n    _verbose: bool,\n) -> Result<()> {\n    let mut rx = client.subscribe();"
text = text.replace(ps_old, ps_new)

# Modify process_lib_slot socket calls
text = text.replace("socket.send(&msg.to_bytes()?)?;", "client.send_message(&msg.path, msg.args.clone()).await?;")
text = re.sub(r'let len = socket\.recv\(&mut buf\)\?;\n\s*let resp = OscMessage::from_bytes\(&buf\[\.\.len\]\)\?;',
r'''let resp = match timeout(Duration::from_millis(500), rx.recv()).await {
        Ok(Ok(m)) => m,
        _ => return Err(X32Error::from("Timeout waiting for node").into()),
    };''', text)

text = re.sub(r'socket\.set_read_timeout\(Some\(Duration::from_millis\(200\)\)\)\?;\n\s*if let Ok\(_len\) = socket\.recv\(&mut buf\) \{\n\s*// Assume success for now\n\s*\}',
r'''let _ = timeout(Duration::from_millis(200), rx.recv()).await;''', text)

text = re.sub(r'socket\.set_read_timeout\(Some\(Duration::from_millis\(500\)\)\)\?;', '', text)

# Fix loop in process_lib_slot
for_loop_recv = r"""        if let Ok(Ok(resp)) = timeout(Duration::from_millis(500), rx.recv()).await {
            if resp.path == "/node" || resp.path == "node" {
                if let Some(OscArg::String(val)) = resp.args.first() {
                    let mut output = val.clone();

                    match t {
                        LibType::Channel => {
                            // Strip "/ch/01" from the beginning
                            if let Some(stripped) = output.strip_prefix("/ch/01").or_else(|| output.strip_prefix("ch/01")) {
                                output = stripped.to_string();
                            }

                            // Remove the "source" element of /config
                            if i == 0 {
                                if let Some(last_space) = output.rfind(' ') {
                                    output.truncate(last_space);
                                }
                            }
                            writeln!(file, "{}", output.trim_start())?;
                        }
                        LibType::Effects => {
                            // Strip "/fx/1/" from the beginning
                            if let Some(stripped) = output.strip_prefix("/fx/1/").or_else(|| output.strip_prefix("fx/1/")) {
                                output = stripped.to_string();
                            }
                            writeln!(file, "{}", output.trim_start())?;
                        }
                        LibType::Routing => {
                            writeln!(file, "{}", output.trim_start())?;
                        }
                        _ => {}
                    }
                }
            }
        } else {
            eprintln!("  Error or timeout on command: /node ,s {}", p);
        }"""
text = re.sub(r'        if let Ok\(len\) = socket\.recv\(&mut buf\) \{[\s\S]*?            eprintln!\("  Error or timeout on command: /node ,s \{\}", p\);\n        \}', for_loop_recv, text)

headamp_recv = r"""        if let Ok(Ok(resp)) = timeout(Duration::from_millis(500), rx.recv()).await {
            if resp.path == "/node" || resp.path == "node" {
                if let Some(OscArg::String(val)) = resp.args.first() {
                    writeln!(file, "{}", val)?;
                }
            }
        } else {
            eprintln!("  Error or timeout on command: /node ,s headamp/000");
        }"""
text = re.sub(r'        if let Ok\(len\) = socket\.recv\(&mut buf\) \{[\s\S]*?            eprintln!\("  Error or timeout on command: /node ,s headamp/000"\);\n        \}', headamp_recv, text)

with open("tools/x32_get_lib/src/main.rs", "w") as f:
    f.write(text)
