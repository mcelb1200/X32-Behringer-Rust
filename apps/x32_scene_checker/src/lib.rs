use clap::Parser;
use osc_lib::OscArg;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::time::Duration;
use x32_lib::scene_parse::SceneParser;
use x32_lib::MixerClient;

#[derive(Parser, Debug)]
#[command(author, version, about = "Intelligent Scene Pre-flight Checker", long_about = None)]
pub struct Args {
    #[arg(short, long, default_value = "192.168.0.64")]
    pub ip: String,

    #[arg(short, long)]
    pub scene: String,
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

#[derive(Debug)]
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

    let mut level = RiskLevel::Low;
    let mut description = format!("Change from {} to {}", format_arg(current), format_arg(scene));

    if path.starts_with("/routing/") || path.ends_with("/config/source") {
        level = RiskLevel::Critical;
        description = format!("Routing change! From {} to {}", format_arg(current), format_arg(scene));
    } else if path.starts_with("/main/st/mix/on") || (path.contains("/mix/on") && (path.starts_with("/bus/") || path.starts_with("/mtx/"))) {
        level = RiskLevel::Critical;
        description = format!("Output mute state change: {} -> {}", format_arg(current), format_arg(scene));
    } else if path.ends_with("/preamp/trim") {
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
            description = format!("EQ bypass changed: {} -> {}", format_arg(current), format_arg(scene));
        } else if path.ends_with("/g") {
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
    } else if path.ends_with("/config/name") || path.ends_with("/config/icon") || path.ends_with("/config/color") {
        level = RiskLevel::Info;
        description = format!("Cosmetic naming/icon change: {} -> {}", format_arg(current), format_arg(scene));
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

    let criticals: Vec<_> = issues.iter().filter(|i| i.level == RiskLevel::Critical).collect();
    let highs: Vec<_> = issues.iter().filter(|i| i.level == RiskLevel::High).collect();
    let moderates: Vec<_> = issues.iter().filter(|i| i.level == RiskLevel::Moderate).collect();
    let lows: Vec<_> = issues.iter().filter(|i| i.level == RiskLevel::Low).collect();
    let infos: Vec<_> = issues.iter().filter(|i| i.level == RiskLevel::Info).collect();

    if !criticals.is_empty() {
        println!("║  🔴 CRITICAL ({} issues){width:<width$}║", "", width = 50 - format!("║  🔴 CRITICAL ({} issues)", criticals.len()).chars().count());
        for i in criticals.iter().take(3) {
            println!("║    • {}: {}", i.path, i.description);
        }
        if criticals.len() > 3 {
            println!("║    ... and {} more", criticals.len() - 3);
        }
        println!("║{:<49}║", "");
    }

    if !highs.is_empty() {
        println!("║  🟠 HIGH ({} issues){width:<width$}║", "", width = 50 - format!("║  🟠 HIGH ({} issues)", highs.len()).chars().count());
        for i in highs.iter().take(3) {
            println!("║    • {}: {}", i.path, i.description);
        }
        if highs.len() > 3 {
            println!("║    ... and {} more", highs.len() - 3);
        }
        println!("║{:<49}║", "");
    }

    if !moderates.is_empty() {
        println!("║  🟡 MODERATE ({} changes)", moderates.len());
    }

    println!("║  🟢 LOW ({} changes)  ⚪ INFO ({} changes)", lows.len(), infos.len());
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

    let mut parser = SceneParser::new();
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

    println!("Fetching current mixer state for {} parameters...", scene_map.len());
    let mut current_map: HashMap<String, OscArg> = HashMap::new();
    let mut count = 0;

    for path in scene_map.keys() {
        match client.query_value(path).await {
            Ok(arg) => {
                current_map.insert(path.clone(), arg);
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
        if let Some(current_arg) = current_map.get(path) {
            if let Some(issue) = classify_risk(path, current_arg, scene_arg) {
                issues.push(issue);
            }
        } else {
            issues.push(RiskIssue {
                level: RiskLevel::Info,
                path: path.to_string(),
                description: format!("Could not verify current state, setting to {}", format_arg(scene_arg)),
                from: OscArg::Int(0), // Dummy
                to: scene_arg.clone(),
            });
        }
    }

    if issues.is_empty() {
        println!("No changes detected. Scene matches current state.");
        return Ok(());
    }

    loop {
        print_report_summary(&issues);
        println!("Options: [L]oad anyway | [S]afe-load (skip critical/high) | [R]eview details | [C]ancel");
        print!("> ");
        let _ = std::io::stdout().flush();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        let trimmed = input.trim().to_lowercase();
        match trimmed.as_str() {
            "l" | "load" => {
                println!("Loading entire scene...");
                for issue in &issues {
                    client.send_message(&issue.path, vec![issue.to.clone()]).await?;
                    tokio::time::sleep(Duration::from_millis(2)).await; // avoid overwhelming
                }
                println!("Scene loaded.");
                break;
            }
            "s" | "safe-load" | "safe" => {
                println!("Loading scene safely (skipping CRITICAL and HIGH)...");
                let mut skipped = 0;
                let mut applied = 0;
                for issue in &issues {
                    if issue.level == RiskLevel::Critical || issue.level == RiskLevel::High {
                        skipped += 1;
                        continue;
                    }
                    client.send_message(&issue.path, vec![issue.to.clone()]).await?;
                    tokio::time::sleep(Duration::from_millis(2)).await;
                    applied += 1;
                }
                println!("Safe load complete. Applied {}, skipped {}.", applied, skipped);
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
