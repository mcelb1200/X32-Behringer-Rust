
use clap::Parser;
use std::io::{self, BufRead};
use x32_lib::{create_socket, error::Result};
use osc_lib::{OscMessage, OscArg};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The IP address of the X32 console.
    #[arg(short, long, default_value = "192.168.0.64")]
    ip: String,

    /// Scene name.
    #[arg(short, long)]
    scene_name: Option<String>,

    /// Note data.
    #[arg(short, long)]
    note: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let socket = create_socket(&args.ip, 10024, 10023, 100)?;

    let scene_name = match args.scene_name {
        Some(name) => name,
        None => {
            println!("Please enter scene name: ");
            let mut name = String::new();
            io::stdin().read_line(&mut name)?;
            name.trim().to_string()
        }
    };

    let note = match args.note {
        Some(note) => note,
        None => {
            println!("Please enter note data: ");
            let mut note = String::new();
            io::stdin().read_line(&mut note)?;
            note.trim().to_string()
        }
    };

    println!("#2.7# \"{}\" \"{}\" %000000000 1 X32GetScene V1.5 (c)2014 Patrick-Gilles Maillot\n", scene_name, note);

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.starts_with('/') {
            let msg = OscMessage::new("/node".to_string(), vec![OscArg::String(line)]);
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
        }
    }

    Ok(())
}
