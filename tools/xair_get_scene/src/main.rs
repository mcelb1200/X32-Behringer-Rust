use anyhow::Result;
use clap::Parser;
use osc_lib::{OscArg, OscMessage};
use std::io::{self, BufRead, Read};
use x32_lib::MixerClient;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    ip: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let client = MixerClient::connect(&args.ip, true).await?;

    let stdin = io::stdin();
    let mut stdin_lock = stdin.lock();
    let mut line = String::new();

    loop {
        line.clear();
        let len = stdin_lock.by_ref().take(4096).read_line(&mut line)?;
        if len == 0 {
            break;
        }
        let line = line.trim();
        if line.starts_with('/') {
            let msg = OscMessage::new("/node".to_string(), vec![OscArg::String(line.to_string())]);
            client.send_message(&msg.path, msg.args.clone()).await?;

            if let Ok(arg) = client.query_value(line).await {
                let response = OscMessage::new(line.to_string(), vec![arg]);
                let mut output = response.path.clone();
                if let Some(OscArg::String(s)) = response.args.first() {
                    output.push(' ');
                    output.push_str(s);
                }
                println!("{}", output);
            }
        }
    }

    Ok(())
}
