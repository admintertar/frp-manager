use std::io::{Cursor, Read};
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::{AppError, AppResult};
use crate::github_release::{fetch_latest_release, GitHubRelease};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeInfo {
    pub version: String,
    pub os: String,
    pub arch: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct RuntimeManager {
    root: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeUpdateCheck {
    pub current_version: Option<String>,
    pub latest_version: String,
    pub update_available: bool,
    pub asset_name: Option<String>,
    pub installed: bool,
    pub runtime_path: Option<PathBuf>,
    pub platform: RuntimePlatform,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RuntimePlatform {
    pub os: String,
    pub arch: String,
    pub executable_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeMetadata {
    pub version: String,
    pub os: String,
    pub arch: String,
    pub path: PathBuf,
    pub asset_name: String,
    pub installed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeStatus {
    pub installed: bool,
    pub current_version: Option<String>,
    pub runtime_path: Option<PathBuf>,
    pub platform: RuntimePlatform,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReleaseCache {
    fetched_at: DateTime<Utc>,
    release: GitHubRelease,
}

#[derive(Debug, Clone)]
pub struct CachedRelease {
    pub release: GitHubRelease,
    pub source: &'static str,
    pub warning: Option<String>,
}

impl RuntimeManager {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn runtime_dir(&self) -> PathBuf {
        self.root.join("runtime")
    }

    pub fn runtime_info_for_version(&self, version: &str, os: &str, arch: &str) -> RuntimeInfo {
        RuntimeInfo {
            version: version.to_string(),
            os: os.to_string(),
            arch: arch.to_string(),
            path: self.runtime_dir().join(format!(
                "frpc-{version}-{os}-{arch}{}",
                executable_suffix(os)
            )),
        }
    }

    pub fn current_runtime_path(&self) -> PathBuf {
        self.runtime_dir().join(runtime_executable_name())
    }

    pub fn current_runtime_version(&self) -> AppResult<String> {
        Ok(self
            .runtime_status()?
            .current_version
            .unwrap_or_else(|| "not installed".to_string()))
    }

    pub fn runtime_status(&self) -> AppResult<RuntimeStatus> {
        let platform = RuntimePlatform::current();
        let Some(metadata) = self.read_metadata()? else {
            return Ok(RuntimeStatus {
                installed: false,
                current_version: None,
                runtime_path: None,
                platform,
            });
        };

        if metadata.os != platform.os || metadata.arch != platform.arch {
            return Ok(RuntimeStatus {
                installed: false,
                current_version: None,
                runtime_path: None,
                platform,
            });
        }

        if !metadata.path.exists() {
            self.clear_metadata()?;
            return Ok(RuntimeStatus {
                installed: false,
                current_version: None,
                runtime_path: None,
                platform,
            });
        }

        Ok(RuntimeStatus {
            installed: true,
            current_version: Some(metadata.version),
            runtime_path: Some(metadata.path),
            platform,
        })
    }

    pub fn installed_runtime_path(&self) -> AppResult<PathBuf> {
        self.runtime_status()?.runtime_path.ok_or_else(|| {
            AppError::Runtime(
                "frpc runtime is not installed. Open Runtime Settings to download it.".into(),
            )
        })
    }

    pub fn install_runtime_archive(
        &self,
        version: &str,
        os: &str,
        arch: &str,
        asset_name: &str,
        archive_bytes: &[u8],
    ) -> AppResult<RuntimeStatus> {
        let executable_bytes = extract_runtime_executable(asset_name, archive_bytes)?;
        std::fs::create_dir_all(self.runtime_dir())?;
        let runtime_path = self.runtime_dir().join(format!(
            "frpc-{version}-{os}-{arch}{}",
            executable_suffix(os)
        ));
        std::fs::write(&runtime_path, executable_bytes)?;
        make_executable(&runtime_path)?;

        let metadata = RuntimeMetadata {
            version: version.to_string(),
            os: os.to_string(),
            arch: arch.to_string(),
            path: runtime_path,
            asset_name: asset_name.to_string(),
            installed_at: Utc::now(),
        };
        self.write_metadata(&metadata)?;
        self.runtime_status()
    }

    pub fn read_runtime_metadata(&self) -> AppResult<Option<RuntimeMetadata>> {
        self.read_metadata()
    }

    pub async fn latest_release(&self) -> AppResult<CachedRelease> {
        if let Some(cache) = self.read_release_cache()? {
            if Utc::now().signed_duration_since(cache.fetched_at) < chrono::Duration::minutes(30) {
                return Ok(CachedRelease {
                    release: cache.release,
                    source: "cache",
                    warning: None,
                });
            }
        }

        match fetch_latest_release().await {
            Ok(release) => {
                self.write_release_cache(&release)?;
                Ok(CachedRelease {
                    release,
                    source: "network",
                    warning: None,
                })
            }
            Err(err) => {
                if let Some(cache) = self.read_release_cache()? {
                    return Ok(CachedRelease {
                        release: cache.release,
                        source: "stale-cache",
                        warning: Some(err.to_string()),
                    });
                }
                Err(err)
            }
        }
    }

    fn metadata_path(&self) -> PathBuf {
        self.runtime_dir().join("current.json")
    }

    fn release_cache_path(&self) -> PathBuf {
        self.runtime_dir().join("latest-release.json")
    }

    fn read_metadata(&self) -> AppResult<Option<RuntimeMetadata>> {
        let path = self.metadata_path();
        if !path.exists() {
            return Ok(None);
        }

        let raw = std::fs::read_to_string(path)?;
        let metadata = serde_json::from_str(&raw)
            .map_err(|err| AppError::Runtime(format!("invalid runtime metadata: {err}")))?;
        Ok(Some(metadata))
    }

    fn write_metadata(&self, metadata: &RuntimeMetadata) -> AppResult<()> {
        std::fs::create_dir_all(self.runtime_dir())?;
        let raw = serde_json::to_string_pretty(metadata).map_err(|err| {
            AppError::Runtime(format!("serialize runtime metadata failed: {err}"))
        })?;
        std::fs::write(self.metadata_path(), raw)?;
        Ok(())
    }

    fn clear_metadata(&self) -> AppResult<()> {
        let path = self.metadata_path();
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }

    fn read_release_cache(&self) -> AppResult<Option<ReleaseCache>> {
        let path = self.release_cache_path();
        if !path.exists() {
            return Ok(None);
        }

        let raw = std::fs::read_to_string(path)?;
        let cache = serde_json::from_str(&raw)
            .map_err(|err| AppError::Runtime(format!("invalid release cache: {err}")))?;
        Ok(Some(cache))
    }

    fn write_release_cache(&self, release: &GitHubRelease) -> AppResult<()> {
        std::fs::create_dir_all(self.runtime_dir())?;
        let cache = ReleaseCache {
            fetched_at: Utc::now(),
            release: release.clone(),
        };
        let raw = serde_json::to_string_pretty(&cache)
            .map_err(|err| AppError::Runtime(format!("serialize release cache failed: {err}")))?;
        std::fs::write(self.release_cache_path(), raw)?;
        Ok(())
    }
}

impl RuntimePlatform {
    pub fn current() -> Self {
        let (os, arch) = current_platform();
        Self {
            os: os.to_string(),
            arch: arch.to_string(),
            executable_name: runtime_executable_name().to_string(),
        }
    }
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("{digest:x}")
}

pub fn current_platform() -> (&'static str, &'static str) {
    let os = if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "linux"
    };

    let arch = if cfg!(target_arch = "aarch64") {
        "arm64"
    } else if cfg!(target_arch = "x86_64") {
        "amd64"
    } else {
        "unknown"
    };

    (os, arch)
}

fn runtime_executable_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "frpc.exe"
    } else {
        "frpc"
    }
}

fn executable_suffix(os: &str) -> &'static str {
    if os == "windows" {
        ".exe"
    } else {
        ""
    }
}

fn extract_runtime_executable(asset_name: &str, archive_bytes: &[u8]) -> AppResult<Vec<u8>> {
    if asset_name.ends_with(".zip") {
        extract_from_zip(archive_bytes)
    } else if asset_name.ends_with(".tar.gz") {
        extract_from_tar_gz(archive_bytes)
    } else {
        Err(AppError::Update(format!(
            "unsupported frpc runtime archive: {asset_name}"
        )))
    }
}

fn extract_from_tar_gz(archive_bytes: &[u8]) -> AppResult<Vec<u8>> {
    let decoder = flate2::read::GzDecoder::new(Cursor::new(archive_bytes));
    let mut archive = tar::Archive::new(decoder);
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
        if path.file_name().and_then(|name| name.to_str()) == Some("frpc") {
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes)?;
            return Ok(bytes);
        }
    }

    Err(AppError::Update(
        "frpc executable not found in archive".into(),
    ))
}

fn extract_from_zip(archive_bytes: &[u8]) -> AppResult<Vec<u8>> {
    let cursor = Cursor::new(archive_bytes);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|err| AppError::Update(format!("invalid frpc zip archive: {err}")))?;
    for index in 0..archive.len() {
        let mut file = archive
            .by_index(index)
            .map_err(|err| AppError::Update(format!("invalid frpc zip entry: {err}")))?;
        let Some(name) = file.enclosed_name() else {
            continue;
        };
        if name.file_name().and_then(|name| name.to_str()) == Some("frpc.exe") {
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)?;
            return Ok(bytes);
        }
    }

    Err(AppError::Update(
        "frpc.exe executable not found in archive".into(),
    ))
}

fn make_executable(path: &std::path::Path) -> AppResult<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = std::fs::metadata(path)?.permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(path, permissions)?;
    }
    Ok(())
}
