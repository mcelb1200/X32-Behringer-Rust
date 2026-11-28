//! Console-first Digital Split, Live/DAW Mode switching, and Turn-On-&-Go orchestration.
//!
//! Designed so an operator can power on the X32 console and computer and run the entire
//! recording / Live <-> DAW Virtual Soundcheck workflow from X32 User Assign buttons 5–12.

use crate::event_recorder::{EventRecorder, RecorderStatus};
use crate::mixer::{discover_first_ip, MixerManager, RoutingState};
use crate::runner::ProcessManager;
use crate::vsc::{
    apply_vsc_routing, compute_vsc_routing_plan_with_options, generate_rpp,
    generate_x32_reaper_ini, resolve_channel_tracks, routing_card_label, send_reaper_transport,
    set_vsc_mode, ReaperIniOptions, RoutingChange, VscRoutingPlan, VscSessionOptions,
};
use crate::UiEvent;
use anyhow::{Context, Result};
use osc_lib::{OscArg, OscMessage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Read as _;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::sync::{broadcast, Mutex, RwLock};
use tokio::task::JoinHandle;

const MAX_PROFILE_BYTES: u64 = 256 * 1024;
const BANK_COLOR_LIVE_RED: i32 = 1;
const BANK_COLOR_DAW_GREEN: i32 = 2;

/// Primary studio operating mode.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OperatingMode {
    /// Live stage inputs routed to FOH (`routswitch = 0`).
    Live,
    /// DAW / REAPER Card returns routed to FOH (`routswitch = 1`).
    Daw,
}

/// Persisted Turn-On & Go profile (`x32_studio_profile.json`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudioProfile {
    /// Target mixer IP or `"auto"` for LAN discovery.
    #[serde(default = "default_auto")]
    pub mixer_ip: String,
    /// Transport (`"auto"`, `"osc"`, `"aes50"`, `"usb"`).
    #[serde(default = "default_auto")]
    pub transport: String,
    /// Automatically discover/connect on startup and reconnect if dropped.
    #[serde(default = "default_true")]
    pub auto_connect_on_boot: bool,
    /// Automatically apply full digital split routing once connected.
    #[serde(default = "default_true")]
    pub auto_apply_digital_split: bool,
    /// Optional AES50 mirror (`"none"`, `"aes50a"`, or `"aes50b"`).
    #[serde(default = "default_none")]
    pub split_mirror_aes50: String,
    /// Also route `PLAY/AUX` to `CARD1-6` during split setup.
    #[serde(default = "default_true")]
    pub include_aux_card: bool,
    /// Brief Main LR mute duration (ms) across `routswitch` transitions to prevent speaker pops (`0` disables).
    #[serde(default = "default_pop_guard_ms")]
    pub pop_guard_mute_ms: u64,
    /// Restore pre-DAW console snapshot when returning from DAW mode to Live mode.
    #[serde(default)]
    pub restore_live_snapshot_on_return: bool,
    /// Program and listen to X32 User Assign buttons 5–12.
    #[serde(default = "default_true")]
    pub enable_console_buttons: bool,
    /// X32 User Assign bank (`"A"`, `"B"`, or `"C"`).
    #[serde(default = "default_bank_c")]
    pub control_bank: String,
    /// Host IP running REAPER.
    #[serde(default = "default_local_ip")]
    pub reaper_ip: String,
    /// UDP port REAPER listens on for incoming OSC.
    #[serde(default = "default_reaper_send_port")]
    pub reaper_send_port: u16,
    /// UDP port `x32_reaper` listens on for REAPER outgoing OSC.
    #[serde(default = "default_reaper_recv_port")]
    pub reaper_recv_port: u16,
    /// Optional `.rpp` project path to pass when auto-launching REAPER.
    #[serde(default)]
    pub reaper_session_path: String,
    /// Automatically launch REAPER on startup/connect if configured.
    #[serde(default)]
    pub auto_launch_reaper: bool,
    /// Path to REAPER executable (optional, defaults to `"reaper"`).
    #[serde(default)]
    pub reaper_exe_path: String,
    /// Directory where `.x32events.jsonl` recordings and pre-DAW snapshots are stored.
    #[serde(default = "default_recordings_dir")]
    pub recordings_dir: String,
    /// Automatically play back matching event recording when entering DAW Play.
    #[serde(default = "default_true")]
    pub sync_event_replay_with_daw: bool,
    /// Automatically start `x32-reaper` bridge while in DAW mode.
    #[serde(default)]
    pub launch_daw_bridge_in_daw_mode: bool,
    /// Preserve `CARD` blocks already assigned to `UOUT1-48` (User Out) instead of overwriting them.
    #[serde(default = "default_true")]
    pub preserve_card_uout: bool,
    /// Automatically generate a fresh digital-twin `.rpp` & `.X32Reaper.ini` from live console state on connect.
    #[serde(default = "default_true")]
    pub auto_generate_rpp_on_connect: bool,
}

fn default_auto() -> String {
    "auto".to_string()
}
fn default_none() -> String {
    "none".to_string()
}
fn default_true() -> bool {
    true
}
fn default_pop_guard_ms() -> u64 {
    60
}
fn default_bank_c() -> String {
    "C".to_string()
}
fn default_local_ip() -> String {
    "127.0.0.1".to_string()
}
fn default_reaper_send_port() -> u16 {
    8000
}
fn default_reaper_recv_port() -> u16 {
    9000
}
fn default_recordings_dir() -> String {
    "recordings".to_string()
}

impl Default for StudioProfile {
    fn default() -> Self {
        Self {
            mixer_ip: default_auto(),
            transport: default_auto(),
            auto_connect_on_boot: true,
            auto_apply_digital_split: true,
            split_mirror_aes50: default_none(),
            include_aux_card: true,
            pop_guard_mute_ms: default_pop_guard_ms(),
            restore_live_snapshot_on_return: false,
            enable_console_buttons: true,
            control_bank: default_bank_c(),
            reaper_ip: default_local_ip(),
            reaper_send_port: default_reaper_send_port(),
            reaper_recv_port: default_reaper_recv_port(),
            reaper_session_path: String::new(),
            auto_launch_reaper: false,
            reaper_exe_path: String::new(),
            recordings_dir: default_recordings_dir(),
            sync_event_replay_with_daw: true,
            launch_daw_bridge_in_daw_mode: false,
            preserve_card_uout: true,
            auto_generate_rpp_on_connect: true,
        }
    }
}

impl StudioProfile {
    /// Loads a [`StudioProfile`] from `path` (bounded to 256 KiB), returning `Default` if missing.
    pub fn load_or_default(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let file = std::fs::File::open(path)?;
        if file.metadata()?.len() > MAX_PROFILE_BYTES {
            anyhow::bail!("Studio profile exceeds {} bytes", MAX_PROFILE_BYTES);
        }
        let mut buf = String::new();
        file.take(MAX_PROFILE_BYTES + 1).read_to_string(&mut buf)?;
        if buf.len() as u64 > MAX_PROFILE_BYTES {
            anyhow::bail!("Studio profile exceeds {} bytes", MAX_PROFILE_BYTES);
        }
        serde_json::from_str(&buf).context("Failed parsing StudioProfile JSON")
    }

    /// Saves the [`StudioProfile`] to `path` as pretty JSON.
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

/// Visual state of one hardware X32 User Assign button (buttons 5–12).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HardwareButtonState {
    /// Physical button number on the X32 User Assign bank (`5..=12`).
    pub button: u8,
    /// Underlying X32 `/-stat/userpar/<nn>/value` parameter index.
    pub userpar: u8,
    /// Short button label shown in the Web UI.
    pub label: String,
    /// Whether the hardware LED is currently lit.
    pub lit: bool,
}

/// Complete live Studio Mode & Digital Split status sent over HTTP/SSE.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudioStatus {
    /// Active operating mode (`live` or `daw`).
    pub mode: OperatingMode,
    /// Whether full digital split routing has been applied.
    pub digital_split_applied: bool,
    /// Whether pop-guard mute is currently active during a transition.
    pub pop_guard_active: bool,
    /// Whether X32 User Assign hardware buttons have been programmed.
    pub console_buttons_programmed: bool,
    /// Active User Assign bank (`"A"`, `"B"`, or `"C"`).
    pub control_bank: String,
    /// Event recorder status.
    pub recorder: RecorderStatus,
    /// Current state of hardware buttons 5–12.
    pub buttons: Vec<HardwareButtonState>,
    /// Active `x32-reaper` bridge job ID if managed by Studio Mode.
    pub reaper_bridge_job_id: Option<String>,
    /// Latest human-readable status message.
    pub last_action: String,
    /// Active persisted profile.
    pub profile: StudioProfile,
}

/// Computes a full digital split routing plan from `current` routing.
///
/// Mirrors physical stage inputs (`IN`) to `CARD` outputs (avoiding card loops and optionally
/// preserving `UOUT` feeds) and routes `PLAY` blocks to `CARD1-8..CARD25-32`. When
/// `mirror_aes50` is `"aes50a"` or `"aes50b"`, mirrors the stage feed onto that AES50 port only
/// if that AES50 port is not already in use for stagebox inputs or active outputs (P16/OUT/CARD/UOUT).
pub fn compute_digital_split_plan(
    current: &RoutingState,
    include_aux_card: bool,
    mirror_aes50: &str,
) -> VscRoutingPlan {
    compute_digital_split_plan_with_options(current, include_aux_card, mirror_aes50, false)
}

/// Computes a digital split routing plan with `preserve_card_uout` and AES50-in-use protection.
pub fn compute_digital_split_plan_with_options(
    current: &RoutingState,
    include_aux_card: bool,
    mirror_aes50: &str,
    preserve_card_uout: bool,
) -> VscRoutingPlan {
    let mut plan =
        compute_vsc_routing_plan_with_options(current, include_aux_card, preserve_card_uout);
    let block_names = ["1-8", "9-16", "17-24", "25-32"];

    match mirror_aes50.to_ascii_lowercase().as_str() {
        "aes50a" => {
            let uses_aes50a_inputs = current.in_blocks.iter().any(|&v| (4..=9).contains(&v));
            let uses_aes50a_outputs = current.aes50a_blocks[..4].iter().any(|&v| v >= 16);
            if uses_aes50a_inputs || uses_aes50a_outputs {
                plan.warnings.push(
                    "AES50-A is already in use (stagebox inputs or active P16/OUT/CARD feeds); skipped AES50-A split mirror.".to_string(),
                );
            } else {
                for i in 0..4 {
                    let target = plan.card_blocks[i];
                    plan.changes.push(RoutingChange {
                        path: format!("/config/routing/AES50A/{}", block_names[i]),
                        label: format!("AES50-A Split {}", block_names[i]),
                        before: current.aes50a_blocks[i],
                        after: target,
                        before_label: routing_card_label(current.aes50a_blocks[i]).to_string(),
                        after_label: routing_card_label(target).to_string(),
                    });
                }
            }
        }
        "aes50b" => {
            let uses_aes50b_inputs = current.in_blocks.iter().any(|&v| (10..=15).contains(&v));
            let uses_aes50b_outputs = current.aes50b_blocks[..4].iter().any(|&v| v >= 16);
            if uses_aes50b_inputs || uses_aes50b_outputs {
                plan.warnings.push(
                    "AES50-B is already in use (stagebox inputs or active P16/OUT/CARD feeds); skipped AES50-B split mirror.".to_string(),
                );
            } else {
                for i in 0..4 {
                    let target = plan.card_blocks[i];
                    plan.changes.push(RoutingChange {
                        path: format!("/config/routing/AES50B/{}", block_names[i]),
                        label: format!("AES50-B Split {}", block_names[i]),
                        before: current.aes50b_blocks[i],
                        after: target,
                        before_label: routing_card_label(current.aes50b_blocks[i]).to_string(),
                        after_label: routing_card_label(target).to_string(),
                    });
                }
            }
        }
        _ => {}
    }

    plan
}

/// Returns `(bank_letter, userbank_idx, first_userpar_index)` for `"A"`, `"B"`, or `"C"`.
pub fn bank_mapping(bank: &str) -> (&'static str, i32, u8) {
    match bank.trim().to_ascii_uppercase().as_str() {
        "A" => ("A", 0, 1),
        "B" => ("B", 1, 9),
        _ => ("C", 2, 17),
    }
}

struct StudioRuntimeState {
    mode: OperatingMode,
    digital_split_applied: bool,
    pop_guard_active: bool,
    console_buttons_programmed: bool,
    reaper_bridge_job_id: Option<String>,
    daw_playing: bool,
    last_action: String,
    initialized_for_ip: Option<String>,
    reaper_launched: bool,
}

/// Orchestrates Console-First Digital Split, Full Event Recording, and Live <-> DAW switching.
pub struct StudioController {
    profile: Arc<RwLock<StudioProfile>>,
    profile_path: PathBuf,
    runtime: Arc<Mutex<StudioRuntimeState>>,
    mode_lock: Arc<Mutex<()>>,
    expected_led_echoes: Arc<Mutex<Vec<(u8, i32, Instant)>>>,
    last_sent_leds: Arc<Mutex<HashMap<u8, i32>>>,
    mixer: Arc<MixerManager>,
    runner: Arc<ProcessManager>,
    recorder: Arc<EventRecorder>,
    events_tx: broadcast::Sender<UiEvent>,
}

impl StudioController {
    /// Creates a new [`StudioController`] and attaches its background OSC listener to `mixer`.
    pub fn new(
        profile: StudioProfile,
        profile_path: PathBuf,
        mixer: Arc<MixerManager>,
        runner: Arc<ProcessManager>,
        events_tx: broadcast::Sender<UiEvent>,
    ) -> Arc<Self> {
        let recorder = Arc::new(EventRecorder::new());
        let sync_replay = profile.sync_event_replay_with_daw;

        let ctrl = Arc::new(Self {
            profile: Arc::new(RwLock::new(profile)),
            profile_path,
            runtime: Arc::new(Mutex::new(StudioRuntimeState {
                mode: OperatingMode::Live,
                digital_split_applied: false,
                pop_guard_active: false,
                console_buttons_programmed: false,
                reaper_bridge_job_id: None,
                daw_playing: false,
                last_action: "Ready (Live Console Mode)".to_string(),
                initialized_for_ip: None,
                reaper_launched: false,
            })),
            mode_lock: Arc::new(Mutex::new(())),
            expected_led_echoes: Arc::new(Mutex::new(Vec::new())),
            last_sent_leds: Arc::new(Mutex::new(HashMap::new())),
            mixer,
            runner,
            recorder,
            events_tx,
        });

        let r_clone = ctrl.recorder.clone();
        tokio::spawn(async move {
            r_clone.set_sync_replay_enabled(sync_replay).await;
        });

        ctrl.spawn_osc_listener();
        ctrl
    }

    fn spawn_osc_listener(self: &Arc<Self>) {
        let ctrl = self.clone();
        let mut rx = self.mixer.subscribe_osc();
        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(msg) => {
                        ctrl.handle_console_osc(&msg).await;
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        });
    }

    /// Spawns the Turn-On & Go auto-connect / auto-init supervisor loop.
    pub fn spawn_supervisor(self: &Arc<Self>) -> JoinHandle<()> {
        let ctrl = self.clone();
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(Duration::from_secs(3));
            loop {
                ticker.tick().await;
                let profile = ctrl.profile.read().await.clone();
                let snap = ctrl.mixer.snapshot().await;

                if !snap.connected {
                    if profile.auto_connect_on_boot {
                        let target_ip = if profile.mixer_ip.eq_ignore_ascii_case("auto")
                            || profile.mixer_ip.trim().is_empty()
                        {
                            match discover_first_ip().await {
                                Ok(Some(found)) => found,
                                _ => continue,
                            }
                        } else {
                            profile.mixer_ip.clone()
                        };
                        if ctrl
                            .mixer
                            .connect(&target_ip, &profile.transport, "", "")
                            .await
                            .is_ok()
                        {
                            let _ = ctrl.initialize_on_console_online().await;
                        }
                    }
                } else {
                    let already_init = {
                        let rt = ctrl.runtime.lock().await;
                        rt.initialized_for_ip.as_deref() == Some(snap.ip.as_str())
                    };
                    if !already_init {
                        let _ = ctrl.initialize_on_console_online().await;
                    }
                }
            }
        })
    }

    /// Runs turn-on-and-go initialization once the console is online (digital split, hardware buttons, optional REAPER launch).
    pub async fn initialize_on_console_online(&self) -> Result<()> {
        let snap = self.mixer.snapshot().await;
        if !snap.connected {
            return Ok(());
        }
        let profile = self.profile.read().await.clone();

        {
            let mut rt = self.runtime.lock().await;
            rt.initialized_for_ip = Some(snap.ip.clone());
            rt.mode = if snap.routing.routswitch == 1 {
                OperatingMode::Daw
            } else {
                OperatingMode::Live
            };
        }

        if profile.auto_apply_digital_split {
            let _ = self.apply_digital_split().await;
        }
        if profile.auto_generate_rpp_on_connect {
            let _ = self.generate_auto_rpp_session(&profile).await;
        }
        if profile.enable_console_buttons {
            let _ = self.program_console_buttons().await;
        }
        if profile.auto_launch_reaper {
            let mut rt = self.runtime.lock().await;
            if !rt.reaper_launched {
                rt.reaper_launched = true;
                drop(rt);
                launch_reaper_process(&profile);
            }
        }
        self.emit_status().await;
        Ok(())
    }

    /// Returns the current [`StudioStatus`].
    pub async fn status(&self) -> StudioStatus {
        let profile = self.profile.read().await.clone();
        let rt = self.runtime.lock().await;
        let rec = self.recorder.status().await;
        let (bank_letter, _, base_up) = bank_mapping(&profile.control_bank);
        let buttons = build_button_states(base_up, &rt, &rec, &profile);

        StudioStatus {
            mode: rt.mode,
            digital_split_applied: rt.digital_split_applied,
            pop_guard_active: rt.pop_guard_active,
            console_buttons_programmed: rt.console_buttons_programmed,
            control_bank: bank_letter.to_string(),
            recorder: rec,
            buttons,
            reaper_bridge_job_id: rt.reaper_bridge_job_id.clone(),
            last_action: rt.last_action.clone(),
            profile,
        }
    }

    /// Updates and persists [`StudioProfile`].
    pub async fn update_profile(&self, updated: StudioProfile) -> Result<StudioStatus> {
        updated.save(&self.profile_path)?;
        self.recorder
            .set_sync_replay_enabled(updated.sync_event_replay_with_daw)
            .await;
        *self.profile.write().await = updated.clone();
        if updated.enable_console_buttons && self.mixer.snapshot().await.connected {
            let _ = self.program_console_buttons().await;
        }
        self.set_last_action("Saved studio profile").await;
        Ok(self.status().await)
    }

    /// Applies full digital split routing (`CARD` Out + `PLAY` Card return + optional AES50 mirror) with backup.
    pub async fn apply_digital_split(&self) -> Result<VscRoutingPlan> {
        let profile = self.profile.read().await.clone();
        let snap = self.mixer.snapshot().await;
        let plan = compute_digital_split_plan_with_options(
            &snap.routing,
            profile.include_aux_card,
            &profile.split_mirror_aes50,
            profile.preserve_card_uout,
        );
        let backup_path = Path::new(&profile.recordings_dir).join("pre_split_routing_backup.json");
        if let Some(parent) = backup_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        apply_vsc_routing(&self.mixer, &plan, Some(&backup_path)).await?;
        {
            let mut rt = self.runtime.lock().await;
            rt.digital_split_applied = true;
            rt.last_action = format!(
                "Applied digital split ({} routing changes)",
                plan.changes.len()
            );
        }
        self.emit_status().await;
        Ok(plan)
    }

    /// Generates a fresh digital-twin `.rpp` & `.X32Reaper.ini` from live console state.
    async fn generate_auto_rpp_session(&self, profile: &StudioProfile) -> Result<PathBuf> {
        let snap = self.mixer.snapshot().await;
        let mut opts = VscSessionOptions::from_mixer_state(&snap);
        opts.digital_twin_mode = true;
        opts.deduplicate_shared_sources = true;

        let rpp = generate_rpp(&opts);
        let target_rpp = if !profile.reaper_session_path.trim().is_empty() {
            PathBuf::from(profile.reaper_session_path.trim())
        } else {
            Path::new(&profile.recordings_dir).join("AutoSession.rpp")
        };
        if let Some(parent) = target_rpp.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        std::fs::write(&target_rpp, rpp)?;

        let ini_path = target_rpp
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(".X32Reaper.ini");
        let resolved = resolve_channel_tracks(&opts);
        let ini = generate_x32_reaper_ini(&ReaperIniOptions {
            x32_ip: snap.ip,
            reaper_ip: profile.reaper_ip.clone(),
            reaper_send_port: profile.reaper_send_port,
            reaper_recv_port: profile.reaper_recv_port,
            trk_min: 1,
            trk_max: (resolved.len().max(1)) as i32,
            ..Default::default()
        });
        std::fs::write(&ini_path, ini)?;
        Ok(target_rpp)
    }

    /// Programs X32 User Assign Bank buttons 5–12 and synchronizes hardware LEDs & bank color.
    pub async fn program_console_buttons(&self) -> Result<()> {
        let profile = self.profile.read().await.clone();
        let (bank_letter, bank_idx, _) = bank_mapping(&profile.control_bank);
        let mn = [
            "MN16000", "MN16001", "MN16002", "MN16003", "MN16004", "MN16005", "MN16006", "MN16007",
        ];

        for i in 5..=12usize {
            let path = format!("/config/userctrl/{}/btn/{}", bank_letter, i);
            self.mixer
                .send_osc(&path, vec![OscArg::String(mn[i - 5].to_string())])
                .await?;
        }
        self.mixer
            .send_osc("/-stat/userbank", vec![OscArg::Int(bank_idx)])
            .await?;

        {
            let mut rt = self.runtime.lock().await;
            rt.console_buttons_programmed = true;
            rt.last_action = format!("Programmed X32 User Assign Bank {} (Buttons 5–12)", bank_letter);
        }
        self.sync_console_leds().await;
        self.emit_status().await;
        Ok(())
    }

    /// Synchronizes X32 User Assign bank color (`Red` = Live, `Green` = DAW) and button LEDs 5–12.
    pub async fn sync_console_leds(&self) {
        if !self.mixer.snapshot().await.connected {
            return;
        }
        let profile = self.profile.read().await.clone();
        let (bank_letter, _, base_up) = bank_mapping(&profile.control_bank);
        let rt = self.runtime.lock().await;
        let rec = self.recorder.status().await;
        let bank_color = match rt.mode {
            OperatingMode::Live => BANK_COLOR_LIVE_RED,
            OperatingMode::Daw => BANK_COLOR_DAW_GREEN,
        };
        let buttons = build_button_states(base_up, &rt, &rec, &profile);
        drop(rt);

        let _ = self
            .mixer
            .send_osc(
                &format!("/config/userctrl/{}/color", bank_letter),
                vec![OscArg::Int(bank_color)],
            )
            .await;

        for b in buttons {
            let led_val = if b.lit { 127 } else { 0 };
            {
                let mut sent = self.last_sent_leds.lock().await;
                if sent.get(&b.userpar) == Some(&led_val) {
                    continue;
                }
                sent.insert(b.userpar, led_val);
            }
            {
                let mut echoes = self.expected_led_echoes.lock().await;
                let now = Instant::now();
                echoes.retain(|(_, _, exp)| *exp > now);
                echoes.push((b.userpar, led_val, now + Duration::from_millis(800)));
            }
            let _ = self
                .mixer
                .send_osc(
                    &format!("/-stat/userpar/{:02}/value", b.userpar),
                    vec![OscArg::Int(led_val)],
                )
                .await;
        }
    }

    /// Gracefully transitions between Live Console Mode and DAW / Virtual Soundcheck Mode.
    pub async fn switch_mode(&self, target: OperatingMode) -> Result<StudioStatus> {
        let _guard = self.mode_lock.lock().await;
        let profile = self.profile.read().await.clone();
        let rec_dir = PathBuf::from(&profile.recordings_dir);
        let snap_path = rec_dir.join("live_pre_daw_snapshot.json");

        match target {
            OperatingMode::Daw => {
                // 1. Flush any active recording cleanly before leaving Live mode
                let rec_state = self.recorder.status().await;
                if rec_state.state == "recording" {
                    let _ = send_reaper_transport(
                        &profile.reaper_ip,
                        profile.reaper_send_port,
                        "stop",
                    )
                    .await;
                    let _ = self.recorder.stop_recording().await;
                }

                // 2. Save pre-DAW console snapshot
                if self.mixer.snapshot().await.connected {
                    let _ = std::fs::create_dir_all(&rec_dir);
                    let _ = self.mixer.save_snapshot_backup(&snap_path).await;
                }

                // 3. Ensure digital split routing is configured
                if !self.runtime.lock().await.digital_split_applied {
                    let _ = self.apply_digital_split().await;
                }

                // 4. Pop-guarded routswitch -> PLAY (1)
                self.switch_routswitch_with_pop_guard(true, profile.pop_guard_mute_ms)
                    .await?;

                // 5. Optional x32-reaper bridge launch
                if profile.launch_daw_bridge_in_daw_mode {
                    let _ = self.ensure_bridge_running(&profile).await;
                }

                {
                    let mut rt = self.runtime.lock().await;
                    rt.mode = OperatingMode::Daw;
                    rt.last_action = "Switched gracefully to DAW Mode (PLAY)".to_string();
                }
            }
            OperatingMode::Live => {
                // 1. Stop any DAW playback + event playback
                let _ = send_reaper_transport(
                    &profile.reaper_ip,
                    profile.reaper_send_port,
                    "stop",
                )
                .await;
                self.recorder.stop_playback().await;

                // 2. Stop managed x32-reaper bridge if running
                let bridge_id = self.runtime.lock().await.reaper_bridge_job_id.take();
                if let Some(job_id) = bridge_id {
                    let _ = self.runner.stop_job(&job_id).await;
                }

                // 3. Pop-guarded routswitch -> REC (0)
                self.switch_routswitch_with_pop_guard(false, profile.pop_guard_mute_ms)
                    .await?;

                // 4. Optional live snapshot restore
                if profile.restore_live_snapshot_on_return && snap_path.exists() {
                    let _ = self.mixer.restore_snapshot_backup(&snap_path).await;
                }

                {
                    let mut rt = self.runtime.lock().await;
                    rt.mode = OperatingMode::Live;
                    rt.daw_playing = false;
                    rt.last_action = "Switched gracefully to Live Console Mode (REC)".to_string();
                }
            }
        }

        self.sync_console_leds().await;
        let status = self.status().await;
        let _ = self.events_tx.send(UiEvent::StudioStatus(status.clone()));
        Ok(status)
    }

    async fn switch_routswitch_with_pop_guard(
        &self,
        play_mode: bool,
        mute_ms: u64,
    ) -> Result<()> {
        let snap = self.mixer.snapshot().await;
        let restore_main = mute_ms > 0 && snap.connected && snap.main_on;
        if restore_main {
            {
                self.runtime.lock().await.pop_guard_active = true;
            }
            let _ = self.mixer.set_mute("main", 1, false).await;
            tokio::time::sleep(Duration::from_millis(12)).await;
        }

        let res = set_vsc_mode(&self.mixer, play_mode, Some(&self.events_tx)).await;

        if restore_main {
            tokio::time::sleep(Duration::from_millis(mute_ms)).await;
            let _ = self.mixer.set_mute("main", 1, true).await;
            self.runtime.lock().await.pop_guard_active = false;
        }
        res
    }

    /// Starts synchronized REAPER audio recording + full X32 OSC event recording.
    pub async fn start_synced_recording(&self, label: Option<&str>) -> Result<StudioStatus> {
        let profile = self.profile.read().await.clone();
        let snap = self.mixer.snapshot().await;
        let full_snapshot = crate::mixer::build_full_snapshot(&snap);
        let session_label = label.unwrap_or("LiveSession");

        let path = self
            .recorder
            .start_recording(Path::new(&profile.recordings_dir), session_label, full_snapshot)
            .await?;

        let _ = send_reaper_transport(&profile.reaper_ip, profile.reaper_send_port, "record").await;

        {
            let mut rt = self.runtime.lock().await;
            rt.daw_playing = false;
            rt.last_action = format!("Recording audio + full console events -> {}", path);
        }
        self.sync_console_leds().await;
        self.emit_status().await;
        Ok(self.status().await)
    }

    /// Stops REAPER recording and flushes the full OSC event recording.
    pub async fn stop_synced_recording(&self) -> Result<StudioStatus> {
        let profile = self.profile.read().await.clone();
        let _ = send_reaper_transport(&profile.reaper_ip, profile.reaper_send_port, "stop").await;
        let saved = self.recorder.stop_recording().await?;
        {
            let mut rt = self.runtime.lock().await;
            rt.last_action = match saved {
                Some(p) => format!("Stopped & saved event recording: {}", p),
                None => "Stopped recording".to_string(),
            };
        }
        self.sync_console_leds().await;
        self.emit_status().await;
        Ok(self.status().await)
    }

    /// Starts synchronized REAPER playback (and event playback if enabled & a recording exists).
    pub async fn start_synced_play(&self, recording_path: Option<&str>) -> Result<StudioStatus> {
        let profile = self.profile.read().await.clone();
        let rec_status = self.recorder.status().await;

        let _ = send_reaper_transport(&profile.reaper_ip, profile.reaper_send_port, "play").await;

        let target_file = recording_path
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .or_else(|| rec_status.active_file.map(PathBuf::from))
            .or_else(|| {
                EventRecorder::list_recordings(Path::new(&profile.recordings_dir))
                    .into_iter()
                    .next()
                    .map(|r| PathBuf::from(r.path))
            });

        if profile.sync_event_replay_with_daw {
            if let Some(ref f) = target_file {
                if f.exists() {
                    let _ = self
                        .recorder
                        .start_playback(
                            f,
                            self.mixer.clone(),
                            true,
                            Some(self.events_tx.clone()),
                        )
                        .await;
                }
            }
        }

        {
            let mut rt = self.runtime.lock().await;
            rt.daw_playing = true;
            rt.last_action = "Playback started (REAPER + Event Replay)".to_string();
        }
        self.sync_console_leds().await;
        self.emit_status().await;
        Ok(self.status().await)
    }

    /// Stops REAPER transport, event recorder, and event playback.
    pub async fn stop_all(&self) -> Result<StudioStatus> {
        let profile = self.profile.read().await.clone();
        let _ = send_reaper_transport(&profile.reaper_ip, profile.reaper_send_port, "stop").await;
        let _ = self.recorder.stop_recording().await;
        self.recorder.stop_playback().await;
        {
            let mut rt = self.runtime.lock().await;
            rt.daw_playing = false;
            rt.last_action = "Transport & event recorder stopped".to_string();
        }
        self.sync_console_leds().await;
        self.emit_status().await;
        Ok(self.status().await)
    }

    /// Drops a marker simultaneously into REAPER (`/action/40157`) and the active event recording.
    pub async fn drop_marker(&self, label: Option<&str>) -> Result<StudioStatus> {
        let profile = self.profile.read().await.clone();
        let addr: SocketAddr = format!("{}:{}", profile.reaper_ip.trim(), profile.reaper_send_port)
            .parse()
            .context("Invalid REAPER address")?;
        if let Ok(sock) = UdpSocket::bind("0.0.0.0:0").await {
            if let Ok(bytes) = OscMessage::new("/action/40157".to_string(), vec![]).to_bytes() {
                let _ = sock.send_to(&bytes, addr).await;
            }
        }

        let marker_label = label
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| "Console Marker".to_string());

        let msg = match self.recorder.add_marker(&marker_label).await {
            Ok(n) => format!("Dropped Marker #{} ({})", n, marker_label),
            Err(_) => "Sent REAPER marker (event recorder idle)".to_string(),
        };
        self.set_last_action(&msg).await;
        Ok(self.status().await)
    }

    /// Toggles `x32-reaper` bridge job on/off.
    pub async fn toggle_bridge(&self) -> Result<StudioStatus> {
        let existing = self.runtime.lock().await.reaper_bridge_job_id.take();
        if let Some(job_id) = existing {
            let _ = self.runner.stop_job(&job_id).await;
            self.set_last_action("Stopped x32-reaper bridge").await;
        } else {
            let profile = self.profile.read().await.clone();
            self.ensure_bridge_running(&profile).await?;
        }
        self.sync_console_leds().await;
        self.emit_status().await;
        Ok(self.status().await)
    }

    async fn ensure_bridge_running(&self, profile: &StudioProfile) -> Result<()> {
        {
            let rt = self.runtime.lock().await;
            if rt.reaper_bridge_job_id.is_some() {
                return Ok(());
            }
        }
        let ini_path = Path::new(&profile.recordings_dir).join(".X32Reaper.ini");
        if !ini_path.exists() {
            let ini = crate::vsc::generate_x32_reaper_ini(&crate::vsc::ReaperIniOptions {
                x32_ip: self.mixer.snapshot().await.ip,
                reaper_ip: profile.reaper_ip.clone(),
                reaper_send_port: profile.reaper_send_port,
                reaper_recv_port: profile.reaper_recv_port,
                ..Default::default()
            });
            if let Some(parent) = ini_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(&ini_path, ini);
        }
        let mut params = HashMap::new();
        params.insert(
            "config".to_string(),
            serde_json::json!(ini_path.to_string_lossy().to_string()),
        );
        params.insert("bind_ip".to_string(), serde_json::json!("127.0.0.1"));
        let job = self.runner.start_tool("x32-reaper", &params).await?;
        let mut rt = self.runtime.lock().await;
        rt.reaper_bridge_job_id = Some(job.job_id.clone());
        rt.last_action = format!("Started x32-reaper bridge ({})", job.job_id);
        Ok(())
    }

    /// Processes an incoming OSC message from the X32 console (event recording + User Assign buttons).
    pub async fn handle_console_osc(&self, msg: &OscMessage) {
        self.recorder.on_incoming_osc(msg).await;

        let profile = self.profile.read().await.clone();
        if !profile.enable_console_buttons {
            return;
        }

        if let Some(rest) = msg.path.strip_prefix("/-stat/userpar/") {
            if let Some(num_str) = rest.strip_suffix("/value") {
                if let (Ok(up_num), Some(OscArg::Int(val))) =
                    (num_str.parse::<u8>(), msg.args.first())
                {
                    let is_self_echo = {
                        let mut echoes = self.expected_led_echoes.lock().await;
                        let now = Instant::now();
                        echoes.retain(|(_, _, exp)| *exp > now);
                        if let Some(pos) =
                            echoes.iter().position(|(u, v, _)| *u == up_num && *v == *val)
                        {
                            echoes.remove(pos);
                            true
                        } else {
                            false
                        }
                    };
                    if is_self_echo {
                        return;
                    }
                    if *val > 0 {
                        let (_, _, base_up) = bank_mapping(&profile.control_bank);
                        if up_num >= base_up && up_num < base_up + 8 {
                            let btn_offset = up_num - base_up;
                            self.dispatch_console_button(btn_offset).await;
                        }
                    }
                }
            }
        }
    }

    /// Executes the action mapped to console button offset `0..=7` (physical buttons 5–12).
    pub async fn dispatch_console_button(&self, offset: u8) {
        match offset {
            0 => {
                // Btn 5: Toggle Live <-> DAW
                let cur = self.runtime.lock().await.mode;
                let next = match cur {
                    OperatingMode::Live => OperatingMode::Daw,
                    OperatingMode::Daw => OperatingMode::Live,
                };
                let _ = self.switch_mode(next).await;
            }
            1 => {
                // Btn 6: Toggle Synced Record
                let is_rec = self.recorder.status().await.state == "recording";
                if is_rec {
                    let _ = self.stop_synced_recording().await;
                } else {
                    let _ = self.start_synced_recording(None).await;
                }
            }
            2 => {
                // Btn 7: Synced Play
                let _ = self.start_synced_play(None).await;
            }
            3 => {
                // Btn 8: Stop All
                let _ = self.stop_all().await;
            }
            4 => {
                // Btn 9: Drop Marker
                let _ = self.drop_marker(None).await;
            }
            5 => {
                // Btn 10: Toggle Event Replay Sync
                let mut prof = self.profile.write().await;
                prof.sync_event_replay_with_daw = !prof.sync_event_replay_with_daw;
                let enabled = prof.sync_event_replay_with_daw;
                let _ = prof.save(&self.profile_path);
                drop(prof);
                self.recorder.set_sync_replay_enabled(enabled).await;
                self.sync_console_leds().await;
                self.set_last_action(if enabled {
                    "Event Replay Sync ENABLED"
                } else {
                    "Event Replay Sync DISABLED"
                })
                .await;
            }
            6 => {
                // Btn 11: Instant Snapshot Backup
                let dir = self.profile.read().await.recordings_dir.clone();
                let _ = std::fs::create_dir_all(&dir);
                let p = Path::new(&dir).join("console_quick_snapshot.json");
                if let Ok(n) = self.mixer.save_snapshot_backup(&p).await {
                    self.set_last_action(&format!("Saved quick snapshot ({} params)", n))
                        .await;
                }
            }
            7 => {
                // Btn 12: Toggle x32-reaper DAW Bridge
                let _ = self.toggle_bridge().await;
            }
            _ => {}
        }
    }

    async fn set_last_action(&self, msg: &str) {
        self.runtime.lock().await.last_action = msg.to_string();
        self.emit_status().await;
    }

    async fn emit_status(&self) {
        let st = self.status().await;
        let _ = self.events_tx.send(UiEvent::StudioStatus(st));
    }
}

fn build_button_states(
    base_up: u8,
    rt: &StudioRuntimeState,
    rec: &RecorderStatus,
    profile: &StudioProfile,
) -> Vec<HardwareButtonState> {
    let is_daw = rt.mode == OperatingMode::Daw;
    let is_rec = rec.state == "recording";
    let is_play = rt.daw_playing || rec.state == "playing";
    let labels = [
        ("5: LIVE/DAW", is_daw),
        ("6: REC", is_rec),
        ("7: PLAY", is_play),
        ("8: STOP", !is_rec && !is_play),
        ("9: MARKER", is_rec),
        ("10: EVT SYNC", profile.sync_event_replay_with_daw),
        ("11: SNAPSHOT", false),
        ("12: DAW BRIDGE", rt.reaper_bridge_job_id.is_some()),
    ];
    labels
        .into_iter()
        .enumerate()
        .map(|(i, (label, lit))| HardwareButtonState {
            button: 5 + i as u8,
            userpar: base_up + i as u8,
            label: label.to_string(),
            lit,
        })
        .collect()
}

fn launch_reaper_process(profile: &StudioProfile) {
    let exe = if profile.reaper_exe_path.trim().is_empty() {
        "reaper"
    } else {
        profile.reaper_exe_path.trim()
    };
    let mut cmd = std::process::Command::new(exe);
    if !profile.reaper_session_path.trim().is_empty() {
        cmd.arg(profile.reaper_session_path.trim());
    } else {
        let auto_rpp = Path::new(&profile.recordings_dir).join("AutoSession.rpp");
        if auto_rpp.exists() {
            cmd.arg(auto_rpp);
        }
    }
    let _ = cmd.spawn();
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_digital_split_plan_mirrors_aes50_safely_and_prevents_stagebox_loop() {
        let mut current = RoutingState::default();
        current.in_blocks = [0, 1, 2, 3]; // Local AN1-32

        // Mirror to AES50A when inputs are Local -> safe to add AES50A 1-32 mirror
        let plan_a = compute_digital_split_plan(&current, true, "aes50a");
        assert!(plan_a
            .changes
            .iter()
            .any(|c| c.path == "/config/routing/AES50A/1-8" && c.after == 0));

        // Now pretend stage inputs come from AES50A (4..9) -> must NOT overwrite AES50A
        current.in_blocks = [4, 5, 2, 3];
        let plan_no_loop = compute_digital_split_plan(&current, false, "aes50a");
        assert!(!plan_no_loop
            .changes
            .iter()
            .any(|c| c.path.starts_with("/config/routing/AES50A/")));
        // But mirroring to AES50B is still allowed!
        let plan_b = compute_digital_split_plan(&current, false, "aes50b");
        assert!(plan_b
            .changes
            .iter()
            .any(|c| c.path == "/config/routing/AES50B/1-8" && c.after == 4));

        // When AES50B already has active OUT/P16/CARD outputs (>= 16), it must NOT be overwritten
        current.aes50b_blocks = [20, 21, 22, 23, 0, 0];
        let plan_b_protected = compute_digital_split_plan_with_options(&current, false, "aes50b", true);
        assert!(!plan_b_protected
            .changes
            .iter()
            .any(|c| c.path.starts_with("/config/routing/AES50B/")));
        assert!(!plan_b_protected.warnings.is_empty());
    }

    #[test]
    fn test_studio_profile_save_and_load_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let p = tmp.path().join("x32_studio_profile.json");
        let mut prof = StudioProfile::default();
        prof.mixer_ip = "192.168.1.64".to_string();
        prof.split_mirror_aes50 = "aes50b".to_string();
        prof.pop_guard_mute_ms = 120;
        prof.control_bank = "B".to_string();
        prof.save(&p).unwrap();

        let loaded = StudioProfile::load_or_default(&p).unwrap();
        assert_eq!(loaded.mixer_ip, "192.168.1.64");
        assert_eq!(loaded.split_mirror_aes50, "aes50b");
        assert_eq!(loaded.pop_guard_mute_ms, 120);
        assert_eq!(loaded.control_bank, "B");
    }
}
