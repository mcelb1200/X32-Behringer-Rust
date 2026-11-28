//! Consolidated Web UI and Reaper Virtual Soundcheck automation for Behringer X32/M32/Wing/X-Air mixers.
//!
//! Provides a zero-external-HTTP-dependency `tokio` HTTP/1.1 + Server-Sent Events (SSE) server
//! serving an embedded responsive pro-audio dashboard for live mixing, routing, OSC control,
//! launching all 48 workspace utilities, and automating full Reaper Virtual Soundcheck setups.

use anyhow::Result;
use clap::Parser;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::broadcast;

/// Full console OSC event recorder and synchronized replayer.
pub mod event_recorder;
/// Live mixer connection, OSC state caching, and control actions.
pub mod mixer;
/// Child-process manager for launching workspace X32 tools.
pub mod runner;
/// Offline X32 `.scn` scene file parser.
pub mod scene_import;
/// Embedded HTTP/1.1 and SSE server.
pub mod server;
/// Console-first digital split, Live/DAW mode switching, and Turn-On-&-Go orchestration.
pub mod studio_mode;
/// Complete catalog of workspace X32/X-Air CLI tools and parameters.
pub mod tools_catalog;
/// Reaper Virtual Soundcheck automation (routing, `.rpp`, `.X32Reaper.ini`, live OSC).
pub mod vsc;

use mixer::MixerManager;
use runner::ProcessManager;
use studio_mode::{StudioController, StudioProfile};

/// Command-line arguments for the consolidated X32 Web UI.
#[derive(Parser, Debug, Clone)]
#[command(
    author,
    version,
    about = "Consolidated X32 Web UI & Reaper Virtual Soundcheck Automation"
)]
pub struct Args {
    /// IP address to bind the Web UI HTTP server to.
    #[arg(long, default_value = "127.0.0.1")]
    pub bind_ip: String,

    /// TCP port for the Web UI HTTP server.
    #[arg(short, long, default_value_t = 8080)]
    pub port: u16,

    /// Optional mixer IP address (or "auto" for LAN discovery) to connect on startup.
    #[arg(short, long)]
    pub ip: Option<String>,

    /// Transport mode (`auto`, `osc`, `aes50`, `usb`).
    #[arg(long, default_value = "auto")]
    pub transport: String,

    /// Optional AES50 bridge IP address.
    #[arg(long, default_value = "")]
    pub aes50_ip: String,

    /// Optional USB MIDI port name.
    #[arg(long, default_value = "")]
    pub usb_port: String,

    /// Automatically open the Web UI URL in the default browser on startup.
    #[arg(long, default_value_t = false)]
    pub open: bool,

    /// Path to persisted Turn-On & Go studio profile JSON.
    #[arg(long, default_value = "x32_studio_profile.json")]
    pub profile: PathBuf,

    /// Enable Turn-On & Go auto-connect & console initialization supervisor on startup.
    #[arg(long, default_value_t = false)]
    pub auto_boot: bool,
}

/// Real-time Server-Sent Event pushed to connected Web UI clients.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type", content = "data")]
pub enum UiEvent {
    /// Full or incremental mixer state update.
    State(mixer::MixerSnapshotState),
    /// Incoming or outgoing OSC message log line.
    Osc(mixer::OscLogEntry),
    /// Output line from a running tool process.
    JobLog {
        /// Unique job identifier.
        job_id: String,
        /// Output stream ("stdout" or "stderr").
        stream: String,
        /// Log line text.
        line: String,
    },
    /// Status transition for a tool job.
    JobStatus(runner::JobSummary),
    /// Virtual Soundcheck status notification.
    VscStatus {
        /// Status message text.
        message: String,
        /// Whether the X32 is currently in PLAY (Virtual Soundcheck) mode.
        play_mode: bool,
    },
    /// Console-first Studio Mode, Digital Split, and Event Recorder status update.
    StudioStatus(studio_mode::StudioStatus),
}

/// Shared application state passed across HTTP request handlers.
pub struct AppContext {
    /// Live mixer state and OSC controller.
    pub mixer: Arc<MixerManager>,
    /// Child-process manager for workspace tools.
    pub runner: Arc<ProcessManager>,
    /// Console-first Digital Split, Mode Switcher, and Event Recorder orchestrator.
    pub studio: Arc<StudioController>,
    /// Broadcast channel for Server-Sent Events.
    pub events_tx: broadcast::Sender<UiEvent>,
}

impl AppContext {
    /// Creates a new shared [`AppContext`] loading `x32_studio_profile.json` if present.
    pub fn new() -> Self {
        Self::with_profile_path(PathBuf::from("x32_studio_profile.json"))
    }

    /// Creates a new shared [`AppContext`] backed by the profile file at `profile_path`.
    pub fn with_profile_path(profile_path: PathBuf) -> Self {
        let profile = StudioProfile::load_or_default(&profile_path).unwrap_or_default();
        let (events_tx, _) = broadcast::channel(256);
        let mixer = Arc::new(MixerManager::new(events_tx.clone()));
        let runner = Arc::new(ProcessManager::new(events_tx.clone()));
        let studio = StudioController::new(
            profile,
            profile_path,
            mixer.clone(),
            runner.clone(),
            events_tx.clone(),
        );
        Self {
            mixer,
            runner,
            studio,
            events_tx,
        }
    }
}

impl Default for AppContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Launches the consolidated X32 Web UI server and blocks until Ctrl+C.
pub async fn run(args: Args) -> Result<()> {
    let ctx = Arc::new(AppContext::with_profile_path(args.profile.clone()));
    let profile_snapshot = ctx.studio.status().await.profile;

    if let Some(ref ip) = args.ip {
        if !ip.trim().is_empty() {
            let target_ip = if ip.eq_ignore_ascii_case("auto") {
                match mixer::discover_first_ip().await {
                    Ok(Some(found)) => found,
                    _ => "127.0.0.1:10023".to_string(),
                }
            } else {
                ip.clone()
            };
            if let Err(err) = ctx
                .mixer
                .connect(&target_ip, &args.transport, &args.aes50_ip, &args.usb_port)
                .await
            {
                eprintln!("Startup mixer connection to '{}' failed: {}", target_ip, err);
            } else {
                let _ = ctx.studio.initialize_on_console_online().await;
            }
        }
    }

    let supervisor_handle = if args.auto_boot || profile_snapshot.auto_connect_on_boot {
        Some(ctx.studio.spawn_supervisor())
    } else {
        None
    };

    let bind_addr: SocketAddr = format!("{}:{}", args.bind_ip, args.port).parse()?;
    let (bound_addr, server_handle) = server::spawn_server(bind_addr, ctx.clone()).await?;
    let url = format!("http://{}/", bound_addr);
    println!("X32 Consolidated Web UI listening at {}", url);

    if args.open {
        open_browser(&url);
    }

    tokio::signal::ctrl_c().await?;
    println!("\nShutting down X32 Web UI...");
    if let Some(h) = supervisor_handle {
        h.abort();
    }
    ctx.runner.stop_all().await;
    server_handle.abort();
    Ok(())
}

fn open_browser(url: &str) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(url).spawn();
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    }
}
