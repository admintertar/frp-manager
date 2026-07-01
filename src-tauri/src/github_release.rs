use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

pub const FRP_RELEASES_API: &str = "https://api.github.com/repos/fatedier/frp/releases/latest";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub assets: Vec<ReleaseAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
}

pub fn select_platform_asset(
    version: &str,
    os: &str,
    arch: &str,
    assets: &[ReleaseAsset],
) -> AppResult<ReleaseAsset> {
    let expected = format!("frp_{version}_{os}_{arch}.tar.gz");
    assets
        .iter()
        .find(|asset| asset.name == expected)
        .cloned()
        .ok_or_else(|| AppError::Update(format!("release asset {expected} not found")))
}

pub fn select_checksums_asset(assets: &[ReleaseAsset]) -> AppResult<ReleaseAsset> {
    assets
        .iter()
        .find(|asset| asset.name == "frp_sha256_checksums.txt")
        .cloned()
        .ok_or_else(|| AppError::Update("frp_sha256_checksums.txt not found".into()))
}

pub async fn fetch_latest_release() -> AppResult<GitHubRelease> {
    let client = reqwest::Client::new();
    client
        .get(FRP_RELEASES_API)
        .header("User-Agent", "frp-manager")
        .send()
        .await
        .map_err(|err| AppError::Update(err.to_string()))?
        .error_for_status()
        .map_err(|err| AppError::Update(err.to_string()))?
        .json::<GitHubRelease>()
        .await
        .map_err(|err| AppError::Update(err.to_string()))
}
