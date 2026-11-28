//! Zero-dependency async HTTP/1.1 and Server-Sent Events (SSE) server for the X32 Web UI.

use crate::event_recorder::EventRecorder;
use crate::scene_import::{parse_x32_scene_file, parse_x32_scene_str};
use crate::studio_mode::{OperatingMode, StudioProfile};
use crate::tools_catalog::all_tools;
use crate::vsc::{
    adapt_rpp, apply_vsc_routing, compute_vsc_routing_plan_with_options, generate_rpp,
    generate_x32_reaper_ini, push_reaper_osc, resolve_channel_tracks, send_reaper_transport,
    set_vsc_mode, ReaperIniOptions, VscRoutingPlan, VscSessionOptions,
};
use crate::AppContext;
use anyhow::Result;
use osc_lib::OscArg;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Read as _;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

const MAX_HEADER_BYTES: usize = 16 * 1024;
const MAX_BODY_BYTES: usize = 4 * 1024 * 1024;
const MAX_RPP_FILE_BYTES: u64 = 4 * 1024 * 1024;

const INDEX_HTML: &str = include_str!("web/index.html");
const APP_CSS: &str = include_str!("web/app.css");
const APP_JS: &str = include_str!("web/app.js");

/// Binds the HTTP server to `bind_addr` and spawns its accept loop in the background.
pub async fn spawn_server(
    bind_addr: SocketAddr,
    ctx: Arc<AppContext>,
) -> Result<(SocketAddr, JoinHandle<()>)> {
    let listener = TcpListener::bind(bind_addr).await?;
    let local_addr = listener.local_addr()?;

    let handle = tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((stream, _peer)) => {
                    let ctx_clone = ctx.clone();
                    tokio::spawn(async move {
                        let _ = handle_connection(stream, ctx_clone).await;
                    });
                }
                Err(err) => {
                    eprintln!("HTTP accept error: {}", err);
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
            }
        }
    });

    Ok((local_addr, handle))
}

struct HttpRequest {
    method: String,
    path: String,
    body: Vec<u8>,
}

async fn read_http_request(stream: &mut TcpStream) -> Result<Option<HttpRequest>> {
    let mut buf = Vec::with_capacity(2048);
    let mut tmp = [0u8; 2048];
    let header_end;

    loop {
        if let Some(pos) = find_subsequence(&buf, b"\r\n\r\n") {
            header_end = pos;
            break;
        }
        if buf.len() >= MAX_HEADER_BYTES {
            anyhow::bail!("HTTP headers exceed {} bytes", MAX_HEADER_BYTES);
        }
        let n = stream.read(&mut tmp).await?;
        if n == 0 {
            return Ok(None);
        }
        buf.extend_from_slice(&tmp[..n]);
    }

    let header_bytes = &buf[..header_end];
    let header_str = std::str::from_utf8(header_bytes)?;
    let mut lines = header_str.split("\r\n");
    let request_line = lines.next().unwrap_or("");
    let mut req_parts = request_line.split_whitespace();
    let method = req_parts.next().unwrap_or("").to_uppercase();
    let raw_target = req_parts.next().unwrap_or("/");
    let path = raw_target
        .split('?')
        .next()
        .unwrap_or("/")
        .to_string();

    let mut content_length: usize = 0;
    for line in lines {
        if let Some((k, v)) = line.split_once(':') {
            if k.trim().eq_ignore_ascii_case("content-length") {
                content_length = v.trim().parse::<usize>().unwrap_or(0);
            }
        }
    }

    if content_length > MAX_BODY_BYTES {
        anyhow::bail!("Request body too large ({} > {})", content_length, MAX_BODY_BYTES);
    }

    let mut body = buf[(header_end + 4)..].to_vec();
    while body.len() < content_length {
        let remaining = content_length - body.len();
        let to_read = remaining.min(tmp.len());
        let n = stream.read(&mut tmp[..to_read]).await?;
        if n == 0 {
            break;
        }
        body.extend_from_slice(&tmp[..n]);
    }
    body.truncate(content_length);

    Ok(Some(HttpRequest { method, path, body }))
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|w| w == needle)
}

async fn handle_connection(mut stream: TcpStream, ctx: Arc<AppContext>) -> Result<()> {
    let req = match read_http_request(&mut stream).await {
        Ok(Some(r)) => r,
        Ok(None) => return Ok(()),
        Err(err) => {
            write_json_error(&mut stream, 400, &err.to_string()).await?;
            return Ok(());
        }
    };

    if req.method == "GET" && req.path == "/api/events" {
        return handle_sse(stream, ctx).await;
    }

    match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/") | ("GET", "/index.html") => {
            write_response(&mut stream, 200, "text/html; charset=utf-8", INDEX_HTML.as_bytes()).await
        }
        ("GET", "/app.css") => {
            write_response(&mut stream, 200, "text/css; charset=utf-8", APP_CSS.as_bytes()).await
        }
        ("GET", "/app.js") => {
            write_response(
                &mut stream,
                200,
                "application/javascript; charset=utf-8",
                APP_JS.as_bytes(),
            )
            .await
        }
        ("GET", "/api/health") => {
            write_json(
                &mut stream,
                200,
                &serde_json::json!({ "status": "ok", "version": env!("CARGO_PKG_VERSION") }),
            )
            .await
        }
        ("GET", "/api/discover") => match ctx.mixer.discover().await {
            Ok(list) => write_json(&mut stream, 200, &list).await,
            Err(e) => write_json_error(&mut stream, 500, &e.to_string()).await,
        },
        ("GET", "/api/state") => {
            let snap = ctx.mixer.snapshot().await;
            write_json(&mut stream, 200, &snap).await
        }
        ("POST", "/api/connect") => handle_connect(&mut stream, &ctx, &req.body).await,
        ("POST", "/api/disconnect") => {
            ctx.mixer.disconnect().await;
            let snap = ctx.mixer.snapshot().await;
            write_json(&mut stream, 200, &snap).await
        }
        ("POST", "/api/refresh") => match ctx.mixer.refresh_full_state().await {
            Ok(snap) => write_json(&mut stream, 200, &snap).await,
            Err(e) => write_json_error(&mut stream, 400, &e.to_string()).await,
        },
        ("POST", "/api/mixer/fader") => handle_fader(&mut stream, &ctx, &req.body).await,
        ("POST", "/api/mixer/mute") => handle_mute(&mut stream, &ctx, &req.body).await,
        ("POST", "/api/mixer/pan") => handle_pan(&mut stream, &ctx, &req.body).await,
        ("POST", "/api/mixer/channel-config") => {
            handle_channel_config(&mut stream, &ctx, &req.body).await
        }
        ("POST", "/api/mixer/routing") => handle_routing(&mut stream, &ctx, &req.body).await,
        ("POST", "/api/mixer/panic-mute") => match ctx.mixer.panic_mute().await {
            Ok(()) => write_json(&mut stream, 200, &serde_json::json!({ "ok": true })).await,
            Err(e) => write_json_error(&mut stream, 400, &e.to_string()).await,
        },
        ("POST", "/api/mixer/snapshot-save") => {
            handle_snapshot_save(&mut stream, &ctx, &req.body).await
        }
        ("POST", "/api/mixer/snapshot-restore") => {
            handle_snapshot_restore(&mut stream, &ctx, &req.body).await
        }
        ("POST", "/api/osc") => handle_raw_osc(&mut stream, &ctx, &req.body).await,
        ("GET", "/api/tools") => {
            let tools = all_tools();
            let jobs = ctx.runner.list_jobs().await;
            write_json(
                &mut stream,
                200,
                &serde_json::json!({ "tools": tools, "jobs": jobs }),
            )
            .await
        }
        ("GET", "/api/tools/jobs") => {
            let jobs = ctx.runner.list_jobs().await;
            write_json(&mut stream, 200, &jobs).await
        }
        ("POST", "/api/tools/run") => handle_tool_run(&mut stream, &ctx, &req.body).await,
        ("POST", "/api/tools/stop") => handle_tool_stop(&mut stream, &ctx, &req.body).await,
        ("POST", "/api/vsc/capture") => handle_vsc_capture(&mut stream, &ctx, &req.body).await,
        ("POST", "/api/vsc/import-scene") => handle_vsc_import_scene(&mut stream, &req.body).await,
        ("POST", "/api/vsc/apply-routing") => {
            handle_vsc_apply_routing(&mut stream, &ctx, &req.body).await
        }
        ("POST", "/api/vsc/mode") => handle_vsc_mode(&mut stream, &ctx, &req.body).await,
        ("POST", "/api/vsc/restore-backup") => {
            handle_snapshot_restore(&mut stream, &ctx, &req.body).await
        }
        ("POST", "/api/vsc/generate-rpp") => {
            handle_vsc_generate_rpp(&mut stream, &ctx, &req.body).await
        }
        ("POST", "/api/vsc/reaper-osc") => handle_vsc_reaper_osc(&mut stream, &req.body).await,
        ("GET", "/api/studio/status") => {
            let status = ctx.studio.status().await;
            write_json(&mut stream, 200, &status).await
        }
        ("POST", "/api/studio/profile") => {
            handle_studio_profile(&mut stream, &ctx, &req.body).await
        }
        ("POST", "/api/studio/apply-split") => match ctx.studio.apply_digital_split().await {
            Ok(plan) => {
                let status = ctx.studio.status().await;
                write_json(
                    &mut stream,
                    200,
                    &serde_json::json!({ "ok": true, "plan": plan, "status": status }),
                )
                .await
            }
            Err(e) => write_json_error(&mut stream, 400, &e.to_string()).await,
        },
        ("POST", "/api/studio/mode") => handle_studio_mode(&mut stream, &ctx, &req.body).await,
        ("POST", "/api/studio/record/start") => {
            handle_studio_record_start(&mut stream, &ctx, &req.body).await
        }
        ("POST", "/api/studio/record/stop") => match ctx.studio.stop_synced_recording().await {
            Ok(status) => write_json(&mut stream, 200, &status).await,
            Err(e) => write_json_error(&mut stream, 400, &e.to_string()).await,
        },
        ("POST", "/api/studio/play") => handle_studio_play(&mut stream, &ctx, &req.body).await,
        ("POST", "/api/studio/stop") => match ctx.studio.stop_all().await {
            Ok(status) => write_json(&mut stream, 200, &status).await,
            Err(e) => write_json_error(&mut stream, 400, &e.to_string()).await,
        },
        ("POST", "/api/studio/marker") => {
            handle_studio_marker(&mut stream, &ctx, &req.body).await
        }
        ("POST", "/api/studio/program-buttons") => {
            match ctx.studio.program_console_buttons().await {
                Ok(()) => {
                    let status = ctx.studio.status().await;
                    write_json(&mut stream, 200, &status).await
                }
                Err(e) => write_json_error(&mut stream, 400, &e.to_string()).await,
            }
        }
        ("POST", "/api/studio/button") => handle_studio_button(&mut stream, &ctx, &req.body).await,
        ("GET", "/api/studio/recordings") => {
            let status = ctx.studio.status().await;
            let list = EventRecorder::list_recordings(Path::new(&status.profile.recordings_dir));
            write_json(&mut stream, 200, &list).await
        }
        _ => write_json_error(&mut stream, 404, "Not found").await,
    }
}

async fn handle_sse(mut stream: TcpStream, ctx: Arc<AppContext>) -> Result<()> {
    let headers = "HTTP/1.1 200 OK\r\n\
        Content-Type: text/event-stream\r\n\
        Cache-Control: no-cache\r\n\
        Connection: keep-alive\r\n\r\n";
    stream.write_all(headers.as_bytes()).await?;

    let initial = crate::UiEvent::State(ctx.mixer.snapshot().await);
    if let Ok(json) = serde_json::to_string(&initial) {
        let frame = format!("data: {}\n\n", json);
        stream.write_all(frame.as_bytes()).await?;
    }

    let mut rx = ctx.events_tx.subscribe();
    let mut ping = tokio::time::interval(Duration::from_secs(15));

    loop {
        tokio::select! {
            ev = rx.recv() => {
                match ev {
                    Ok(event) => {
                        if let Ok(json) = serde_json::to_string(&event) {
                            let frame = format!("data: {}\n\n", json);
                            if stream.write_all(frame.as_bytes()).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            _ = ping.tick() => {
                if stream.write_all(b": heartbeat\n\n").await.is_err() {
                    break;
                }
            }
        }
    }
    Ok(())
}

#[derive(Deserialize)]
struct ConnectReq {
    ip: String,
    #[serde(default = "default_auto")]
    transport: String,
    #[serde(default)]
    aes50_ip: String,
    #[serde(default)]
    usb_port: String,
}
fn default_auto() -> String {
    "auto".to_string()
}

async fn handle_connect(stream: &mut TcpStream, ctx: &AppContext, body: &[u8]) -> Result<()> {
    let req: ConnectReq = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(e) => return write_json_error(stream, 400, &e.to_string()).await,
    };
    let target_ip = if req.ip.trim().eq_ignore_ascii_case("auto") {
        match crate::mixer::discover_first_ip().await {
            Ok(Some(ip)) => ip,
            _ => "127.0.0.1:10023".to_string(),
        }
    } else {
        req.ip
    };
    match ctx
        .mixer
        .connect(&target_ip, &req.transport, &req.aes50_ip, &req.usb_port)
        .await
    {
        Ok(snap) => write_json(stream, 200, &snap).await,
        Err(e) => write_json_error(stream, 502, &e.to_string()).await,
    }
}

#[derive(Deserialize)]
struct FaderReq {
    kind: String,
    index: u8,
    value: f32,
}

async fn handle_fader(stream: &mut TcpStream, ctx: &AppContext, body: &[u8]) -> Result<()> {
    let req: FaderReq = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(e) => return write_json_error(stream, 400, &e.to_string()).await,
    };
    match ctx.mixer.set_fader(&req.kind, req.index, req.value).await {
        Ok(()) => write_json(stream, 200, &serde_json::json!({ "ok": true })).await,
        Err(e) => write_json_error(stream, 400, &e.to_string()).await,
    }
}

#[derive(Deserialize)]
struct MuteReq {
    kind: String,
    index: u8,
    on: bool,
}

async fn handle_mute(stream: &mut TcpStream, ctx: &AppContext, body: &[u8]) -> Result<()> {
    let req: MuteReq = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(e) => return write_json_error(stream, 400, &e.to_string()).await,
    };
    match ctx.mixer.set_mute(&req.kind, req.index, req.on).await {
        Ok(()) => write_json(stream, 200, &serde_json::json!({ "ok": true })).await,
        Err(e) => write_json_error(stream, 400, &e.to_string()).await,
    }
}

#[derive(Deserialize)]
struct PanReq {
    index: u8,
    pan: f32,
}

async fn handle_pan(stream: &mut TcpStream, ctx: &AppContext, body: &[u8]) -> Result<()> {
    let req: PanReq = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(e) => return write_json_error(stream, 400, &e.to_string()).await,
    };
    match ctx.mixer.set_pan(req.index, req.pan).await {
        Ok(()) => write_json(stream, 200, &serde_json::json!({ "ok": true })).await,
        Err(e) => write_json_error(stream, 400, &e.to_string()).await,
    }
}

#[derive(Deserialize)]
struct ChannelConfigReq {
    #[serde(default = "default_ch_kind")]
    kind: String,
    index: u8,
    name: Option<String>,
    color: Option<i32>,
}
fn default_ch_kind() -> String {
    "ch".to_string()
}

async fn handle_channel_config(
    stream: &mut TcpStream,
    ctx: &AppContext,
    body: &[u8],
) -> Result<()> {
    let req: ChannelConfigReq = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(e) => return write_json_error(stream, 400, &e.to_string()).await,
    };
    match ctx
        .mixer
        .set_strip_config(&req.kind, req.index, req.name.as_deref(), req.color)
        .await
    {
        Ok(()) => write_json(stream, 200, &serde_json::json!({ "ok": true })).await,
        Err(e) => write_json_error(stream, 400, &e.to_string()).await,
    }
}

#[derive(Deserialize)]
struct RoutingReq {
    path: String,
    value: i32,
}

async fn handle_routing(stream: &mut TcpStream, ctx: &AppContext, body: &[u8]) -> Result<()> {
    let req: RoutingReq = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(e) => return write_json_error(stream, 400, &e.to_string()).await,
    };
    match ctx.mixer.set_routing(&req.path, req.value).await {
        Ok(()) => write_json(stream, 200, &serde_json::json!({ "ok": true })).await,
        Err(e) => write_json_error(stream, 400, &e.to_string()).await,
    }
}

#[derive(Deserialize)]
struct SnapshotPathReq {
    #[serde(default = "default_snapshot_path")]
    path: String,
}
fn default_snapshot_path() -> String {
    "x32_pre_vsc_backup.json".to_string()
}

async fn handle_snapshot_save(
    stream: &mut TcpStream,
    ctx: &AppContext,
    body: &[u8],
) -> Result<()> {
    let req: SnapshotPathReq = if body.is_empty() {
        SnapshotPathReq {
            path: default_snapshot_path(),
        }
    } else {
        match serde_json::from_slice(body) {
            Ok(v) => v,
            Err(e) => return write_json_error(stream, 400, &e.to_string()).await,
        }
    };
    match ctx.mixer.save_snapshot_backup(Path::new(&req.path)).await {
        Ok(count) => {
            write_json(
                stream,
                200,
                &serde_json::json!({ "ok": true, "path": req.path, "entries": count }),
            )
            .await
        }
        Err(e) => write_json_error(stream, 500, &e.to_string()).await,
    }
}

async fn handle_snapshot_restore(
    stream: &mut TcpStream,
    ctx: &AppContext,
    body: &[u8],
) -> Result<()> {
    let req: SnapshotPathReq = if body.is_empty() {
        SnapshotPathReq {
            path: default_snapshot_path(),
        }
    } else {
        match serde_json::from_slice(body) {
            Ok(v) => v,
            Err(e) => return write_json_error(stream, 400, &e.to_string()).await,
        }
    };
    match ctx.mixer.restore_snapshot_backup(Path::new(&req.path)).await {
        Ok(count) => {
            write_json(
                stream,
                200,
                &serde_json::json!({ "ok": true, "path": req.path, "restored": count }),
            )
            .await
        }
        Err(e) => write_json_error(stream, 400, &e.to_string()).await,
    }
}

#[derive(Deserialize)]
struct RawOscArg {
    #[serde(rename = "type")]
    arg_type: String,
    value: serde_json::Value,
}

#[derive(Deserialize)]
struct RawOscReq {
    path: String,
    #[serde(default)]
    args: Vec<RawOscArg>,
}

async fn handle_raw_osc(stream: &mut TcpStream, ctx: &AppContext, body: &[u8]) -> Result<()> {
    let req: RawOscReq = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(e) => return write_json_error(stream, 400, &e.to_string()).await,
    };

    let mut osc_args = Vec::with_capacity(req.args.len());
    for a in req.args {
        match a.arg_type.as_str() {
            "i" | "int" => {
                let v = a
                    .value
                    .as_i64()
                    .or_else(|| a.value.as_str().and_then(|s| s.parse().ok()))
                    .unwrap_or(0) as i32;
                osc_args.push(OscArg::Int(v));
            }
            "f" | "float" => {
                let v = a
                    .value
                    .as_f64()
                    .or_else(|| a.value.as_str().and_then(|s| s.parse().ok()))
                    .unwrap_or(0.0) as f32;
                osc_args.push(OscArg::Float(v));
            }
            _ => {
                let s = a
                    .value
                    .as_str()
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| a.value.to_string());
                osc_args.push(OscArg::String(s));
            }
        }
    }

    match ctx.mixer.send_osc(&req.path, osc_args).await {
        Ok(()) => write_json(stream, 200, &serde_json::json!({ "ok": true })).await,
        Err(e) => write_json_error(stream, 400, &e.to_string()).await,
    }
}

#[derive(Deserialize)]
struct RunToolReq {
    tool_id: String,
    #[serde(default)]
    params: HashMap<String, serde_json::Value>,
}

async fn handle_tool_run(stream: &mut TcpStream, ctx: &AppContext, body: &[u8]) -> Result<()> {
    let req: RunToolReq = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(e) => return write_json_error(stream, 400, &e.to_string()).await,
    };
    match ctx.runner.start_tool(&req.tool_id, &req.params).await {
        Ok(job) => write_json(stream, 200, &job).await,
        Err(e) => write_json_error(stream, 400, &e.to_string()).await,
    }
}

#[derive(Deserialize)]
struct StopToolReq {
    job_id: String,
}

async fn handle_tool_stop(stream: &mut TcpStream, ctx: &AppContext, body: &[u8]) -> Result<()> {
    let req: StopToolReq = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(e) => return write_json_error(stream, 400, &e.to_string()).await,
    };
    match ctx.runner.stop_job(&req.job_id).await {
        Ok(job) => write_json(stream, 200, &job).await,
        Err(e) => write_json_error(stream, 404, &e.to_string()).await,
    }
}

#[derive(Deserialize, Default)]
struct VscCaptureReq {
    #[serde(default)]
    refresh: bool,
    #[serde(default)]
    include_aux_card: bool,
    #[serde(default = "default_true_flag")]
    preserve_card_uout: bool,
}

fn default_true_flag() -> bool {
    true
}

#[derive(Serialize)]
struct VscCaptureResp {
    state: crate::mixer::MixerSnapshotState,
    session_options: VscSessionOptions,
    routing_plan: VscRoutingPlan,
}

async fn handle_vsc_capture(
    stream: &mut TcpStream,
    ctx: &AppContext,
    body: &[u8],
) -> Result<()> {
    let req: VscCaptureReq = if body.is_empty() {
        VscCaptureReq {
            refresh: false,
            include_aux_card: false,
            preserve_card_uout: true,
        }
    } else {
        serde_json::from_slice(body).unwrap_or_default()
    };

    let state = if req.refresh {
        match ctx.mixer.refresh_full_state().await {
            Ok(s) => s,
            Err(_) => ctx.mixer.snapshot().await,
        }
    } else {
        ctx.mixer.snapshot().await
    };

    let session_options = VscSessionOptions::from_mixer_state(&state);
    let routing_plan = compute_vsc_routing_plan_with_options(
        &state.routing,
        req.include_aux_card,
        req.preserve_card_uout,
    );
    write_json(
        stream,
        200,
        &VscCaptureResp {
            state,
            session_options,
            routing_plan,
        },
    )
    .await
}

#[derive(Deserialize, Default)]
struct VscImportSceneReq {
    #[serde(default)]
    scene_content: Option<String>,
    #[serde(default)]
    scene_path: Option<String>,
    #[serde(default)]
    include_aux_card: bool,
    #[serde(default = "default_true_flag")]
    preserve_card_uout: bool,
}

async fn handle_vsc_import_scene(stream: &mut TcpStream, body: &[u8]) -> Result<()> {
    let req: VscImportSceneReq = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(e) => return write_json_error(stream, 400, &e.to_string()).await,
    };

    let state = if let Some(ref content) = req.scene_content {
        if content.trim().is_empty() {
            return write_json_error(stream, 400, "Empty scene_content").await;
        }
        match parse_x32_scene_str(content) {
            Ok(s) => s,
            Err(e) => return write_json_error(stream, 400, &e.to_string()).await,
        }
    } else if let Some(ref path) = req.scene_path {
        match parse_x32_scene_file(Path::new(path.trim())) {
            Ok(s) => s,
            Err(e) => return write_json_error(stream, 400, &e.to_string()).await,
        }
    } else {
        return write_json_error(stream, 400, "Provide scene_content or scene_path").await;
    };

    let session_options = VscSessionOptions::from_mixer_state(&state);
    let routing_plan = compute_vsc_routing_plan_with_options(
        &state.routing,
        req.include_aux_card,
        req.preserve_card_uout,
    );
    write_json(
        stream,
        200,
        &VscCaptureResp {
            state,
            session_options,
            routing_plan,
        },
    )
    .await
}

#[derive(Deserialize, Default)]
struct VscApplyRoutingReq {
    #[serde(default)]
    include_aux_card: bool,
    #[serde(default = "default_true_flag")]
    preserve_card_uout: bool,
    #[serde(default)]
    backup_path: Option<String>,
    #[serde(default)]
    switch_to_play: bool,
}

async fn handle_vsc_apply_routing(
    stream: &mut TcpStream,
    ctx: &AppContext,
    body: &[u8],
) -> Result<()> {
    let req: VscApplyRoutingReq = if body.is_empty() {
        VscApplyRoutingReq {
            include_aux_card: false,
            preserve_card_uout: true,
            backup_path: None,
            switch_to_play: false,
        }
    } else {
        match serde_json::from_slice(body) {
            Ok(v) => v,
            Err(e) => return write_json_error(stream, 400, &e.to_string()).await,
        }
    };

    let current = ctx.mixer.snapshot().await;
    let plan = compute_vsc_routing_plan_with_options(
        &current.routing,
        req.include_aux_card,
        req.preserve_card_uout,
    );
    let backup = req
        .backup_path
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(Path::new);

    match apply_vsc_routing(&ctx.mixer, &plan, backup).await {
        Ok(applied) => {
            if req.switch_to_play {
                let _ = set_vsc_mode(&ctx.mixer, true, Some(&ctx.events_tx)).await;
            }
            let updated = ctx.mixer.snapshot().await;
            write_json(
                stream,
                200,
                &serde_json::json!({
                    "ok": true,
                    "applied_changes": applied,
                    "backup_path": req.backup_path,
                    "routing": updated.routing,
                    "plan": plan,
                }),
            )
            .await
        }
        Err(e) => write_json_error(stream, 400, &e.to_string()).await,
    }
}

#[derive(Deserialize)]
struct VscModeReq {
    mode: String,
}

async fn handle_vsc_mode(stream: &mut TcpStream, ctx: &AppContext, body: &[u8]) -> Result<()> {
    let req: VscModeReq = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(e) => return write_json_error(stream, 400, &e.to_string()).await,
    };
    let play_mode = match req.mode.to_ascii_lowercase().as_str() {
        "play" | "vsc" | "1" | "true" => true,
        "rec" | "live" | "0" | "false" => false,
        other => return write_json_error(stream, 400, &format!("Invalid mode '{}'", other)).await,
    };
    match set_vsc_mode(&ctx.mixer, play_mode, Some(&ctx.events_tx)).await {
        Ok(()) => {
            let state = ctx.mixer.snapshot().await;
            write_json(
                stream,
                200,
                &serde_json::json!({
                    "ok": true,
                    "play_mode": play_mode,
                    "routswitch": state.routing.routswitch
                }),
            )
            .await
        }
        Err(e) => write_json_error(stream, 400, &e.to_string()).await,
    }
}

#[derive(Deserialize)]
struct VscGenerateRppReq {
    #[serde(default)]
    session_options: VscSessionOptions,
    #[serde(default)]
    ini_options: Option<ReaperIniOptions>,
    #[serde(default)]
    template_content: Option<String>,
    #[serde(default)]
    template_path: Option<String>,
    #[serde(default)]
    output_rpp_path: Option<String>,
    #[serde(default)]
    output_ini_path: Option<String>,
}

async fn handle_vsc_generate_rpp(
    stream: &mut TcpStream,
    ctx: &AppContext,
    body: &[u8],
) -> Result<()> {
    let mut req: VscGenerateRppReq = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(e) => return write_json_error(stream, 400, &e.to_string()).await,
    };

    let snapshot = ctx.mixer.snapshot().await;
    if req.session_options.selected_channels.is_empty() {
        req.session_options = VscSessionOptions::from_mixer_state(&snapshot);
    }

    let template_text = if let Some(ref content) = req.template_content {
        if !content.trim().is_empty() {
            Some(content.clone())
        } else {
            None
        }
    } else if let Some(ref path) = req.template_path {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            match read_bounded_file(Path::new(trimmed), MAX_RPP_FILE_BYTES) {
                Ok(text) => Some(text),
                Err(e) => {
                    return write_json_error(
                        stream,
                        400,
                        &format!("Failed to read template '{}': {}", trimmed, e),
                    )
                    .await
                }
            }
        } else {
            None
        }
    } else {
        None
    };

    let rpp_content = if let Some(tpl) = template_text {
        match adapt_rpp(&tpl, &req.session_options) {
            Ok(s) => s,
            Err(e) => return write_json_error(stream, 400, &e.to_string()).await,
        }
    } else {
        generate_rpp(&req.session_options)
    };

    let mut ini_opts = req.ini_options.unwrap_or_default();
    if ini_opts.x32_ip.trim().is_empty() && !snapshot.ip.trim().is_empty() {
        ini_opts.x32_ip = snapshot.ip.clone();
    }
    let resolved_tracks = resolve_channel_tracks(&req.session_options);
    if !resolved_tracks.is_empty() {
        ini_opts.trk_min = 1;
        ini_opts.trk_max = resolved_tracks.len() as i32;
    }
    let ini_content = generate_x32_reaper_ini(&ini_opts);

    let mut saved_rpp_path = None;
    if let Some(ref out_path) = req.output_rpp_path {
        let p = out_path.trim();
        if !p.is_empty() {
            if let Some(parent) = Path::new(p).parent() {
                if !parent.as_os_str().is_empty() {
                    let _ = std::fs::create_dir_all(parent);
                }
            }
            if let Err(e) = std::fs::write(p, &rpp_content) {
                return write_json_error(stream, 500, &format!("Failed writing .rpp: {}", e)).await;
            }
            saved_rpp_path = Some(p.to_string());
        }
    }

    let mut saved_ini_path = None;
    if let Some(ref ini_path) = req.output_ini_path {
        let p = ini_path.trim();
        if !p.is_empty() {
            if let Some(parent) = Path::new(p).parent() {
                if !parent.as_os_str().is_empty() {
                    let _ = std::fs::create_dir_all(parent);
                }
            }
            if let Err(e) = std::fs::write(p, &ini_content) {
                return write_json_error(stream, 500, &format!("Failed writing .ini: {}", e)).await;
            }
            saved_ini_path = Some(p.to_string());
        }
    }

    write_json(
        stream,
        200,
        &serde_json::json!({
            "ok": true,
            "rpp_content": rpp_content,
            "ini_content": ini_content,
            "saved_rpp_path": saved_rpp_path,
            "saved_ini_path": saved_ini_path,
            "track_count": resolved_tracks.len(),
        }),
    )
    .await
}

#[derive(Deserialize)]
struct ReaperOscReq {
    reaper_ip: String,
    #[serde(default = "default_reaper_port")]
    reaper_port: u16,
    #[serde(default)]
    action: Option<String>,
    #[serde(default)]
    session_options: Option<VscSessionOptions>,
    #[serde(default)]
    arm_all: Option<bool>,
}
fn default_reaper_port() -> u16 {
    8000
}

async fn handle_vsc_reaper_osc(stream: &mut TcpStream, body: &[u8]) -> Result<()> {
    let req: ReaperOscReq = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(e) => return write_json_error(stream, 400, &e.to_string()).await,
    };

    if let Some(ref action) = req.action {
        match send_reaper_transport(&req.reaper_ip, req.reaper_port, action).await {
            Ok(()) => {
                return write_json(
                    stream,
                    200,
                    &serde_json::json!({ "ok": true, "action": action }),
                )
                .await
            }
            Err(e) => return write_json_error(stream, 400, &e.to_string()).await,
        }
    }

    let opts = req.session_options.unwrap_or_default();
    let tracks = resolve_channel_tracks(&opts);
    match push_reaper_osc(&req.reaper_ip, req.reaper_port, &tracks, req.arm_all).await {
        Ok(sent) => {
            write_json(
                stream,
                200,
                &serde_json::json!({ "ok": true, "messages_sent": sent, "tracks": tracks.len() }),
            )
            .await
        }
        Err(e) => write_json_error(stream, 400, &e.to_string()).await,
    }
}

async fn handle_studio_profile(
    stream: &mut TcpStream,
    ctx: &AppContext,
    body: &[u8],
) -> Result<()> {
    let profile: StudioProfile = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(e) => return write_json_error(stream, 400, &e.to_string()).await,
    };
    match ctx.studio.update_profile(profile).await {
        Ok(status) => write_json(stream, 200, &status).await,
        Err(e) => write_json_error(stream, 400, &e.to_string()).await,
    }
}

#[derive(Deserialize)]
struct StudioModeReq {
    mode: String,
}

async fn handle_studio_mode(stream: &mut TcpStream, ctx: &AppContext, body: &[u8]) -> Result<()> {
    let req: StudioModeReq = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(e) => return write_json_error(stream, 400, &e.to_string()).await,
    };
    let mode = match req.mode.to_ascii_lowercase().as_str() {
        "live" | "rec" => OperatingMode::Live,
        "daw" | "play" | "vsc" => OperatingMode::Daw,
        other => {
            return write_json_error(stream, 400, &format!("Invalid studio mode '{}'", other))
                .await
        }
    };
    match ctx.studio.switch_mode(mode).await {
        Ok(status) => write_json(stream, 200, &status).await,
        Err(e) => write_json_error(stream, 400, &e.to_string()).await,
    }
}

#[derive(Deserialize, Default)]
struct StudioRecordStartReq {
    #[serde(default)]
    label: Option<String>,
}

async fn handle_studio_record_start(
    stream: &mut TcpStream,
    ctx: &AppContext,
    body: &[u8],
) -> Result<()> {
    let req: StudioRecordStartReq = if body.is_empty() {
        StudioRecordStartReq::default()
    } else {
        serde_json::from_slice(body).unwrap_or_default()
    };
    match ctx.studio.start_synced_recording(req.label.as_deref()).await {
        Ok(status) => write_json(stream, 200, &status).await,
        Err(e) => write_json_error(stream, 400, &e.to_string()).await,
    }
}

#[derive(Deserialize, Default)]
struct StudioPlayReq {
    #[serde(default)]
    file: Option<String>,
}

async fn handle_studio_play(stream: &mut TcpStream, ctx: &AppContext, body: &[u8]) -> Result<()> {
    let req: StudioPlayReq = if body.is_empty() {
        StudioPlayReq::default()
    } else {
        serde_json::from_slice(body).unwrap_or_default()
    };
    match ctx.studio.start_synced_play(req.file.as_deref()).await {
        Ok(status) => write_json(stream, 200, &status).await,
        Err(e) => write_json_error(stream, 400, &e.to_string()).await,
    }
}

#[derive(Deserialize, Default)]
struct StudioMarkerReq {
    #[serde(default)]
    label: Option<String>,
}

async fn handle_studio_marker(
    stream: &mut TcpStream,
    ctx: &AppContext,
    body: &[u8],
) -> Result<()> {
    let req: StudioMarkerReq = if body.is_empty() {
        StudioMarkerReq::default()
    } else {
        serde_json::from_slice(body).unwrap_or_default()
    };
    match ctx.studio.drop_marker(req.label.as_deref()).await {
        Ok(status) => write_json(stream, 200, &status).await,
        Err(e) => write_json_error(stream, 400, &e.to_string()).await,
    }
}

#[derive(Deserialize)]
struct StudioButtonReq {
    offset: u8,
}

async fn handle_studio_button(
    stream: &mut TcpStream,
    ctx: &AppContext,
    body: &[u8],
) -> Result<()> {
    let req: StudioButtonReq = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(e) => return write_json_error(stream, 400, &e.to_string()).await,
    };
    ctx.studio.dispatch_console_button(req.offset).await;
    let status = ctx.studio.status().await;
    write_json(stream, 200, &status).await
}

fn read_bounded_file(path: &Path, max_bytes: u64) -> Result<String> {
    let file = std::fs::File::open(path)?;
    let meta_len = file.metadata()?.len();
    if meta_len > max_bytes {
        anyhow::bail!("File exceeds {} byte limit", max_bytes);
    }
    let mut buf = String::new();
    file.take(max_bytes + 1).read_to_string(&mut buf)?;
    if buf.len() as u64 > max_bytes {
        anyhow::bail!("File exceeds {} byte limit", max_bytes);
    }
    Ok(buf)
}

async fn write_json<T: Serialize>(stream: &mut TcpStream, status: u16, data: &T) -> Result<()> {
    let body = serde_json::to_vec(data)?;
    write_response(stream, status, "application/json; charset=utf-8", &body).await
}

async fn write_json_error(stream: &mut TcpStream, status: u16, msg: &str) -> Result<()> {
    write_json(stream, status, &serde_json::json!({ "error": msg })).await
}

async fn write_response(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &[u8],
) -> Result<()> {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        _ => "OK",
    };
    let header = format!(
        "HTTP/1.1 {} {}\r\n\
         Content-Type: {}\r\n\
         Content-Length: {}\r\n\
         Cache-Control: no-store\r\n\
         Connection: close\r\n\r\n",
        status,
        reason,
        content_type,
        body.len()
    );
    stream.write_all(header.as_bytes()).await?;
    stream.write_all(body).await?;
    stream.flush().await?;
    Ok(())
}
