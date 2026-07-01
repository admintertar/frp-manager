use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::AppResult;

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
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub asset_name: Option<String>,
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
            path: self
                .runtime_dir()
                .join(format!("frpc-{version}-{os}-{arch}")),
        }
    }

    pub fn current_runtime_path(&self) -> PathBuf {
        self.runtime_dir().join("frpc-current")
    }

    pub fn current_runtime_version(&self) -> AppResult<String> {
        Ok("0.69.1".to_string())
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
