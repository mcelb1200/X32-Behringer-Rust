use anyhow::{Context, Result};
use clap::Parser;
use osc_lib::{OscArg, OscMessage};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::interval;
use x32_lib::{scene_parse::SceneParser, MixerClient};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, default_value = "192.168.1.64")]
    pub ip: String,

    #[arg(long, default_value = "auto")]
    pub transport: String,

    #[arg(long, default_value = "")]
    pub usb_port: String,

    #[arg(long, default_value = "")]
    pub aes50_ip: String,

    #[arg(index = 1)]
    pub file1: PathBuf,

    #[arg(index = 2)]
    pub file2: PathBuf,

    #[arg(short, long, default_value = "5.0")]
    pub duration: f32,

    #[arg(short, long, default_value = "0.05")]
    pub step: f32,

    #[arg(long, default_value = "0.5")]
    pub discrete_at: f32,
}

pub struct SceneData {
    pub floats: HashMap<String, f32>,
    pub discrete: Vec<OscMessage>,
}

pub fn load_scene_file(path: &PathBuf) -> Result<SceneData> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let mut parser = SceneParser::new();
    let mut floats = HashMap::new();
    let mut discrete = Vec::new();

    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let messages = parser.parse_scene_line(line);
        for msg in messages {
            if let Some(OscArg::Float(f)) = msg.args.first() {
                floats.insert(msg.path.clone(), *f);
            } else {
                discrete.push(msg);
            }
        }
    }

    Ok(SceneData { floats, discrete })
}

pub async fn run(args: Args) -> Result<()> {
    println!("Loading scenes...");
    let scene1 = load_scene_file(&args.file1).context("Failed to load first scene")?;
    let scene2 = load_scene_file(&args.file2).context("Failed to load second scene")?;

    println!("Finding differences...");
    let mut transitions = Vec::new();

    for (path, &val2) in &scene2.floats {
        if let Some(&val1) = scene1.floats.get(path) {
            if (val1 - val2).abs() > 0.001 {
                transitions.push((path.clone(), val1, val2));
            }
        }
    }

    println!("Found {} float parameters to interpolate.", transitions.len());

    println!("Connecting to mixer at {}", args.ip);
    let (client, _transport) = MixerClient::connect_with_transport(
        &args.ip,
        &args.aes50_ip,
        &args.usb_port,
        &args.transport,
        false,
    )
    .await?;

    println!("Starting crossfade...");

    let steps = (args.duration / args.step) as usize;
    let mut interval = interval(Duration::from_secs_f32(args.step));
    let mut discrete_sent = false;
    let discrete_step = (steps as f32 * args.discrete_at) as usize;

    for step in 0..=steps {
        interval.tick().await;

        let progress = if steps == 0 { 1.0 } else { step as f32 / steps as f32 };

        if !discrete_sent && step >= discrete_step {
            for msg in &scene2.discrete {
                let _ = client.send_message(&msg.path, msg.args.clone()).await;
            }
            discrete_sent = true;
        }

        for (path, val1, val2) in &transitions {
            let current = val1 + (val2 - val1) * progress;
            let _ = client.send_message(path, vec![OscArg::Float(current)]).await;
        }
    }

    // Ensure discrete sent if duration was 0 or discrete_at > 1.0
    if !discrete_sent {
        for msg in &scene2.discrete {
            let _ = client.send_message(&msg.path, msg.args.clone()).await;
        }
    }

    println!("Crossfade complete.");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_scene_file() {
        let mut file = NamedTempFile::new().unwrap();
        let scene_content = r#"
# Some comment
/ch/01/mix/fader 0.75
/ch/01/mix/on ON
/ch/02/mix/fader 0.5
/ch/02/mix/on OFF
/ch/01/config/name "Vocal"
"#;
        file.write_all(scene_content.as_bytes()).unwrap();

        let path = file.path().to_path_buf();
        let scene_data = load_scene_file(&path).unwrap();

        assert_eq!(scene_data.floats.len(), 2);
        assert_eq!(scene_data.floats.get("/ch/01/mix/fader"), Some(&0.76875));
        assert_eq!(scene_data.floats.get("/ch/02/mix/fader"), Some(&0.7625));

        assert_eq!(scene_data.discrete.len(), 3);

        let on1 = scene_data.discrete.iter().find(|m| m.path == "/ch/01/mix/on").unwrap();
        assert_eq!(on1.args[0], OscArg::Int(1));

        let on2 = scene_data.discrete.iter().find(|m| m.path == "/ch/02/mix/on").unwrap();
        assert_eq!(on2.args[0], OscArg::Int(0));

        let name = scene_data.discrete.iter().find(|m| m.path == "/ch/01/config/name").unwrap();
        assert_eq!(name.args[0], OscArg::String("Vocal".to_string()));
    }

    #[test]
    fn test_interpolation_math() {
        let val1 = 0.0;
        let val2 = 1.0;

        let progress_0 = 0.0;
        assert_eq!(val1 + (val2 - val1) * progress_0, 0.0);

        let progress_50 = 0.5;
        assert_eq!(val1 + (val2 - val1) * progress_50, 0.5);

        let progress_100 = 1.0;
        assert_eq!(val1 + (val2 - val1) * progress_100, 1.0);

        let val3 = 0.75;
        let val4 = 0.25;
        assert_eq!(val3 + (val4 - val3) * 0.5, 0.5);
    }
}
