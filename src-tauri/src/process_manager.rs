use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::fs::OpenOptions;
use tokio::io::{self, AsyncRead};
use tokio::process::{Child, Command};
use tokio::task::JoinHandle;
use tokio::time::{sleep, timeout, Duration, Instant};

use crate::error::{AppError, AppResult};

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
            .map(|stdout| spawn_pipe_drain(stdout, log_path.clone()));
        let stderr_drain = child
            .stderr
            .take()
            .map(|stderr| spawn_pipe_drain(stderr, log_path.clone()));
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
    working_dir.join("logs").join("current.log")
}

fn prepare_log_file(log_path: &Path) -> std::io::Result<()> {
    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_path)?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn apply_platform_process_options(command: &mut Command) {
    command.creation_flags(WINDOWS_CREATE_NO_WINDOW);
}

#[cfg(not(target_os = "windows"))]
fn apply_platform_process_options(_command: &mut Command) {}

fn spawn_pipe_drain<R>(mut reader: R, log_path: PathBuf) -> JoinHandle<()>
where
    R: AsyncRead + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        match OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .await
        {
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
