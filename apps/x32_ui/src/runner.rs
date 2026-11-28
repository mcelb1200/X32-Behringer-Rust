//! Child-process runner for executing workspace X32 tools via `x32-cli` and streaming logs.

use crate::tools_catalog::{build_cli_args, find_tool, ToolMode};
use crate::UiEvent;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{broadcast, Mutex};

const MAX_JOB_LOGS: usize = 200;

/// Public status summary for a launched tool process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSummary {
    /// Unique job identifier (e.g. `"job-1"`).
    pub job_id: String,
    /// Tool subcommand ID (e.g. `"x32-reaper"`).
    pub tool_id: String,
    /// Full CLI command line executed.
    pub command_line: String,
    /// Current status (`"running"`, `"exited"`, `"stopped"`, `"failed"`, `"tui_hint"`).
    pub status: String,
    /// Exit code if exited.
    pub exit_code: Option<i32>,
    /// Recent stdout/stderr lines.
    pub logs: Vec<String>,
}

struct ManagedJob {
    summary: JobSummary,
    child: Option<Child>,
}

/// Manages child processes for tools launched from the consolidated Web UI.
pub struct ProcessManager {
    jobs: Arc<Mutex<HashMap<String, ManagedJob>>>,
    seq: AtomicU64,
    events_tx: broadcast::Sender<UiEvent>,
}

impl ProcessManager {
    /// Creates a new [`ProcessManager`].
    pub fn new(events_tx: broadcast::Sender<UiEvent>) -> Self {
        Self {
            jobs: Arc::new(Mutex::new(HashMap::new())),
            seq: AtomicU64::new(1),
            events_tx,
        }
    }

    /// Resolves the `x32-cli` binary path on disk or via `PATH`.
    pub fn resolve_x32_cli_path() -> PathBuf {
        let exe_name = if cfg!(windows) {
            "x32-cli.exe"
        } else {
            "x32-cli"
        };
        if let Ok(curr) = std::env::current_exe() {
            if let Some(dir) = curr.parent() {
                let sibling = dir.join(exe_name);
                if sibling.exists() {
                    return sibling;
                }
            }
        }
        for candidate in [
            format!("target/debug/{}", exe_name),
            format!("target/release/{}", exe_name),
        ] {
            let p = PathBuf::from(&candidate);
            if p.exists() {
                return p;
            }
        }
        PathBuf::from(exe_name)
    }

    /// Launches a cataloged tool with the provided parameter map.
    pub async fn start_tool(
        &self,
        tool_id: &str,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<JobSummary> {
        let tool = find_tool(tool_id).ok_or_else(|| anyhow::anyhow!("Unknown tool: {}", tool_id))?;
        let cli_args = build_cli_args(&tool, params)?;
        let bin = Self::resolve_x32_cli_path();
        let command_line = format!("{} {}", bin.display(), cli_args.join(" "));
        let id = format!("job-{}", self.seq.fetch_add(1, Ordering::Relaxed));

        if tool.mode == ToolMode::InteractiveTui {
            let summary = JobSummary {
                job_id: id.clone(),
                tool_id: tool_id.to_string(),
                command_line: command_line.clone(),
                status: "tui_hint".to_string(),
                exit_code: None,
                logs: vec![format!(
                    "'{}' is an interactive terminal UI (TUI). Run it in a terminal: {}",
                    tool.name, command_line
                )],
            };
            self.jobs.lock().await.insert(
                id,
                ManagedJob {
                    summary: summary.clone(),
                    child: None,
                },
            );
            let _ = self.events_tx.send(UiEvent::JobStatus(summary.clone()));
            return Ok(summary);
        }

        let mut cmd = Command::new(&bin);
        cmd.args(&cli_args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        let mut child = cmd
            .spawn()
            .map_err(|e| anyhow::anyhow!("Failed to launch '{}': {}", command_line, e))?;

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        let summary = JobSummary {
            job_id: id.clone(),
            tool_id: tool_id.to_string(),
            command_line,
            status: "running".to_string(),
            exit_code: None,
            logs: Vec::new(),
        };

        self.jobs.lock().await.insert(
            id.clone(),
            ManagedJob {
                summary: summary.clone(),
                child: Some(child),
            },
        );
        let _ = self.events_tx.send(UiEvent::JobStatus(summary.clone()));

        if let Some(out) = stdout {
            spawn_log_reader(id.clone(), "stdout", out, self.jobs.clone(), self.events_tx.clone());
        }
        if let Some(err) = stderr {
            spawn_log_reader(id.clone(), "stderr", err, self.jobs.clone(), self.events_tx.clone());
        }

        // Background exit watcher
        let jobs_ref = self.jobs.clone();
        let events_ref = self.events_tx.clone();
        let watch_id = id.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(250)).await;
                let mut guard = jobs_ref.lock().await;
                let Some(job) = guard.get_mut(&watch_id) else {
                    break;
                };
                if let Some(ref mut child) = job.child {
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            job.summary.exit_code = status.code();
                            if job.summary.status == "running" {
                                job.summary.status = if status.success() {
                                    "exited".to_string()
                                } else {
                                    "failed".to_string()
                                };
                            }
                            job.child = None;
                            let s = job.summary.clone();
                            drop(guard);
                            let _ = events_ref.send(UiEvent::JobStatus(s));
                            break;
                        }
                        Ok(None) => continue,
                        Err(_) => break,
                    }
                } else {
                    break;
                }
            }
        });

        Ok(summary)
    }

    /// Stops a running job by ID.
    pub async fn stop_job(&self, job_id: &str) -> Result<JobSummary> {
        let mut guard = self.jobs.lock().await;
        let job = guard
            .get_mut(job_id)
            .ok_or_else(|| anyhow::anyhow!("Job '{}' not found", job_id))?;
        if let Some(mut child) = job.child.take() {
            let _ = child.kill().await;
        }
        job.summary.status = "stopped".to_string();
        let summary = job.summary.clone();
        drop(guard);
        let _ = self.events_tx.send(UiEvent::JobStatus(summary.clone()));
        Ok(summary)
    }

    /// Lists all tracked jobs sorted newest-first.
    pub async fn list_jobs(&self) -> Vec<JobSummary> {
        let guard = self.jobs.lock().await;
        let mut out: Vec<JobSummary> = guard.values().map(|j| j.summary.clone()).collect();
        out.sort_by(|a, b| b.job_id.cmp(&a.job_id));
        out
    }

    /// Terminates all running child processes during shutdown.
    pub async fn stop_all(&self) {
        let mut guard = self.jobs.lock().await;
        for (_, job) in guard.iter_mut() {
            if let Some(mut child) = job.child.take() {
                let _ = child.kill().await;
            }
            if job.summary.status == "running" {
                job.summary.status = "stopped".to_string();
            }
        }
    }
}

fn spawn_log_reader<R: tokio::io::AsyncRead + Unpin + Send + 'static>(
    job_id: String,
    stream: &'static str,
    reader: R,
    jobs: Arc<Mutex<HashMap<String, ManagedJob>>>,
    events_tx: broadcast::Sender<UiEvent>,
) {
    tokio::spawn(async move {
        let mut lines = BufReader::new(reader).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            {
                let mut guard = jobs.lock().await;
                if let Some(job) = guard.get_mut(&job_id) {
                    let mut buf: VecDeque<String> = job.summary.logs.drain(..).collect();
                    if buf.len() >= MAX_JOB_LOGS {
                        buf.pop_front();
                    }
                    buf.push_back(format!("[{}] {}", stream, line));
                    job.summary.logs = buf.into_iter().collect();
                }
            }
            let _ = events_tx.send(UiEvent::JobLog {
                job_id: job_id.clone(),
                stream: stream.to_string(),
                line,
            });
        }
    });
}
