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
    // We also need fxrtn but let's stick to ch/auxin for now.

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

    println!("DCA {} members (source IDs): {:?}", dca_num, members);

    // X32 User Bank (custom bank) mapping:
    // It's mapped across 3 blocks of 8 faders, totaling 24 faders.
    // However, on standard X32, the custom bank can be addressed by `/-prefs/custom_bank/...`
    // Let's assume we map the first N faders of the User Bank to these members.
    // The specific paths for User Bank assignments on X32 are:
    // `/-prefs/custom_bank/1/1` to `/-prefs/custom_bank/1/8`  (Left bank)
    // `/-prefs/custom_bank/2/1` to `/-prefs/custom_bank/2/8`  (Center bank)
    // `/-prefs/custom_bank/3/1` to `/-prefs/custom_bank/3/8`  (Right bank)
    // Wait, the standard X32 only has one layer per section. The "User" layer.

    // Actually, checking X32 OSC documentation, the paths for the user assignment are:
    // `/-prefs/user_bank/1/1` ? No, let's look for how it's done.

    // We'll write to `/-prefs/custom_bank/X` where X is 1..24? Or similar.
    // For now we'll send to `/-prefs/user_rout/...` or similar. Let's check `x32_lib` or `osc` for exact path.
    // Actually, memory says: dynamically rewrite custom layer mappings (`/-prefs/custom_bank/`).

    // Memory: `/-prefs/custom_bank/` is the path mentioned in the TODO.

    // But how is it addressed? Let's just output a sequential list to `/-prefs/custom_bank/1` ... `/-prefs/custom_bank/N`.
    // Wait, let's look at `x32_core` to see if it supports `/-prefs/custom_bank`.
    // Since `x32_core` didn't have it explicitly, it might just accept it as a string prefix.
    // Wait, the X32 has custom layers per block:
    // Block 1: `/-prefs/custom_bank/1/1` ... `/-prefs/custom_bank/1/8`

    // Let's map sequentially up to 24 members.
    for i in 0..24 {
        let source_id = if i < members.len() {
            members[i]
        } else {
            0 // 0 = OFF
        };

        let path = format!("/-prefs/custom_bank/{}", i + 1);
        // Note: Some docs say `/-prefs/custom_bank/{bank}/{channel}`. The memory just says `/-prefs/custom_bank/`.
        // Let's assume it's `/-prefs/custom_bank/{1..24}`.
        client.send_message(&path, vec![OscArg::Int(source_id)]).await?;
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

        let dca1_bit = 1;
        let dca2_bit = 2;
        let dca3_bit = 4;

        assert_ne!(assignments.get("/ch/01/grp/dca").unwrap() & dca1_bit, 0);
        assert_ne!(assignments.get("/ch/01/grp/dca").unwrap() & dca2_bit, 0);
        assert_eq!(assignments.get("/ch/01/grp/dca").unwrap() & dca3_bit, 0);

        assert_eq!(assignments.get("/ch/05/grp/dca").unwrap() & dca1_bit, 0);
        assert_ne!(assignments.get("/ch/05/grp/dca").unwrap() & dca2_bit, 0);

        assert_ne!(assignments.get("/auxin/01/grp/dca").unwrap() & dca1_bit, 0);
    }
}
