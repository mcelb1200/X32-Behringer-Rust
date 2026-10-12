use anyhow::{Context, Result, bail};
use clap::Parser;
use osc_lib::OscArg;
use std::collections::HashMap;
use tokio::sync::broadcast;
use x32_lib::{MixerClient, MixerModel};

#[derive(Parser, Debug, Clone)]
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

    /// Mixer model: X32, Wing, XR18, XR16, XR12
    #[arg(short, long, default_value = "X32")]
    pub model: MixerModel,

    /// DCA button to monitor (1-8 for X32/XAir, 1-16 for Wing). Defaults to all if not specified (will spill any pressed DCA).
    #[arg(short, long)]
    pub dca: Option<u8>,
}

pub fn max_channels_for_model(model: MixerModel) -> u8 {
    match model {
        MixerModel::X32 => 32,
        MixerModel::Wing => 40,
        MixerModel::XR18 | MixerModel::XR16 => 16,
        MixerModel::XR12 => 12,
    }
}

pub fn max_auxins_for_model(model: MixerModel) -> u8 {
    match model {
        MixerModel::X32 | MixerModel::Wing => 8,
        MixerModel::XR18 | MixerModel::XR16 => 6,
        MixerModel::XR12 => 4,
    }
}

pub fn max_fxrtns_for_model(model: MixerModel) -> u8 {
    match model {
        MixerModel::X32 | MixerModel::Wing => 8,
        MixerModel::XR18 | MixerModel::XR16 | MixerModel::XR12 => 4,
    }
}

pub fn max_dcas_for_model(model: MixerModel) -> u8 {
    match model {
        MixerModel::X32 => 8,
        MixerModel::Wing => 16,
        MixerModel::XR18 | MixerModel::XR16 | MixerModel::XR12 => 4,
    }
}

pub fn dca_select_index_range(model: MixerModel) -> (i32, i32) {
    match model {
        MixerModel::X32 => (72, 79),
        MixerModel::Wing => (72, 87),
        MixerModel::XR18 | MixerModel::XR16 | MixerModel::XR12 => (72, 75),
    }
}

pub fn get_dca_from_selidx(model: MixerModel, idx: i32) -> Option<u8> {
    let (start, end) = dca_select_index_range(model);
    if idx >= start && idx <= end {
        Some((idx - start + 1) as u8)
    } else {
        None
    }
}

pub fn get_custom_bank_path(_model: MixerModel, i: usize) -> String {
    let block = (i / 8) + 1;
    let fader = (i % 8) + 1;
    format!("/-prefs/custom_bank/{}/{}", block, fader)
}

pub async fn run(args: Args) -> Result<()> {
    let max_dca = max_dcas_for_model(args.model);
    if let Some(target) = args.dca {
        if target < 1 || target > max_dca {
            bail!(
                "Invalid DCA number {} for model {:?}. Allowed range is 1-{}",
                target,
                args.model,
                max_dca
            );
        }
    }

    println!("Connecting to {:?} at {} for DCA Spills...", args.model, args.ip);

    // Pass true to enable heartbeat / xremote background task
    let mut client = MixerClient::connect(&args.ip, true)
        .await
        .context("Failed to connect to mixer")?;

    let mut rx = client.subscribe();

    // We maintain a cache of DCA assignments
    let mut dca_assignments: HashMap<String, u8> = HashMap::new();

    let max_ch = max_channels_for_model(args.model);
    let max_aux = max_auxins_for_model(args.model);
    let max_fx = max_fxrtns_for_model(args.model);

    // Fetch initial DCA assignments
    println!("Fetching initial DCA assignments for {:?}...", args.model);
    for i in 1..=max_ch {
        let path = format!("/ch/{:02}/grp/dca", i);
        client.send_message(&path, vec![]).await?;
    }
    for i in 1..=max_aux {
        let path = format!("/auxin/{:02}/grp/dca", i);
        client.send_message(&path, vec![]).await?;
    }
    for i in 1..=max_fx {
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
                        if let Some(dca_num) = get_dca_from_selidx(args.model, *idx) {
                            if let Some(target_dca) = args.dca {
                                if dca_num != target_dca {
                                    continue;
                                }
                            }

                            println!("DCA {} selected! Spilling members...", dca_num);
                            spill_dca(&mut client, args.model, dca_num, &dca_assignments).await?;
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

pub async fn spill_dca(
    client: &mut MixerClient,
    model: MixerModel,
    dca_num: u8,
    assignments: &HashMap<String, u8>,
) -> Result<()> {
    let dca_bit = 1u16 << (dca_num - 1);
    let mut members = Vec::new();

    let max_ch = max_channels_for_model(model);
    let max_aux = max_auxins_for_model(model);
    let max_fx = max_fxrtns_for_model(model);

    // Determine members
    for i in 1..=max_ch {
        let path = format!("/ch/{:02}/grp/dca", i);
        if let Some(mask) = assignments.get(&path) {
            if (*mask as u16 & dca_bit) != 0 {
                members.push(i as i32);
            }
        }
    }

    for i in 1..=max_aux {
        let path = format!("/auxin/{:02}/grp/dca", i);
        if let Some(mask) = assignments.get(&path) {
            if (*mask as u16 & dca_bit) != 0 {
                members.push((i + max_ch) as i32);
            }
        }
    }

    for i in 1..=max_fx {
        let path = format!("/fxrtn/{:02}/grp/dca", i);
        if let Some(mask) = assignments.get(&path) {
            if (*mask as u16 & dca_bit) != 0 {
                members.push((i + max_ch + max_aux) as i32);
            }
        }
    }

    println!("DCA {} members (source IDs): {:?}", dca_num, members);

    // Map sequentially up to 24 members across custom bank faders
    for i in 0..24 {
        let source_id = if i < members.len() {
            members[i]
        } else {
            0 // 0 = OFF
        };

        let path = get_custom_bank_path(model, i);
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

    #[test]
    fn test_model_channel_limits() {
        assert_eq!(max_channels_for_model(MixerModel::X32), 32);
        assert_eq!(max_channels_for_model(MixerModel::Wing), 40);
        assert_eq!(max_channels_for_model(MixerModel::XR18), 16);
        assert_eq!(max_channels_for_model(MixerModel::XR16), 16);
        assert_eq!(max_channels_for_model(MixerModel::XR12), 12);
    }

    #[test]
    fn test_model_aux_and_fx_limits() {
        assert_eq!(max_auxins_for_model(MixerModel::X32), 8);
        assert_eq!(max_auxins_for_model(MixerModel::Wing), 8);
        assert_eq!(max_auxins_for_model(MixerModel::XR18), 6);
        assert_eq!(max_auxins_for_model(MixerModel::XR16), 6);
        assert_eq!(max_auxins_for_model(MixerModel::XR12), 4);

        assert_eq!(max_fxrtns_for_model(MixerModel::X32), 8);
        assert_eq!(max_fxrtns_for_model(MixerModel::Wing), 8);
        assert_eq!(max_fxrtns_for_model(MixerModel::XR18), 4);
        assert_eq!(max_fxrtns_for_model(MixerModel::XR16), 4);
        assert_eq!(max_fxrtns_for_model(MixerModel::XR12), 4);
    }

    #[test]
    fn test_model_dca_limits() {
        assert_eq!(max_dcas_for_model(MixerModel::X32), 8);
        assert_eq!(max_dcas_for_model(MixerModel::Wing), 16);
        assert_eq!(max_dcas_for_model(MixerModel::XR18), 4);
        assert_eq!(max_dcas_for_model(MixerModel::XR16), 4);
        assert_eq!(max_dcas_for_model(MixerModel::XR12), 4);
    }

    #[test]
    fn test_dca_select_index_mapping() {
        assert_eq!(get_dca_from_selidx(MixerModel::X32, 72), Some(1));
        assert_eq!(get_dca_from_selidx(MixerModel::X32, 79), Some(8));
        assert_eq!(get_dca_from_selidx(MixerModel::X32, 80), None);

        assert_eq!(get_dca_from_selidx(MixerModel::Wing, 72), Some(1));
        assert_eq!(get_dca_from_selidx(MixerModel::Wing, 87), Some(16));
        assert_eq!(get_dca_from_selidx(MixerModel::Wing, 88), None);

        assert_eq!(get_dca_from_selidx(MixerModel::XR18, 72), Some(1));
        assert_eq!(get_dca_from_selidx(MixerModel::XR18, 75), Some(4));
        assert_eq!(get_dca_from_selidx(MixerModel::XR18, 76), None);
    }

    #[test]
    fn test_custom_bank_path_formatting() {
        assert_eq!(get_custom_bank_path(MixerModel::X32, 0), "/-prefs/custom_bank/1/1");
        assert_eq!(get_custom_bank_path(MixerModel::X32, 7), "/-prefs/custom_bank/1/8");
        assert_eq!(get_custom_bank_path(MixerModel::Wing, 8), "/-prefs/custom_bank/2/1");
        assert_eq!(get_custom_bank_path(MixerModel::XR18, 23), "/-prefs/custom_bank/3/8");
    }
}
