use anyhow::Result;
use clap::Parser;
use osc_lib::{OscArg, OscMessage};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

#[derive(Parser, Debug)]
#[command(author, version, about = "Bidirectional synchronization tool for two X32/M32 consoles", long_about = None)]
struct Args {
    /// IP address of Console A
    #[arg(short = 'a', long)]
    ip_a: String,

    /// IP address of Console B
    #[arg(short = 'b', long)]
    ip_b: String,

    /// Optional prefix filters to synchronize. If empty, syncs all messages.
    #[arg(short, long)]
    prefix: Vec<String>,
}

#[derive(Clone)]
struct SharedState {
    cache: Arc<Mutex<HashMap<String, Vec<OscArg>>>>,
    prefixes: Vec<String>,
}

impl SharedState {
    fn new(prefixes: Vec<String>) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            prefixes,
        }
    }

    fn should_sync(&self, path: &str) -> bool {
        if self.prefixes.is_empty() {
            return true;
        }
        self.prefixes.iter().any(|p| path.starts_with(p))
    }

    async fn update_and_check(&self, path: &str, args: &[OscArg]) -> bool {
        let mut cache = self.cache.lock().await;
        if let Some(existing) = cache.get(path) {
            if existing.as_slice() == args {
                return false; // Loop prevention / split-horizon: unchanged value
            }
        }
        cache.insert(path.to_string(), args.to_vec());
        true
    }
}

async fn handle_message(
    buf: &[u8],
    len: usize,
    state: &SharedState,
    target_sock: &UdpSocket,
    target_addr: &str,
) -> Result<()> {
    if let Ok(msg) = OscMessage::from_bytes(&buf[..len]) {
        if !state.should_sync(&msg.path) {
            return Ok(());
        }

        if state.update_and_check(&msg.path, &msg.args).await {
            // Forward the exact message with all arguments
            if let Ok(bytes) = msg.to_bytes() {
                let _ = target_sock.send_to(&bytes, target_addr).await;
            }
        }
    }
    Ok(())
}

async fn run_proxy(
    local_sock: Arc<UdpSocket>,
    target_sock: Arc<UdpSocket>,
    target_addr: String,
    state: SharedState,
) {
    let mut buf = [0u8; 1024];
    loop {
        if let Ok((len, _addr)) = local_sock.recv_from(&mut buf).await {
            let _ = handle_message(&buf, len, &state, &target_sock, &target_addr).await;
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

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

    let state = SharedState::new(args.prefix.clone());

    let sock_a = Arc::new(UdpSocket::bind("0.0.0.0:0").await?);
    let sock_b = Arc::new(UdpSocket::bind("0.0.0.0:0").await?);

    println!("Starting sync between {} and {}", ip_a, ip_b);

    // Spawn proxy A -> B
    let state_a = state.clone();
    let sock_a_clone = sock_a.clone();
    let sock_b_clone = sock_b.clone();
    let ip_b_clone = ip_b.clone();
    tokio::spawn(async move {
        run_proxy(sock_a_clone, sock_b_clone, ip_b_clone, state_a).await;
    });

    // Spawn proxy B -> A
    let state_b = state.clone();
    let sock_a_clone = sock_a.clone();
    let sock_b_clone = sock_b.clone();
    let ip_a_clone = ip_a.clone();
    tokio::spawn(async move {
        run_proxy(sock_b_clone, sock_a_clone, ip_a_clone, state_b).await;
    });

    // Keep-alive loop
    loop {
        let xremote_msg = OscMessage::new("/xremote".to_string(), vec![]);
        if let Ok(bytes) = xremote_msg.to_bytes() {
            let _ = sock_a.send_to(&bytes, &ip_a).await;
            let _ = sock_b.send_to(&bytes, &ip_b).await;
        }
        sleep(Duration::from_secs(9)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_split_horizon() {
        let state = SharedState::new(vec![]);

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
        let state = SharedState::new(vec!["/ch/01/".to_string(), "/main/".to_string()]);

        assert_eq!(state.should_sync("/ch/01/mix/fader"), true);
        assert_eq!(state.should_sync("/main/st/mix/fader"), true);
        assert_eq!(state.should_sync("/ch/02/mix/fader"), false);
        assert_eq!(state.should_sync("/bus/01/mix/fader"), false);

        let state_all = SharedState::new(vec![]);
        assert_eq!(state_all.should_sync("/ch/01/mix/fader"), true);
        assert_eq!(state_all.should_sync("/bus/01/mix/fader"), true);
    }
}
