
use clap::Parser;
use x32_lib::{create_socket, Result};
use osc_lib::{OscMessage, OscArg};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The IP address of the X32 console.
    #[arg(short, long, default_value = "192.168.0.64")]
    ip: String,

    /// The OSC path to query.
    #[arg(index = 1)]
    path: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let socket = create_socket(&args.ip, 100)?;

    let msg = OscMessage::new("/node".to_string(), vec![OscArg::String(args.path)]);
    socket.send(&msg.to_bytes()?)?;

    let mut buf = [0; 512];
    let len = socket.recv(&mut buf)?;
    let response = OscMessage::from_bytes(&buf[..len])?;

    let mut output = response.path.clone();
    if let Some(OscArg::String(s)) = response.args.get(0) {
        output.push_str(" ");
        output.push_str(s);
    }

    println!("{}", output);

    Ok(())
}
