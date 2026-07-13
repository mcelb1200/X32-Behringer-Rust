use anyhow::Result;
use clap::Parser;
use tokio::time::{interval, Duration};
use x32_lib::MixerClient;
use osc_lib::OscArg;

#[derive(Parser, Debug, Clone)]
#[command(
    author,
    version,
    about = "Automated System Tuning (Oscillator-Assisted)",
    long_about = "Guided oscillator testing using the X32's built-in signal generator."
)]
pub struct Args {
    /// IP address of the X32 console
    #[arg(short, long)]
    pub ip: String,

    /// Target destination index for the oscillator
    #[arg(short, long, default_value_t = 0)]
    pub dest: i32,

    /// Max level as a float 0.0 to 1.0 (to restrict max dBFS conservatively)
    #[arg(short, long, default_value_t = 0.5)]
    pub max_level: f32,

    /// Type of signal: 0 = Sine, 1 = Pink Noise, 2 = White Noise
    #[arg(short, long, default_value_t = 1)]
    pub signal_type: i32,

    /// Interval in milliseconds for ramping volume
    #[arg(short = 'r', long, default_value_t = 250)]
    pub ramp_interval_ms: u64,
}

pub async fn run(args: Args) -> Result<()> {
    println!("Connecting to X32 at {}", args.ip);
    let client = MixerClient::connect(&args.ip, true).await?;

    // 1. Initial configuration
    println!("Setting oscillator destination to {} and type to {}", args.dest, args.signal_type);
    let _ = client.send_message("/config/osc/dest", vec![OscArg::Int(args.dest)]).await;
    let _ = client.send_message("/config/osc/type", vec![OscArg::Int(args.signal_type)]).await;
    let _ = client.send_message("/config/osc/level", vec![OscArg::Float(0.0)]).await;

    // 2. Start oscillator
    println!("Turning oscillator ON.");
    let _ = client.send_message("/config/osc", vec![OscArg::Int(1)]).await;

    // 3. Ramp up the volume
    let steps = 20;
    let max_level = args.max_level.clamp(0.0, 1.0);
    let mut current_level = 0.0;
    let step_val = max_level / (steps as f32);

    let mut ticker = interval(Duration::from_millis(args.ramp_interval_ms));
    println!("Ramping up to max level {:.2}", max_level);

    let timeout_duration = tokio::time::sleep(Duration::from_secs(30));
    tokio::pin!(timeout_duration);

    let mut is_active = true;

    while is_active {
        tokio::select! {
            _ = ticker.tick() => {
                if current_level < max_level {
                    current_level += step_val;
                    if current_level > max_level {
                        current_level = max_level;
                    }
                    let _ = client.send_message("/config/osc/level", vec![OscArg::Float(current_level)]).await;
                }
            }
            _ = tokio::signal::ctrl_c() => {
                println!("Received Ctrl-C, gracefully shutting down...");
                is_active = false;
            }
            _ = &mut timeout_duration => {
                println!("Auto-Timeout reached (30s). Shutting down.");
                is_active = false;
            }
        }
    }

    // 4. Cleanup
    println!("Turning oscillator OFF and resetting level.");
    let _ = client.send_message("/config/osc/level", vec![OscArg::Float(0.0)]).await;
    let _ = client.send_message("/config/osc", vec![OscArg::Int(0)]).await;

    // Slight delay to ensure messages are sent
    tokio::time::sleep(Duration::from_millis(100)).await;
    println!("Done.");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::net::UdpSocket;
    use x32_core::Mixer;

    #[tokio::test]
    async fn test_x32_system_tune() {
        let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let port = socket.local_addr().unwrap().port();
        let addr = format!("127.0.0.1:{}", port);

        let mut mixer = Mixer::new();
        let socket_arc = Arc::new(socket);
        let socket_rx = socket_arc.clone();

        let _ = tokio::spawn(async move {
            let mut buf = [0u8; 1024];
            while let Ok((len, src)) = socket_rx.recv_from(&mut buf).await {
                let responses_opt = mixer.dispatch(&buf[..len], src).ok();
                if let Some(responses) = responses_opt {
                    for (addr, response_bytes) in responses {
                        let _ = socket_rx.send_to(&response_bytes, addr).await;
                    }
                }
            }
        });

        let args = Args {
            ip: addr,
            dest: 2,
            max_level: 0.2,
            signal_type: 0,
            ramp_interval_ms: 10,
        };

        // Run the app in a background task
        let handle = tokio::spawn(async move {
            let _ = run(args).await;
        });

        // Let it run for a bit
        tokio::time::sleep(Duration::from_millis(150)).await;

        handle.abort();
    }
}
