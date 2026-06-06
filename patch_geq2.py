import re

with open("tools/x32_geq2_cpy/src/main.rs", "r") as f:
    text = f.read()

# Add tokio dep import
text = text.replace("use std::thread;\n", "")
text = text.replace("use std::time::Duration;\n", "use tokio::time::{sleep, Duration};\n")
text = text.replace("use x32_lib::{create_socket, get_parameter, set_parameter, verify_fx_type};", "use x32_lib::{MixerClient, get_parameter_async, set_parameter_async, verify_fx_type_async};\n")

# Modify Args
cli_repl = """struct Args {
    #[arg(short, long, default_value = "192.168.0.64")]
    ip: String,

    #[arg(long, default_value = "auto")]
    transport: String,

    #[arg(long, default_value = "")]
    usb_port: String,

    #[arg(long, default_value = "")]
    aes50_ip: String,"""
text = re.sub(r'struct Args \{[\s\n]*/// IP address of the X32 console\.[\s\n]*#\[arg\(short, long, default_value = "192.168.0.64"\)\][\s\n]*ip: String,', cli_repl, text)

# Modify main signature
main_old = r"""fn main() -> Result<()> {
    let args = Args::parse();

    if args.debug {
        println!("Debug mode is on.");
        println!("Arguments: {:?}", args);
    }

    let socket = create_socket(&args.ip, 1000)?;"""
main_new = r"""#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    if args.debug {
        println!("Debug mode is on.");
        println!("Arguments: {:?}", args);
    }

    let (client, _) = MixerClient::connect_with_transport(
        &args.ip,
        &args.aes50_ip,
        &args.usb_port,
        &args.transport,
        false,
    ).await?;
    let client = std::sync::Arc::new(client);"""
text = text.replace(main_old, main_new)

# Replace verify_fx_type
text = text.replace("verify_fx_type(&socket,", "verify_fx_type_async(&client,")
text = text.replace(", \"EQ\")?", ", \"EQ\").await?")

# Replace get_parameter and set_parameter
text = text.replace("get_parameter(&socket, ", "get_parameter_async(&client, ")
text = text.replace("set_parameter(&socket, ", "set_parameter_async(&client, ")

text = re.sub(r'get_parameter_async\(&client, ([^)]+)\)\?', r'get_parameter_async(&client, \1).await?', text)
text = re.sub(r'set_parameter_async\(&client, ([^,]+), ([^)]+)\)\?', r'set_parameter_async(&client, \1, \2).await?', text)

text = text.replace("thread::sleep(", "sleep(")

with open("tools/x32_geq2_cpy/src/main.rs", "w") as f:
    f.write(text)
