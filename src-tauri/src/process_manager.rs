use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::fs::OpenOptions;
use tokio::io::{self, AsyncRead, AsyncWrite};
use tokio::process::{Child, Command};
use tokio::task::JoinHandle;
use tokio::time::{sleep, timeout, Duration, Instant};

use crate::error::{AppError, AppResult};
use crate::log_store;

#[cfg(target_os = "windows")]
const WINDOWS_CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ProfileProcessState {
    Stopped,
    Starting,
    Running,
    Reloading,
    Degraded,
    Failed,
}

struct ManagedProcess {
    child: Child,
    pid: Option<u32>,
    started_at: DateTime<Utc>,
    state: ProfileProcessState,
    stdout_drain: Option<JoinHandle<()>>,
    stderr_drain: Option<JoinHandle<()>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessSnapshot {
    pub state: ProfileProcessState,
    pub pid: Option<u32>,
    pub started_at: Option<DateTime<Utc>>,
}

#[derive(Default)]
pub struct ProcessRegistry {
    processes: HashMap<String, ManagedProcess>,
}

impl ProcessRegistry {
    pub async fn start(
        &mut self,
        profile_id: &str,
        frpc_path: &Path,
        config_path: &Path,
        working_dir: &Path,
    ) -> AppResult<()> {
        if self.processes.contains_key(profile_id) {
            return Err(AppError::ProcessAlreadyRunning(profile_id.to_string()));
        }

        let log_path = current_log_path(working_dir);
        prepare_log_file(&log_path)?;
        let log_target = Arc::new(LogTarget::new(log_path));

        let mut command = Command::new(frpc_path);
        command
            .arg("-c")
            .arg(config_path)
            .current_dir(working_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        apply_platform_process_options(&mut command);

        let mut child = command.spawn()?;
        let stdout_drain = child
            .stdout
            .take()
            .map(|stdout| spawn_pipe_drain(stdout, Arc::clone(&log_target)));
        let stderr_drain = child
            .stderr
            .take()
            .map(|stderr| spawn_pipe_drain(stderr, Arc::clone(&log_target)));
        let pid = child.id();

        match detect_immediate_exit(&mut child).await {
            Ok(Some(status)) => {
                abort_pipe_drain(stdout_drain);
                abort_pipe_drain(stderr_drain);
                return Err(AppError::Runtime(format!(
                    "process {profile_id} exited immediately with status {status}"
                )));
            }
            Ok(None) => {}
            Err(err) => {
                abort_pipe_drain(stdout_drain);
                abort_pipe_drain(stderr_drain);
                return Err(AppError::Io(err));
            }
        }

        self.processes.insert(
            profile_id.to_string(),
            ManagedProcess {
                child,
                pid,
                started_at: Utc::now(),
                state: ProfileProcessState::Running,
                stdout_drain,
                stderr_drain,
            },
        );
        Ok(())
    }

    pub async fn stop(&mut self, profile_id: &str) -> AppResult<()> {
        {
            let managed = self
                .processes
                .get_mut(profile_id)
                .ok_or_else(|| AppError::ProcessNotRunning(profile_id.to_string()))?;

            if managed.child.try_wait()?.is_none() {
                managed.child.start_kill()?;
                match timeout(Duration::from_secs(3), managed.child.wait()).await {
                    Ok(Ok(_status)) => {}
                    Ok(Err(err)) => return Err(AppError::Io(err)),
                    Err(_) => {
                        return Err(AppError::Runtime(format!(
                            "timed out stopping process {profile_id}"
                        )));
                    }
                }
            }
        }

        if let Some(managed) = self.processes.remove(profile_id) {
            abort_pipe_drain(managed.stdout_drain);
            abort_pipe_drain(managed.stderr_drain);
        }
        Ok(())
    }

    pub async fn stop_all(&mut self) -> AppResult<()> {
        let profile_ids = self.processes.keys().cloned().collect::<Vec<_>>();
        let mut first_error = None;

        for profile_id in profile_ids {
            if let Err(err) = self.stop(&profile_id).await {
                if first_error.is_none() {
                    first_error = Some(err);
                }
            }
        }

        match first_error {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }

    pub async fn restart(
        &mut self,
        profile_id: &str,
        frpc_path: &Path,
        config_path: &Path,
        working_dir: &Path,
    ) -> AppResult<()> {
        self.stop(profile_id).await?;
        self.start(profile_id, frpc_path, config_path, working_dir)
            .await
    }

    pub fn state(&self, profile_id: &str) -> ProfileProcessState {
        self.processes
            .get(profile_id)
            .map(|managed| managed.state)
            .unwrap_or(ProfileProcessState::Stopped)
    }

    pub fn refresh_state(&mut self, profile_id: &str) -> AppResult<ProfileProcessState> {
        Ok(self.refresh_snapshot(profile_id)?.state)
    }

    pub fn refresh_snapshot(&mut self, profile_id: &str) -> AppResult<ProcessSnapshot> {
        let Some(managed) = self.processes.get_mut(profile_id) else {
            return Ok(ProcessSnapshot {
                state: ProfileProcessState::Stopped,
                pid: None,
                started_at: None,
            });
        };

        if managed.child.try_wait()?.is_none() {
            return Ok(ProcessSnapshot {
                state: managed.state,
                pid: managed.pid,
                started_at: Some(managed.started_at),
            });
        }

        if let Some(managed) = self.processes.remove(profile_id) {
            abort_pipe_drain(managed.stdout_drain);
            abort_pipe_drain(managed.stderr_drain);
        }
        Ok(ProcessSnapshot {
            state: ProfileProcessState::Failed,
            pid: None,
            started_at: None,
        })
    }

    pub fn running_count(&self) -> usize {
        self.processes.len()
    }
}

fn current_log_path(working_dir: &Path) -> PathBuf {
    log_store::current_log_path(&working_dir.join("logs"))
}

/// Start a fresh live log for this run, keeping the previous one as an archive
/// so a restart does not silently discard history.
fn prepare_log_file(log_path: &Path) -> std::io::Result<()> {
    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let has_content = std::fs::metadata(log_path)
        .map(|metadata| metadata.len() > 0)
        .unwrap_or(false);
    if has_content {
        let _ = log_store::archive_current(log_path, log_store::DEFAULT_KEEP_ARCHIVES);
    }

    std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_path)?;
    Ok(())
}

/// Shared destination for a profile's stdout and stderr.
///
/// Both pipes append to the same file, so the rotation decision is guarded by a
/// lock to keep the two drains from archiving the file out from under each other.
struct LogTarget {
    path: PathBuf,
    max_bytes: u64,
    keep_archives: usize,
    rotation_lock: Mutex<()>,
}

impl LogTarget {
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            max_bytes: log_store::DEFAULT_MAX_LOG_BYTES,
            keep_archives: log_store::DEFAULT_KEEP_ARCHIVES,
            rotation_lock: Mutex::new(()),
        }
    }
}

/// Append-only log writer that rotates itself once it reaches the size cap.
///
/// Rotation is checked against a locally tracked size so the common path costs
/// no extra syscalls; the file is only inspected once a write would cross the
/// cap.
struct RotatingLog {
    target: Arc<LogTarget>,
    file: tokio::fs::File,
    written: u64,
}

impl RotatingLog {
    async fn open(target: Arc<LogTarget>) -> std::io::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&target.path)
            .await?;
        let written = file.metadata().await.map(|metadata| metadata.len()).unwrap_or(0);
        Ok(Self {
            target,
            file,
            written,
        })
    }

    fn rotate_if_needed(&mut self, incoming: usize) -> std::io::Result<()> {
        if self.written + incoming as u64 <= self.target.max_bytes {
            return Ok(());
        }

        let Ok(_guard) = self.target.rotation_lock.lock() else {
            return Ok(());
        };

        let rotated = log_store::rotate_if_needed(
            &self.target.path,
            self.target.max_bytes,
            self.target.keep_archives,
        )?;
        if rotated.is_some() {
            let reopened = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.target.path)?;
            self.file = tokio::fs::File::from_std(reopened);
        }

        // Both pipes share one file, so resync from disk rather than trusting
        // this writer's own byte count.
        if let Ok(metadata) = std::fs::metadata(&self.target.path) {
            self.written = metadata.len();
        }
        Ok(())
    }
}

impl AsyncWrite for RotatingLog {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        let this = self.get_mut();
        if let Err(err) = this.rotate_if_needed(buf.len()) {
            return Poll::Ready(Err(err));
        }
        match Pin::new(&mut this.file).poll_write(cx, buf) {
            Poll::Ready(Ok(written)) => {
                this.written += written as u64;
                Poll::Ready(Ok(written))
            }
            other => other,
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.get_mut().file).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.get_mut().file).poll_shutdown(cx)
    }
}

#[cfg(target_os = "windows")]
fn apply_platform_process_options(command: &mut Command) {
    command.creation_flags(WINDOWS_CREATE_NO_WINDOW);
}

#[cfg(not(target_os = "windows"))]
fn apply_platform_process_options(_command: &mut Command) {}

fn spawn_pipe_drain<R>(mut reader: R, target: Arc<LogTarget>) -> JoinHandle<()>
where
    R: AsyncRead + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        match RotatingLog::open(target).await {
            Ok(mut log_file) => {
                let _ = io::copy(&mut reader, &mut log_file).await;
            }
            Err(_) => {
                let mut sink = io::sink();
                let _ = io::copy(&mut reader, &mut sink).await;
            }
        }
    })
}

async fn detect_immediate_exit(
    child: &mut Child,
) -> std::io::Result<Option<std::process::ExitStatus>> {
    let deadline = Instant::now() + Duration::from_secs(1);
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(Some(status));
        }
        if Instant::now() >= deadline {
            return Ok(None);
        }
        sleep(Duration::from_millis(25)).await;
    }
}

fn abort_pipe_drain(handle: Option<JoinHandle<()>>) {
    if let Some(handle) = handle {
        handle.abort();
    }
}
