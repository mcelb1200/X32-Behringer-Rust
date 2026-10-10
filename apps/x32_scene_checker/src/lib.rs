use clap::Parser;
use osc_lib::OscArg;
use std::collections::HashMap;
use std::io::{BufRead, Read, Write};
use std::time::Duration;
use x32_lib::MixerClient;
use x32_lib::MixerModel;
use x32_lib::scene_parse::SceneParser;

#[derive(Parser, Debug)]
#[command(author, version, about = "Intelligent Scene Pre-flight Checker", long_about = None)]
pub struct Args {
    #[arg(short, long, default_value = "192.168.0.64")]
    pub ip: String,

    #[arg(short, long)]
    pub scene: String,

    #[arg(
        short,
        long,
        default_value = "X32",
        help = "Mixer model: X32, Wing, XR18, XR16, XR12"
    )]
    pub model: MixerModel,

    #[arg(long)]
    pub auto_load: bool,

    #[arg(
        long,
        help = "Comma-separated list of OSC paths or prefixes to lock (e.g. /routing,/main/st/mix/on)"
    )]
    pub locked_paths: Option<String>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum RiskLevel {
    Info,
    Low,
    Moderate,
    High,
    Critical,
}

impl RiskLevel {
    fn name(&self) -> &'static str {
        match self {
            RiskLevel::Info => "⚪ INFO",
            RiskLevel::Low => "🟢 LOW",
            RiskLevel::Moderate => "🟡 MODERATE",
            RiskLevel::High => "🟠 HIGH",
            RiskLevel::Critical => "🔴 CRITICAL",
        }
    }
}

#[derive(Debug, Clone)]
pub struct RiskIssue {
    pub level: RiskLevel,
    pub path: String,
    pub description: String,
    pub from: OscArg,
    pub to: OscArg,
}

fn format_arg(arg: &OscArg) -> String {
    match arg {
        OscArg::Int(i) => i.to_string(),
        OscArg::Float(f) => format!("{:.3}", f),
        OscArg::String(s) => s.clone(),
        OscArg::Blob(_) => "[Blob]".to_string(),
    }
}

pub fn classify_risk(path: &str, current: &OscArg, scene: &OscArg) -> Option<RiskIssue> {
    classify_risk_with_model(MixerModel::X32, path, current, scene)
}

struct ModelLimits {
    channels: usize,
    buses: usize,
    auxins: usize,
    fxrtns: usize,
    matrices: usize,
    dcas: usize,
    fx_slots: usize,
    channel_eq_bands: usize,
    auxin_eq_bands: usize,
    fxrtn_eq_bands: usize,
    bus_eq_bands: usize,
    mtx_eq_bands: usize,
    main_eq_bands: usize,
}

fn get_model_limits(model: MixerModel) -> ModelLimits {
    match model {
        MixerModel::X32 => ModelLimits {
            channels: 32,
            buses: 16,
            auxins: 8,
            fxrtns: 8,
            matrices: 6,
            dcas: 8,
            fx_slots: 8,
            channel_eq_bands: 4,
            auxin_eq_bands: 2,
            fxrtn_eq_bands: 2,
            bus_eq_bands: 6,
            mtx_eq_bands: 6,
            main_eq_bands: 6,
        },
        MixerModel::Wing => ModelLimits {
            channels: 40,
            buses: 28,
            auxins: 16,
            fxrtns: 16,
            matrices: 8,
            dcas: 16,
            fx_slots: 16,
            channel_eq_bands: 8,
            auxin_eq_bands: 4,
            fxrtn_eq_bands: 4,
            bus_eq_bands: 8,
            mtx_eq_bands: 8,
            main_eq_bands: 8,
        },
        MixerModel::XR18 => ModelLimits {
            channels: 16,
            buses: 6,
            auxins: 2,
            fxrtns: 4,
            matrices: 0,
            dcas: 4,
            fx_slots: 4,
            channel_eq_bands: 4,
            auxin_eq_bands: 4,
            fxrtn_eq_bands: 4,
            bus_eq_bands: 6,
            mtx_eq_bands: 0,
            main_eq_bands: 6,
        },
        MixerModel::XR16 => ModelLimits {
            channels: 16,
            buses: 4,
            auxins: 2,
            fxrtns: 4,
            matrices: 0,
            dcas: 4,
            fx_slots: 4,
            channel_eq_bands: 4,
            auxin_eq_bands: 4,
            fxrtn_eq_bands: 4,
            bus_eq_bands: 6,
            mtx_eq_bands: 0,
            main_eq_bands: 6,
        },
        MixerModel::XR12 => ModelLimits {
            channels: 12,
            buses: 2,
            auxins: 2,
            fxrtns: 4,
            matrices: 0,
            dcas: 4,
            fx_slots: 4,
            channel_eq_bands: 4,
            auxin_eq_bands: 4,
            fxrtn_eq_bands: 4,
            bus_eq_bands: 6,
            mtx_eq_bands: 0,
            main_eq_bands: 6,
        },
    }
}

fn check_model_bounds(model: MixerModel, path: &str) -> Option<String> {
    let limits = get_model_limits(model);
    let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    if parts.is_empty() {
        return None;
    }

    match parts[0] {
        "ch" if parts.len() >= 2 => {
            if let Ok(idx) = parts[1].parse::<usize>() {
                if idx == 0 || idx > limits.channels {
                    return Some(format!(
                        "Channel index {} is out of bounds for model {} (max {})",
                        idx, model, limits.channels
                    ));
                }
            }
        }
        "bus" if parts.len() >= 2 => {
            if let Ok(idx) = parts[1].parse::<usize>() {
                if idx == 0 || idx > limits.buses {
                    return Some(format!(
                        "Bus index {} is out of bounds for model {} (max {})",
                        idx, model, limits.buses
                    ));
                }
            }
        }
        "auxin" if parts.len() >= 2 => {
            if let Ok(idx) = parts[1].parse::<usize>() {
                if idx == 0 || idx > limits.auxins {
                    return Some(format!(
                        "Auxin index {} is out of bounds for model {} (max {})",
                        idx, model, limits.auxins
                    ));
                }
            }
        }
        "fxrtn" if parts.len() >= 2 => {
            if let Ok(idx) = parts[1].parse::<usize>() {
                if idx == 0 || idx > limits.fxrtns {
                    return Some(format!(
                        "FX return index {} is out of bounds for model {} (max {})",
                        idx, model, limits.fxrtns
                    ));
                }
            }
        }
        "mtx" if parts.len() >= 2 => {
            if limits.matrices == 0 {
                return Some(format!("Matrix outputs are unsupported on model {}", model));
            }
            if let Ok(idx) = parts[1].parse::<usize>() {
                if idx == 0 || idx > limits.matrices {
                    return Some(format!(
                        "Matrix index {} is out of bounds for model {} (max {})",
                        idx, model, limits.matrices
                    ));
                }
            }
        }
        "dca" if parts.len() >= 2 => {
            if let Ok(idx) = parts[1].parse::<usize>() {
                if idx == 0 || idx > limits.dcas {
                    return Some(format!(
                        "DCA index {} is out of bounds for model {} (max {})",
                        idx, model, limits.dcas
                    ));
                }
            }
        }
        "fx" if parts.len() >= 2 => {
            if let Ok(idx) = parts[1].parse::<usize>() {
                if idx == 0 || idx > limits.fx_slots {
                    return Some(format!(
                        "FX slot index {} is out of bounds for model {} (max {})",
                        idx, model, limits.fx_slots
                    ));
                }
            }
        }
        "main" if parts.len() >= 2 && parts[1] == "m" => {
            if matches!(
                model,
                MixerModel::XR18 | MixerModel::XR16 | MixerModel::XR12
            ) {
                return Some(format!("Mono main bus is unsupported on model {}", model));
            }
        }
        _ => {}
    }

    // Check EQ band limits if /eq/ is present in path
    if let Some(eq_idx) = parts.iter().position(|&p| p == "eq") {
        if eq_idx + 1 < parts.len() {
            if let Ok(band) = parts[eq_idx + 1].parse::<usize>() {
                let max_bands = match parts[0] {
                    "ch" => limits.channel_eq_bands,
                    "auxin" => limits.auxin_eq_bands,
                    "fxrtn" => limits.fxrtn_eq_bands,
                    "bus" => limits.bus_eq_bands,
                    "mtx" => limits.mtx_eq_bands,
                    "main" => limits.main_eq_bands,
                    _ => 6,
                };
                if band == 0 || band > max_bands {
                    return Some(format!(
                        "EQ band {} exceeds max bands ({}) for {} on model {}",
                        band, max_bands, parts[0], model
                    ));
                }
            }
        }
    }

    None
}

pub fn classify_risk_with_model(
    model: MixerModel,
    path: &str,
    current: &OscArg,
    scene: &OscArg,
) -> Option<RiskIssue> {
    if current == scene {
        match (current, scene) {
            (OscArg::Float(f1), OscArg::Float(f2)) => {
                if (f1 - f2).abs() < f32::EPSILON {
                    return None;
                }
            }
            _ => return None,
        }
    }

    // First check model-specific bounds & capability violations
    if let Some(out_of_bounds_reason) = check_model_bounds(model, path) {
        return Some(RiskIssue {
            level: RiskLevel::Critical,
            path: path.to_string(),
            description: out_of_bounds_reason,
            from: current.clone(),
            to: scene.clone(),
        });
    }

    let mut level = RiskLevel::Low;
    let mut description = format!(
        "Change from {} to {}",
        format_arg(current),
        format_arg(scene)
    );

    if path.starts_with("/routing/") || path.ends_with("/config/source") {
        level = RiskLevel::Critical;
        description = format!(
            "Routing change! From {} to {}",
            format_arg(current),
            format_arg(scene)
        );
    } else if path.starts_with("/main/st/mix/on")
        || path.starts_with("/main/m/mix/on")
        || (path.contains("/mix/on") && (path.starts_with("/bus/") || path.starts_with("/mtx/")))
    {
        level = RiskLevel::Critical;
        description = format!(
            "Output mute state change: {} -> {}",
            format_arg(current),
            format_arg(scene)
        );
    } else if path.ends_with("/preamp/trim") || path.contains("/headamp/") {
        if let (OscArg::Float(c), OscArg::Float(s)) = (current, scene) {
            let diff = (c - s).abs();
            if diff > 0.166 {
                level = RiskLevel::High;
                description = format!("Large gain jump (>{:.2} change): {} -> {}", diff, c, s);
            } else {
                level = RiskLevel::Moderate;
            }
        }
    } else if path.contains("/eq/") {
        if path.ends_with("/on") {
            level = RiskLevel::High;
            description = format!(
                "EQ bypass changed: {} -> {}",
                format_arg(current),
                format_arg(scene)
            );
        } else if path.ends_with("/g") || path.ends_with("/gain") {
            if let (OscArg::Float(c), OscArg::Float(s)) = (current, scene) {
                let diff = (c - s).abs();
                if diff > 0.166 {
                    level = RiskLevel::High;
                    description = format!("Dramatic EQ gain change (>{:.2}): {} -> {}", diff, c, s);
                }
            }
        }
    } else if path.ends_with("/mix/fader") {
        if let (OscArg::Float(c), OscArg::Float(s)) = (current, scene) {
            let diff = (c - s).abs();
            if diff > 0.25 {
                level = RiskLevel::Moderate;
                description = format!("Fader level change > 10dB: {:.2} -> {:.2}", c, s);
            }
        }
    } else if path.ends_with("/config/name")
        || path.ends_with("/config/icon")
        || path.ends_with("/config/color")
    {
        level = RiskLevel::Info;
        description = format!(
            "Cosmetic naming/icon change: {} -> {}",
            format_arg(current),
            format_arg(scene)
        );
    } else if path.contains("/dyn/") || path.contains("/gate/") {
        level = RiskLevel::Low;
    }

    Some(RiskIssue {
        level,
        path: path.to_string(),
        description,
        from: current.clone(),
        to: scene.clone(),
    })
}

fn print_report_summary(issues: &[RiskIssue]) {
    println!("\n╔══════════════════════════════════════════════════╗");
    println!("║  SCENE PRE-FLIGHT CHECK                          ║");
    println!("╠══════════════════════════════════════════════════╣");

    // ⚡ Bolt: Iterate through issues once, avoiding 5 separate O(N) heap allocations
    // from multiple .filter().collect() calls. We only collect references for Critical
    // and High because we need to display a few examples of them.
    let mut criticals = Vec::new();
    let mut highs = Vec::new();
    let mut moderates_count = 0;
    let mut lows_count = 0;
    let mut infos_count = 0;

    for i in issues {
        match i.level {
            RiskLevel::Critical => criticals.push(i),
            RiskLevel::High => highs.push(i),
            RiskLevel::Moderate => moderates_count += 1,
            RiskLevel::Low => lows_count += 1,
            RiskLevel::Info => infos_count += 1,
        }
    }

    if !criticals.is_empty() {
        let text = format!("║  🔴 CRITICAL ({} issues)", criticals.len());
        println!("{text}{:<width$}║", "", width = 50 - text.chars().count());
        for i in criticals.iter().take(3) {
            println!("║    • {}: {}", i.path, i.description);
        }
        if criticals.len() > 3 {
            println!("║    ... and {} more", criticals.len() - 3);
        }
        println!("║{:<49}║", "");
    }

    if !highs.is_empty() {
        let text = format!("║  🟠 HIGH ({} issues)", highs.len());
        println!("{text}{:<width$}║", "", width = 50 - text.chars().count());
        for i in highs.iter().take(3) {
            println!("║    • {}: {}", i.path, i.description);
        }
        if highs.len() > 3 {
            println!("║    ... and {} more", highs.len() - 3);
        }
        println!("║{:<49}║", "");
    }

    if moderates_count > 0 {
        println!("║  🟡 MODERATE ({} changes)", moderates_count);
    }

    println!(
        "║  🟢 LOW ({} changes)  ⚪ INFO ({} changes)",
        lows_count, infos_count
    );
    println!("╚══════════════════════════════════════════════════╝");
}

fn print_full_details(issues: &[RiskIssue]) {
    println!("\n--- FULL DETAILS ---");
    let mut sorted_issues = issues.iter().collect::<Vec<_>>();
    sorted_issues.sort_by_key(|i| std::cmp::Reverse(i.level)); // Critical first

    let mut current_level = None;
    for issue in sorted_issues {
        if current_level != Some(issue.level) {
            println!("\n{}", issue.level.name());
            current_level = Some(issue.level);
        }
        println!("  {}: {}", issue.path, issue.description);
    }
    println!("--------------------");
}

pub async fn run(args: Args) -> anyhow::Result<()> {
    let f = std::fs::File::open(&args.scene)?;
    let mut scn_content = String::new();
    f.take(256 * 1024 + 1).read_to_string(&mut scn_content)?;
    if scn_content.len() > 256 * 1024 {
        anyhow::bail!("Scene file is too large (exceeds 256KB)");
    }

    let mut parser = SceneParser::with_model(args.model);
    let mut scene_map: HashMap<String, OscArg> = HashMap::new();
    for line in scn_content.lines() {
        for msg in parser.parse_scene_line(line) {
            if let Some(arg) = msg.args.first() {
                scene_map.insert(msg.path.clone(), arg.clone());
            }
        }
    }

    if scene_map.is_empty() {
        anyhow::bail!("No valid parameters found in scene file");
    }

    println!("Connecting to mixer at {}...", args.ip);
    let client = MixerClient::connect(&args.ip, true).await?;
    let client = std::sync::Arc::new(client);

    println!(
        "Fetching current mixer state for {} parameters...",
        scene_map.len()
    );
    let mut current_map: HashMap<&str, OscArg> = HashMap::with_capacity(scene_map.len());
    let mut count = 0;

    for path in scene_map.keys() {
        match client.query_value(path).await {
            Ok(arg) => {
                current_map.insert(path.as_str(), arg);
            }
            Err(_e) => {
                // Ignore missing parameters
            }
        }
        count += 1;
        if count % 100 == 0 {
            print!(".");
            let _ = std::io::stdout().flush();
        }
    }
    println!();

    let mut issues = Vec::new();
    for (path, scene_arg) in &scene_map {
        if let Some(current_arg) = current_map.get(path.as_str()) {
            if let Some(issue) = classify_risk_with_model(args.model, path, current_arg, scene_arg)
            {
                issues.push(issue);
            }
        } else {
            issues.push(RiskIssue {
                level: RiskLevel::Info,
                path: path.to_string(),
                description: format!(
                    "Could not verify current state, setting to {}",
                    format_arg(scene_arg)
                ),
                from: OscArg::Int(0), // Dummy
                to: scene_arg.clone(),
            });
        }
    }

    if issues.is_empty() {
        println!("No changes detected. Scene matches current state.");
        return Ok(());
    }

    let locked_prefixes: Vec<String> = args
        .locked_paths
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if args.auto_load {
        println!("Auto-loading entire scene...");
        let mut skipped = 0;
        let mut applied = 0;
        for issue in &issues {
            if locked_prefixes.iter().any(|p| issue.path.starts_with(p)) {
                skipped += 1;
                continue;
            }
            client
                .send_message(&issue.path, vec![issue.to.clone()])
                .await?;
            tokio::time::sleep(Duration::from_millis(2)).await; // avoid overwhelming
            applied += 1;
        }
        println!("Scene loaded. Applied {}, skipped {}.", applied, skipped);
        return Ok(());
    }

    loop {
        print_report_summary(&issues);
        if !locked_prefixes.is_empty() {
            println!("Active locks: {:?}", locked_prefixes);
        }
        println!(
            "Options: [L]oad anyway | [S]afe-load (skip critical/high) | [R]eview details | [C]ancel"
        );
        print!("> ");
        let _ = std::io::stdout().flush();

        let mut byte_buf = Vec::new();
        let stdin = std::io::stdin();
        let mut stdin_lock = stdin.lock();
        let mut handle = stdin_lock.by_ref().take(1024);

        match handle.read_until(b'\n', &mut byte_buf) {
            Ok(0) => return Ok(()),
            Err(e) => return Err(e.into()),
            Ok(len) => {
                if len == 1024 && !byte_buf.ends_with(b"\n") {
                    // Line too long, discard remainder
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Input line too long",
                    )
                    .into());
                }
            }
        }

        let input = String::from_utf8_lossy(&byte_buf).into_owned();
        let trimmed = input.trim().to_lowercase();
        match trimmed.as_str() {
            "l" | "load" => {
                println!("Loading entire scene...");
                let mut skipped = 0;
                let mut applied = 0;
                for issue in &issues {
                    if locked_prefixes.iter().any(|p| issue.path.starts_with(p)) {
                        skipped += 1;
                        continue;
                    }
                    client
                        .send_message(&issue.path, vec![issue.to.clone()])
                        .await?;
                    tokio::time::sleep(Duration::from_millis(2)).await; // avoid overwhelming
                    applied += 1;
                }
                println!("Scene loaded. Applied {}, skipped {}.", applied, skipped);
                break;
            }
            "s" | "safe-load" | "safe" => {
                println!("Loading scene safely (skipping CRITICAL and HIGH)...");
                let mut skipped = 0;
                let mut applied = 0;
                for issue in &issues {
                    if issue.level == RiskLevel::Critical
                        || issue.level == RiskLevel::High
                        || locked_prefixes.iter().any(|p| issue.path.starts_with(p))
                    {
                        skipped += 1;
                        continue;
                    }
                    client
                        .send_message(&issue.path, vec![issue.to.clone()])
                        .await?;
                    tokio::time::sleep(Duration::from_millis(2)).await;
                    applied += 1;
                }
                println!(
                    "Safe load complete. Applied {}, skipped {}.",
                    applied, skipped
                );
                break;
            }
            "r" | "review" => {
                print_full_details(&issues);
            }
            "c" | "cancel" => {
                println!("Operation cancelled.");
                break;
            }
            _ => {
                println!("Unknown option. Please enter L, S, R, or C.");
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_model_bounds_channels() {
        let dummy = OscArg::Float(0.0);
        let scene = OscArg::Float(0.5);

        // XR18: 16 channels
        assert!(
            classify_risk_with_model(MixerModel::XR18, "/ch/16/mix/fader", &dummy, &scene)
                .unwrap()
                .level
                != RiskLevel::Critical
        );
        let issue =
            classify_risk_with_model(MixerModel::XR18, "/ch/17/mix/fader", &dummy, &scene).unwrap();
        assert_eq!(issue.level, RiskLevel::Critical);
        assert!(
            issue
                .description
                .contains("Channel index 17 is out of bounds")
        );

        // XR12: 12 channels
        assert!(
            classify_risk_with_model(MixerModel::XR12, "/ch/12/mix/fader", &dummy, &scene)
                .unwrap()
                .level
                != RiskLevel::Critical
        );
        let issue =
            classify_risk_with_model(MixerModel::XR12, "/ch/13/mix/fader", &dummy, &scene).unwrap();
        assert_eq!(issue.level, RiskLevel::Critical);

        // X32: 32 channels
        assert!(
            classify_risk_with_model(MixerModel::X32, "/ch/32/mix/fader", &dummy, &scene)
                .unwrap()
                .level
                != RiskLevel::Critical
        );
        let issue =
            classify_risk_with_model(MixerModel::X32, "/ch/33/mix/fader", &dummy, &scene).unwrap();
        assert_eq!(issue.level, RiskLevel::Critical);

        // Wing: 40 channels
        assert!(
            classify_risk_with_model(MixerModel::Wing, "/ch/40/mix/fader", &dummy, &scene)
                .unwrap()
                .level
                != RiskLevel::Critical
        );
        let issue =
            classify_risk_with_model(MixerModel::Wing, "/ch/41/mix/fader", &dummy, &scene).unwrap();
        assert_eq!(issue.level, RiskLevel::Critical);
    }

    #[test]
    fn test_check_model_bounds_buses() {
        let dummy = OscArg::Float(0.0);
        let scene = OscArg::Float(0.5);

        // XR12: 2 buses
        assert!(
            classify_risk_with_model(MixerModel::XR12, "/bus/02/mix/fader", &dummy, &scene)
                .unwrap()
                .level
                != RiskLevel::Critical
        );
        let issue = classify_risk_with_model(MixerModel::XR12, "/bus/03/mix/fader", &dummy, &scene)
            .unwrap();
        assert_eq!(issue.level, RiskLevel::Critical);

        // XR16: 4 buses
        assert!(
            classify_risk_with_model(MixerModel::XR16, "/bus/04/mix/fader", &dummy, &scene)
                .unwrap()
                .level
                != RiskLevel::Critical
        );
        let issue = classify_risk_with_model(MixerModel::XR16, "/bus/05/mix/fader", &dummy, &scene)
            .unwrap();
        assert_eq!(issue.level, RiskLevel::Critical);

        // XR18: 6 buses
        assert!(
            classify_risk_with_model(MixerModel::XR18, "/bus/06/mix/fader", &dummy, &scene)
                .unwrap()
                .level
                != RiskLevel::Critical
        );
        let issue = classify_risk_with_model(MixerModel::XR18, "/bus/07/mix/fader", &dummy, &scene)
            .unwrap();
        assert_eq!(issue.level, RiskLevel::Critical);

        // X32: 16 buses
        assert!(
            classify_risk_with_model(MixerModel::X32, "/bus/16/mix/fader", &dummy, &scene)
                .unwrap()
                .level
                != RiskLevel::Critical
        );
        let issue =
            classify_risk_with_model(MixerModel::X32, "/bus/17/mix/fader", &dummy, &scene).unwrap();
        assert_eq!(issue.level, RiskLevel::Critical);
    }

    #[test]
    fn test_check_model_bounds_matrices_and_mono() {
        let dummy = OscArg::Float(0.0);
        let scene = OscArg::Float(0.5);

        // Matrices unsupported on XR18/16/12
        let issue = classify_risk_with_model(MixerModel::XR18, "/mtx/01/mix/fader", &dummy, &scene)
            .unwrap();
        assert_eq!(issue.level, RiskLevel::Critical);
        assert!(issue.description.contains("Matrix outputs are unsupported"));

        // Mono main unsupported on XR models
        let issue = classify_risk_with_model(MixerModel::XR18, "/main/m/mix/fader", &dummy, &scene)
            .unwrap();
        assert_eq!(issue.level, RiskLevel::Critical);
        assert!(issue.description.contains("Mono main bus is unsupported"));

        // Matrix valid on X32
        let issue =
            classify_risk_with_model(MixerModel::X32, "/mtx/06/mix/fader", &dummy, &scene).unwrap();
        assert_eq!(issue.level, RiskLevel::Moderate);

        let issue =
            classify_risk_with_model(MixerModel::X32, "/mtx/07/mix/fader", &dummy, &scene).unwrap();
        assert_eq!(issue.level, RiskLevel::Critical);
    }

    #[test]
    fn test_check_model_bounds_eq_bands() {
        let dummy = OscArg::Float(1000.0);
        let scene = OscArg::Float(2000.0);

        // Aux in EQ bands: X32 has 2, XR18 has 4
        let issue =
            classify_risk_with_model(MixerModel::X32, "/auxin/01/eq/3/f", &dummy, &scene).unwrap();
        assert_eq!(issue.level, RiskLevel::Critical);
        assert!(issue.description.contains("EQ band 3 exceeds max bands"));

        let issue =
            classify_risk_with_model(MixerModel::XR18, "/auxin/01/eq/3/f", &dummy, &scene).unwrap();
        assert_eq!(issue.level, RiskLevel::Low);

        // Wing channel has 8 EQ bands
        let issue =
            classify_risk_with_model(MixerModel::Wing, "/ch/01/eq/8/f", &dummy, &scene).unwrap();
        assert_eq!(issue.level, RiskLevel::Low);

        let issue =
            classify_risk_with_model(MixerModel::Wing, "/ch/01/eq/9/f", &dummy, &scene).unwrap();
        assert_eq!(issue.level, RiskLevel::Critical);
    }
}
