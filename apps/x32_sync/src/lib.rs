use anyhow::Result;
use clap::Parser;
use osc_lib::OscArg;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use x32_fxparse::parse_parameter;
use x32_lib::{MixerClient, MixerModel};

#[derive(Parser, Debug)]
#[command(author, version, about = "Bidirectional synchronization tool for digital mixer consoles", long_about = None)]
pub struct Args {
    /// IP address of Console A
    #[arg(short = 'a', long)]
    pub ip_a: String,

    /// IP address of Console B
    #[arg(short = 'b', long)]
    pub ip_b: String,

    /// Default mixer model for both consoles (default: X32)
    #[arg(short = 'M', long, default_value = "X32")]
    pub model: MixerModel,

    /// Mixer model for Console A (overrides --model)
    #[arg(long)]
    pub model_a: Option<MixerModel>,

    /// Mixer model for Console B (overrides --model)
    #[arg(long)]
    pub model_b: Option<MixerModel>,

    /// Optional prefix filters to synchronize. If empty, syncs all messages.
    #[arg(short, long)]
    pub prefix: Vec<String>,
}

#[derive(Clone)]
struct SharedState {
    cache: Arc<Mutex<HashMap<String, Vec<OscArg>>>>,
    prefixes: Vec<String>,
    target_model: MixerModel,
}

impl SharedState {
    fn new(prefixes: Vec<String>, target_model: MixerModel) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            prefixes,
            target_model,
        }
    }

    fn should_sync(&self, path: &str, args: &[OscArg]) -> bool {
        if !self.prefixes.is_empty() && !self.prefixes.iter().any(|p| path.starts_with(p)) {
            return false;
        }

        let arg_str = args
            .first()
            .map(|a| match a {
                OscArg::Float(f) => f.to_string(),
                OscArg::Int(i) => i.to_string(),
                OscArg::String(s) => format!("\"{}\"", s),
                _ => "0.0".to_string(),
            })
            .unwrap_or_else(|| "0.0".to_string());

        parse_parameter(self.target_model, path, &arg_str).is_some()
            || parse_parameter(self.target_model, path, "0.0").is_some()
            || parse_parameter(self.target_model, path, "0").is_some()
            || parse_parameter(self.target_model, path, "ON").is_some()
            || parse_parameter(self.target_model, path, "OFF").is_some()
            || parse_parameter(self.target_model, path, "\"text\"").is_some()
    }

    async fn update_and_check(&self, path: &str, args: &[OscArg]) -> bool {
        let mut cache = self.cache.lock().await;
        if let Some(existing) = cache.get_mut(path) {
            if existing.as_slice() == args {
                return false; // Loop prevention / split-horizon: unchanged value
            }
            // ⚡ Bolt: Reuse the existing key string and vector allocation
            // instead of re-allocating path.to_string() and a new Vec on every update.
            existing.clear();
            existing.extend_from_slice(args);
            return true;
        }
        cache.insert(path.to_string(), args.to_vec());
        true
    }
}

use tokio::sync::broadcast::error::RecvError;

async fn run_proxy(
    local_client: Arc<MixerClient>,
    target_client: Arc<MixerClient>,
    state: SharedState,
) {
    let mut rx = local_client.subscribe();
    loop {
        match rx.recv().await {
            Ok(msg) => {
                if !state.should_sync(&msg.path, &msg.args) {
                    continue;
                }

                if state.update_and_check(&msg.path, &msg.args).await {
                    // Forward the exact message with all arguments
                    let _ = target_client.send_message(&msg.path, msg.args).await;
                }
            }
            Err(RecvError::Lagged(_)) => {
                continue; // Ignore lagged messages and wait for the next one
            }
            Err(RecvError::Closed) => {
                break; // Exit the loop if the channel is closed
            }
        }
    }
}

pub async fn run(args: Args) -> Result<()> {
    // Add default port 10023 if not specified
    let ip_a = if args.ip_a.contains(':') {
        args.ip_a.clone()
    } else {
        format!("{}:10023", args.ip_a)
    };
    let ip_b = if args.ip_b.contains(':') {
        args.ip_b.clone()
    } else {
        format!("{}:10023", args.ip_b)
    };

    let model_a = args.model_a.unwrap_or(args.model);
    let model_b = args.model_b.unwrap_or(args.model);

    let state_a_to_b = SharedState::new(args.prefix.clone(), model_b);
    let state_b_to_a = SharedState::new(args.prefix.clone(), model_a);

    println!(
        "Starting sync between {} ({}) and {} ({})",
        ip_a, model_a, ip_b, model_b
    );

    loop {
        let client_a = match MixerClient::connect(&ip_a, true).await {
            Ok(client) => Arc::new(client),
            Err(e) => {
                eprintln!("Failed to connect to Console A ({}): {}", ip_a, e);
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                continue;
            }
        };

        let client_b = match MixerClient::connect(&ip_b, true).await {
            Ok(client) => Arc::new(client),
            Err(e) => {
                eprintln!("Failed to connect to Console B ({}): {}", ip_b, e);
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                continue;
            }
        };

        println!("Connected to both consoles. Synchronizing...");

        // Spawn proxy A -> B (forwarding to Console B)
        let state_a = state_a_to_b.clone();
        let client_a_clone = client_a.clone();
        let client_b_clone = client_b.clone();
        let handle_a = tokio::spawn(async move {
            run_proxy(client_a_clone, client_b_clone, state_a).await;
        });

        // Spawn proxy B -> A (forwarding to Console A)
        let state_b = state_b_to_a.clone();
        let client_a_clone = client_a.clone();
        let client_b_clone = client_b.clone();
        let handle_b = tokio::spawn(async move {
            run_proxy(client_b_clone, client_a_clone, state_b).await;
        });

        // Wait for both proxies (they loop indefinitely).
        // If either fails or exits, we catch it and reconnect.
        tokio::select! {
            res = handle_a => {
                eprintln!("Proxy A -> B task stopped (result: {:?}). Attempting reconnection...", res);
            },
            res = handle_b => {
                eprintln!("Proxy B -> A task stopped (result: {:?}). Attempting reconnection...", res);
            },
        }

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_split_horizon() {
        let state = SharedState::new(vec![], MixerModel::X32);

        let path = "/ch/01/mix/fader";
        let args1 = vec![OscArg::Float(0.75)];
        let args2 = vec![OscArg::Float(0.80)];

        // Initial update from A should propagate
        assert_eq!(state.update_and_check(path, &args1).await, true);

        // Echo back from B should be blocked
        assert_eq!(state.update_and_check(path, &args1).await, false);

        // New update from B should propagate
        assert_eq!(state.update_and_check(path, &args2).await, true);

        // Echo back from A should be blocked
        assert_eq!(state.update_and_check(path, &args2).await, false);
    }

    #[tokio::test]
    async fn test_prefix_filter() {
        let state = SharedState::new(
            vec!["/ch/01/".to_string(), "/main/".to_string()],
            MixerModel::X32,
        );
        let dummy_arg = vec![OscArg::Float(0.0)];

        assert_eq!(state.should_sync("/ch/01/mix/fader", &dummy_arg), true);
        assert_eq!(state.should_sync("/main/st/mix/fader", &dummy_arg), true);
        assert_eq!(state.should_sync("/ch/02/mix/fader", &dummy_arg), false);
        assert_eq!(state.should_sync("/bus/01/mix/fader", &dummy_arg), false);

        let state_all = SharedState::new(vec![], MixerModel::X32);
        assert_eq!(state_all.should_sync("/ch/01/mix/fader", &dummy_arg), true);
        assert_eq!(state_all.should_sync("/bus/01/mix/fader", &dummy_arg), true);
    }

    #[test]
    fn test_model_specific_filtering() {
        let dummy_arg = vec![OscArg::Float(0.0)];

        let state_x32 = SharedState::new(vec![], MixerModel::X32);
        let state_wing = SharedState::new(vec![], MixerModel::Wing);
        let state_xr18 = SharedState::new(vec![], MixerModel::XR18);
        let state_xr16 = SharedState::new(vec![], MixerModel::XR16);
        let state_xr12 = SharedState::new(vec![], MixerModel::XR12);

        // Channel 32
        assert_eq!(state_x32.should_sync("/ch/32/mix/fader", &dummy_arg), true);
        assert_eq!(state_wing.should_sync("/ch/32/mix/fader", &dummy_arg), true);
        assert_eq!(
            state_xr18.should_sync("/ch/32/mix/fader", &dummy_arg),
            false
        );
        assert_eq!(
            state_xr16.should_sync("/ch/32/mix/fader", &dummy_arg),
            false
        );
        assert_eq!(
            state_xr12.should_sync("/ch/32/mix/fader", &dummy_arg),
            false
        );

        // Channel 40
        assert_eq!(state_x32.should_sync("/ch/40/mix/fader", &dummy_arg), false);
        assert_eq!(state_wing.should_sync("/ch/40/mix/fader", &dummy_arg), true);

        // Buses
        assert_eq!(
            state_xr18.should_sync("/bus/06/mix/fader", &dummy_arg),
            true
        );
        assert_eq!(
            state_xr16.should_sync("/bus/06/mix/fader", &dummy_arg),
            false
        );
        assert_eq!(
            state_xr12.should_sync("/bus/03/mix/fader", &dummy_arg),
            false
        );
    }

    #[test]
    fn test_args_model_parsing() {
        let parsed = Args::try_parse_from([
            "x32_sync",
            "--ip-a",
            "192.168.1.10",
            "--ip-b",
            "192.168.1.20",
            "-M",
            "xr18",
        ])
        .expect("Valid args");
        assert_eq!(parsed.model, MixerModel::XR18);
        assert!(parsed.model_a.is_none());
        assert!(parsed.model_b.is_none());

        let parsed_override = Args::try_parse_from([
            "x32_sync",
            "--ip-a",
            "192.168.1.10",
            "--ip-b",
            "192.168.1.20",
            "--model-a",
            "x32",
            "--model-b",
            "wing",
        ])
        .expect("Valid args");
        assert_eq!(parsed_override.model_a, Some(MixerModel::X32));
        assert_eq!(parsed_override.model_b, Some(MixerModel::Wing));
    }
}
