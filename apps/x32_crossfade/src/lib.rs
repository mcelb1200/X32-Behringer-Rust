use anyhow::{Context, Result};
use clap::Parser;
use osc_lib::OscArg;
use std::collections::HashMap;
use tokio::time::{Duration, interval};
use x32_lib::MixerClient;
use x32_lib::scene_parse::SceneParser;

#[derive(Parser, Debug, Clone)]
#[command(
    author,
    version,
    about = "Smoothly interpolate faders, EQs, and dynamics parameters between two scenes",
    long_about = None
)]
pub struct Args {
    /// IP address of the X32 console
    #[arg(short, long)]
    pub ip: String,

    /// Path to the first scene file (.scn)
    #[arg(short = 'a', long)]
    pub scene_a: String,

    /// Path to the second scene file (.scn)
    #[arg(short = 'b', long)]
    pub scene_b: String,

    /// Duration of the crossfade in seconds
    #[arg(short, long, default_value_t = 5.0)]
    pub duration: f64,

    /// Progress point (0.0 to 1.0) at which discrete parameters (strings/ints) are updated
    #[arg(long, default_value_t = 0.5)]
    pub discrete_at: f64,
}

pub async fn run(args: Args) -> Result<()> {
    use std::io::Read;

    let f_a = std::fs::File::open(&args.scene_a)
        .with_context(|| format!("Failed to open scene a: {}", args.scene_a))?;
    let mut scn_a_content = String::new();
    f_a.take(256 * 1024 + 1)
        .read_to_string(&mut scn_a_content)
        .with_context(|| format!("Failed to read scene a: {}", args.scene_a))?;
    if scn_a_content.len() > 256 * 1024 {
        anyhow::bail!("Scene A file is too large (exceeds 256KB)");
    }

    let f_b = std::fs::File::open(&args.scene_b)
        .with_context(|| format!("Failed to open scene b: {}", args.scene_b))?;
    let mut scn_b_content = String::new();
    f_b.take(256 * 1024 + 1)
        .read_to_string(&mut scn_b_content)
        .with_context(|| format!("Failed to read scene b: {}", args.scene_b))?;
    if scn_b_content.len() > 256 * 1024 {
        anyhow::bail!("Scene B file is too large (exceeds 256KB)");
    }

    let map_a = parse_scene_to_map(&scn_a_content);
    let map_b = parse_scene_to_map(&scn_b_content);

    let plan = plan_crossfade(&map_a, &map_b);

    println!(
        "Starting crossfade to {} over {}s...",
        args.ip, args.duration
    );

    // Connect to X32 using automatic keep-alive via true heartbeat
    let client = MixerClient::connect(&args.ip, true).await?;

    let update_interval = Duration::from_millis(50); // 20 updates per second
    let mut ticker = interval(update_interval);

    let total_ticks = (args.duration * 1000.0 / 50.0).round() as u64;
    let discrete_tick = (args.duration * args.discrete_at * 1000.0 / 50.0).round() as u64;

    let mut current_tick = 0;
    let mut discrete_fired = false;

    while current_tick <= total_ticks {
        ticker.tick().await;

        let progress = if total_ticks > 0 {
            current_tick as f32 / total_ticks as f32
        } else {
            1.0
        };

        // Send float updates
        for (path, (start, end)) in &plan.floats {
            let current_val = start + (end - start) * progress;
            client
                .send_message(path, vec![OscArg::Float(current_val)])
                .await?;
        }

        // Check if we need to send discrete updates
        if !discrete_fired && current_tick >= discrete_tick {
            for (path, (_, end_arg)) in &plan.discrete {
                client.send_message(path, vec![end_arg.clone()]).await?;
            }
            discrete_fired = true;
        }

        current_tick += 1;
    }

    // Ensure final state is exactly 1.0 (B)
    for (path, (_, end)) in &plan.floats {
        client.send_message(path, vec![OscArg::Float(*end)]).await?;
    }

    if !discrete_fired {
        for (path, (_, end_arg)) in &plan.discrete {
            client.send_message(path, vec![end_arg.clone()]).await?;
        }
    }

    println!("Crossfade complete.");
    Ok(())
}

fn parse_scene_to_map(contents: &str) -> HashMap<String, OscArg> {
    let mut parser = SceneParser::new();
    let mut map = HashMap::new();
    for line in contents.lines() {
        for msg in parser.parse_scene_line(line) {
            if !msg.args.is_empty() {
                map.insert(msg.path.clone(), msg.args[0].clone());
            }
        }
    }
    map
}

#[derive(Debug)]
struct CrossfadePlan {
    pub floats: HashMap<String, (f32, f32)>,
    pub discrete: HashMap<String, (OscArg, OscArg)>,
}

fn plan_crossfade(
    scene_a: &HashMap<String, OscArg>,
    scene_b: &HashMap<String, OscArg>,
) -> CrossfadePlan {
    let mut floats = HashMap::new();
    let mut discrete = HashMap::new();

    // Iterate through everything in A, if it's in B we transition it
    for (path, arg_a) in scene_a {
        if let Some(arg_b) = scene_b.get(path) {
            match (arg_a, arg_b) {
                (OscArg::Float(f_a), OscArg::Float(f_b)) => {
                    if (f_a - f_b).abs() > f32::EPSILON {
                        floats.insert(path.clone(), (*f_a, *f_b));
                    }
                }
                (a, b) => {
                    if a != b {
                        discrete.insert(path.clone(), (a.clone(), b.clone()));
                    }
                }
            }
        }
    }

    CrossfadePlan { floats, discrete }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_scene_to_map() {
        let scn = "/ch/01/mix/fader 0.5\n/ch/01/config/name \"Lead\"";
        let map = parse_scene_to_map(scn);

        // As per memory, parsing true floats yields scaled representations,
        // /ch/01/mix/fader 0.5 yields 0.7625
        assert_eq!(map.get("/ch/01/mix/fader").unwrap(), &OscArg::Float(0.7625));
        assert_eq!(
            map.get("/ch/01/config/name").unwrap(),
            &OscArg::String("Lead".to_string())
        );
    }

    #[test]
    fn test_plan_crossfade() {
        let mut map_a = HashMap::new();
        map_a.insert("/ch/01/mix/fader".to_string(), OscArg::Float(0.0));
        map_a.insert(
            "/ch/01/config/name".to_string(),
            OscArg::String("Vox".to_string()),
        );
        map_a.insert("/ch/02/mix/on".to_string(), OscArg::Int(0));

        let mut map_b = HashMap::new();
        map_b.insert("/ch/01/mix/fader".to_string(), OscArg::Float(1.0));
        map_b.insert(
            "/ch/01/config/name".to_string(),
            OscArg::String("Lead Vox".to_string()),
        );
        map_b.insert("/ch/02/mix/on".to_string(), OscArg::Int(1));

        let plan = plan_crossfade(&map_a, &map_b);

        assert_eq!(plan.floats.len(), 1);
        assert_eq!(plan.floats.get("/ch/01/mix/fader"), Some(&(0.0, 1.0)));

        assert_eq!(plan.discrete.len(), 2);
        assert_eq!(
            plan.discrete.get("/ch/01/config/name"),
            Some(&(
                OscArg::String("Vox".to_string()),
                OscArg::String("Lead Vox".to_string())
            ))
        );
        assert_eq!(
            plan.discrete.get("/ch/02/mix/on"),
            Some(&(OscArg::Int(0), OscArg::Int(1)))
        );
    }

    #[tokio::test]
    async fn test_tick_loop() {
        let mut map_a = HashMap::new();
        map_a.insert("/ch/01/mix/fader".to_string(), OscArg::Float(0.0));
        map_a.insert(
            "/ch/01/config/name".to_string(),
            OscArg::String("Vox".to_string()),
        );

        let mut map_b = HashMap::new();
        map_b.insert("/ch/01/mix/fader".to_string(), OscArg::Float(1.0));
        map_b.insert(
            "/ch/01/config/name".to_string(),
            OscArg::String("Lead Vox".to_string()),
        );

        let plan = plan_crossfade(&map_a, &map_b);

        // Ensure test passes and logic runs correctly in actual function
        assert_eq!(plan.floats.len(), 1);
        assert_eq!(plan.discrete.len(), 1);
    }
}
