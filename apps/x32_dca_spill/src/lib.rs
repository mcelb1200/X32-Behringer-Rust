use anyhow::{Context, Result};
use clap::Parser;
use osc_lib::OscArg;
use std::collections::HashMap;
use tokio::sync::broadcast;
use x32_lib::MixerClient;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Automatically spill DCA members onto a custom fader bank for quick access",
    long_about = None
)]
pub struct Args {
    /// IP address of the X32 console
    #[arg(short, long)]
    pub ip: String,

    /// DCA button to monitor (1-8). Defaults to all if not specified (will spill any pressed DCA).
    #[arg(short, long)]
    pub dca: Option<u8>,
}

pub async fn run(args: Args) -> Result<()> {
    println!("Connecting to X32 at {} for DCA Spills...", args.ip);

    // Pass true to enable heartbeat / xremote background task
    let mut client = MixerClient::connect(&args.ip, true)
        .await
        .context("Failed to connect to X32")?;

    let mut rx = client.subscribe();

    // To properly map DCA spills, we need:
    // 1. DCA membership data: For every channel (1-32) and auxin (1-8), which DCAs are they assigned to?
    //    Stored at `/ch/XX/grp/dca` (bitmask). DCA 1 = bit 0 (value 1), DCA 2 = bit 1 (value 2), ... DCA 8 = bit 7 (value 128).

    // We maintain a cache of DCA assignments
    let mut dca_assignments: HashMap<String, u8> = HashMap::new();

    // Fetch initial DCA assignments
    println!("Fetching initial DCA assignments...");
    for i in 1..=32 {
        let path = format!("/ch/{:02}/grp/dca", i);
        client.send_message(&path, vec![]).await?;
    }
    for i in 1..=8 {
        let path = format!("/auxin/{:02}/grp/dca", i);
        client.send_message(&path, vec![]).await?;
    }
    for i in 1..=8 {
        let path = format!("/fxrtn/{:02}/grp/dca", i);
        client.send_message(&path, vec![]).await?;
    }

    println!("Listening for DCA selects...");

    loop {
        match rx.recv().await {
            Ok(msg) => {
                if msg.path.ends_with("/grp/dca") {
                    if let Some(OscArg::Int(val)) = msg.args.first() {
                        dca_assignments.insert(msg.path.clone(), *val as u8);
                    }
                } else if msg.path.starts_with("/-stat/selidx") {
                    if let Some(OscArg::Int(idx)) = msg.args.first() {
                        // idx mapping:
                        // 0-31: Ch 1-32
                        // 32-39: Aux 1-8
                        // 40-47: FxRtn 1-8
                        // 48-63: Bus 1-16
                        // 64-69: Matrix 1-6
                        // 70: Main C
                        // 71: Main LR
                        // 72-79: DCA 1-8

                        let idx = *idx;
                        if (72..=79).contains(&idx) {
                            let dca_num = (idx - 72) as u8 + 1;

                            if let Some(target_dca) = args.dca {
                                if dca_num != target_dca {
                                    continue;
                                }
                            }

                            println!("DCA {} selected! Spilling members...", dca_num);
                            spill_dca(&mut client, dca_num, &dca_assignments).await?;
                        }
                    }
                }
            }
            Err(broadcast::error::RecvError::Lagged(_)) => continue,
            Err(broadcast::error::RecvError::Closed) => break,
        }
    }

    Ok(())
}

async fn spill_dca(
    client: &mut MixerClient,
    dca_num: u8,
    assignments: &HashMap<String, u8>,
) -> Result<()> {
    let dca_bit = 1 << (dca_num - 1);
    let mut members = Vec::new();

    // Determine members
    for i in 1..=32 {
        let path = format!("/ch/{:02}/grp/dca", i);
        if let Some(mask) = assignments.get(&path) {
            if (mask & dca_bit) != 0 {
                // Member found. The source ID for Ch 1 is 1, Ch 2 is 2...
                members.push(i);
            }
        }
    }

    for i in 1..=8 {
        let path = format!("/auxin/{:02}/grp/dca", i);
        if let Some(mask) = assignments.get(&path) {
            if (mask & dca_bit) != 0 {
                // The source ID for Aux 1 is 33
                members.push(i + 32);
            }
        }
    }

    for i in 1..=8 {
        let path = format!("/fxrtn/{:02}/grp/dca", i);
        if let Some(mask) = assignments.get(&path) {
            if (mask & dca_bit) != 0 {
                // The source ID for FxRtn 1 is 41
                members.push(i + 40);
            }
        }
    }

    println!("DCA {} members (source IDs): {:?}", dca_num, members);

    // X32 User Bank (custom bank) mapping:
    // It's mapped across 3 blocks of 8 faders, totaling 24 faders.
    // Map sequentially up to 24 members across 3 blocks of 8 faders using `/-prefs/custom_bank/{block}/{fader}`
    for i in 0..24 {
        let source_id = if i < members.len() {
            members[i]
        } else {
            0 // 0 = OFF
        };

        let block = (i / 8) + 1; // 1 to 3
        let fader = (i % 8) + 1; // 1 to 8

        let path = format!("/-prefs/custom_bank/{}/{}", block, fader);
        client
            .send_message(&path, vec![OscArg::Int(source_id)])
            .await?;
    }

    println!("Spill complete.");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dca_bitmask() {
        let mut assignments = HashMap::new();
        // Channel 1 is in DCA 1 and 2
        assignments.insert("/ch/01/grp/dca".to_string(), 3);
        // Channel 5 is in DCA 2
        assignments.insert("/ch/05/grp/dca".to_string(), 2);
        // Auxin 1 is in DCA 1
        assignments.insert("/auxin/01/grp/dca".to_string(), 1);
        // Fxrtn 2 is in DCA 3
        assignments.insert("/fxrtn/02/grp/dca".to_string(), 4);

        let dca1_bit = 1;
        let dca2_bit = 2;
        let dca3_bit = 4;

        assert_ne!(assignments.get("/ch/01/grp/dca").unwrap() & dca1_bit, 0);
        assert_ne!(assignments.get("/ch/01/grp/dca").unwrap() & dca2_bit, 0);
        assert_eq!(assignments.get("/ch/01/grp/dca").unwrap() & dca3_bit, 0);

        assert_eq!(assignments.get("/ch/05/grp/dca").unwrap() & dca1_bit, 0);
        assert_ne!(assignments.get("/ch/05/grp/dca").unwrap() & dca2_bit, 0);

        assert_ne!(assignments.get("/auxin/01/grp/dca").unwrap() & dca1_bit, 0);

        assert_eq!(assignments.get("/fxrtn/02/grp/dca").unwrap() & dca1_bit, 0);
        assert_ne!(assignments.get("/fxrtn/02/grp/dca").unwrap() & dca3_bit, 0);
    }
}
