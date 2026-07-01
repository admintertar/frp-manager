use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::runtime_manager::sha256_hex;

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
    let extension = if os == "windows" { "zip" } else { "tar.gz" };
    let expected = format!("frp_{version}_{os}_{arch}.{extension}");
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

pub fn verify_asset_checksum(asset_name: &str, bytes: &[u8], checksums: &str) -> AppResult<()> {
    let expected = checksums.lines().find_map(|line| {
        let mut parts = line.split_whitespace();
        let checksum = parts.next()?;
        let name = parts.next()?;
        (name == asset_name).then_some(checksum)
    });

    let Some(expected) = expected else {
        return Err(AppError::Update(format!(
            "checksum for {asset_name} not found"
        )));
    };

    let actual = sha256_hex(bytes);
    if actual != expected {
        return Err(AppError::Update(format!(
            "checksum mismatch for {asset_name}"
        )));
    }

    Ok(())
}

pub async fn fetch_latest_release() -> AppResult<GitHubRelease> {
    let client = reqwest::Client::new();
    let response = client
        .get(FRP_RELEASES_API)
        .header("User-Agent", "frp-manager")
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await
        .map_err(|err| AppError::Update(err.to_string()))?;

    if !response.status().is_success() {
        return Err(AppError::Update(github_status_error(&response)));
    }

    response
        .json::<GitHubRelease>()
        .await
        .map_err(|err| AppError::Update(err.to_string()))
}

pub async fn download_url(url: &str) -> AppResult<Vec<u8>> {
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header("User-Agent", "frp-manager")
        .header("Accept", "application/octet-stream")
        .send()
        .await
        .map_err(|err| AppError::Update(err.to_string()))?;

    if !response.status().is_success() {
        return Err(AppError::Update(github_status_error(&response)));
    }

    response
        .bytes()
        .await
        .map(|bytes| bytes.to_vec())
        .map_err(|err| AppError::Update(err.to_string()))
}

fn github_status_error(response: &reqwest::Response) -> String {
    let status = response.status();
    if status == reqwest::StatusCode::FORBIDDEN {
        let remaining = response
            .headers()
            .get("x-ratelimit-remaining")
            .and_then(|value| value.to_str().ok());
        if remaining == Some("0") {
            return "GitHub API rate limit reached. Please try again later.".into();
        }
        return "GitHub API refused the request (403 Forbidden). Please try again later.".into();
    }

    format!("HTTP status {status}")
}
