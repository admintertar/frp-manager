use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::runtime_manager::sha256_hex;

pub const FRP_LATEST_RELEASE_URL: &str = "https://github.com/fatedier/frp/releases/latest";
pub const APP_LATEST_RELEASE_URL: &str =
    "https://github.com/admintertar/frp-manager/releases/latest";
const FRP_RELEASE_DOWNLOAD_BASE: &str = "https://github.com/fatedier/frp/releases/download";
const APP_RELEASE_DOWNLOAD_BASE: &str =
    "https://github.com/admintertar/frp-manager/releases/download";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    #[serde(default)]
    pub html_url: Option<String>,
    pub assets: Vec<ReleaseAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppLatestRelease {
    pub tag_name: String,
    pub html_url: String,
    pub asset_name: String,
    pub download_url: String,
}

pub fn select_platform_asset(
    version: &str,
    os: &str,
    arch: &str,
    assets: &[ReleaseAsset],
) -> AppResult<ReleaseAsset> {
    let expected = platform_asset_name(version, os, arch);
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

pub async fn fetch_latest_release_for_platform(
    os: &str,
    arch: &str,
) -> AppResult<GitHubRelease> {
    let html_url = fetch_latest_redirect_url(FRP_LATEST_RELEASE_URL).await?;
    frp_release_from_latest_url(&html_url, os, arch)
}

pub async fn fetch_latest_app_release() -> AppResult<AppLatestRelease> {
    let (os, arch) = crate::runtime_manager::current_platform();
    fetch_latest_app_release_for_platform(os, arch).await
}

pub async fn fetch_latest_app_release_for_platform(
    os: &str,
    arch: &str,
) -> AppResult<AppLatestRelease> {
    let html_url = fetch_latest_redirect_url(APP_LATEST_RELEASE_URL).await?;
    app_release_from_latest_url(&html_url, os, arch)
}

async fn fetch_latest_redirect_url(url: &str) -> AppResult<String> {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .map_err(|err| AppError::Update(err.to_string()))?;
    let response = client
        .get(url)
        .header("User-Agent", "frp-manager")
        .send()
        .await
        .map_err(|err| AppError::Update(err.to_string()))?;

    if !response.status().is_success() {
        return Err(AppError::Update(github_status_error(&response)));
    }

    Ok(response.url().to_string())
}

pub fn frp_release_from_latest_url(url: &str, os: &str, arch: &str) -> AppResult<GitHubRelease> {
    let tag_name = release_tag_from_latest_url(url)?;
    let version = tag_name.trim_start_matches('v');
    let asset_name = platform_asset_name(version, os, arch);
    let checksums_name = "frp_sha256_checksums.txt".to_string();

    Ok(GitHubRelease {
        tag_name: tag_name.clone(),
        html_url: Some(url.to_string()),
        assets: vec![
            ReleaseAsset {
                name: asset_name.clone(),
                browser_download_url: release_asset_download_url(&tag_name, &asset_name),
            },
            ReleaseAsset {
                name: checksums_name.clone(),
                browser_download_url: release_asset_download_url(&tag_name, &checksums_name),
            },
        ],
    })
}

pub fn app_release_tag_from_latest_url(url: &str) -> AppResult<String> {
    release_tag_from_latest_url(url)
}

pub fn app_release_from_latest_url(url: &str, os: &str, arch: &str) -> AppResult<AppLatestRelease> {
    let tag_name = release_tag_from_latest_url(url)?;
    let version = tag_name.trim_start_matches('v');
    let asset = select_app_platform_asset(version, os, arch)?;

    Ok(AppLatestRelease {
        tag_name: tag_name.clone(),
        html_url: url.to_string(),
        download_url: app_release_asset_download_url(&tag_name, &asset.name),
        asset_name: asset.name,
    })
}

pub fn select_app_platform_asset(version: &str, os: &str, arch: &str) -> AppResult<ReleaseAsset> {
    let (asset_arch, setup, extension) = match (os, arch) {
        ("darwin", "arm64") => ("aarch64", "", ".dmg"),
        ("darwin", "amd64") => ("x64", "", ".dmg"),
        ("windows", "amd64") => ("x64", "", ".msi"),
        ("windows", "arm64") => ("arm64", "", ".msi"),
        ("linux", "amd64") => ("amd64", "", ".AppImage"),
        ("linux", "arm64") => ("aarch64", "", ".AppImage"),
        _ => {
            return Err(AppError::Update(format!(
                "unsupported FRP Manager update platform: {os}_{arch}"
            )))
        }
    };
    let name = format!("FRP-Manager_{version}_{os}_{asset_arch}{setup}{extension}");

    Ok(ReleaseAsset {
        browser_download_url: String::new(),
        name,
    })
}

fn release_tag_from_latest_url(url: &str) -> AppResult<String> {
    let parsed = reqwest::Url::parse(url)
        .map_err(|err| AppError::Update(format!("invalid release URL: {err}")))?;
    let segments = parsed
        .path_segments()
        .ok_or_else(|| AppError::Update("latest release URL has no path".into()))?
        .collect::<Vec<_>>();

    let Some(tag_index) = segments
        .windows(2)
        .position(|window| window == ["releases", "tag"])
        .map(|index| index + 2)
    else {
        return Err(AppError::Update(format!(
            "latest release tag not found in {url}"
        )));
    };

    segments
        .get(tag_index)
        .filter(|tag| !tag.is_empty())
        .map(|tag| (*tag).to_string())
        .ok_or_else(|| AppError::Update(format!("latest release tag not found in {url}")))
}

fn platform_asset_name(version: &str, os: &str, arch: &str) -> String {
    let extension = if os == "windows" { "zip" } else { "tar.gz" };
    format!("frp_{version}_{os}_{arch}.{extension}")
}

fn release_asset_download_url(tag: &str, asset_name: &str) -> String {
    format!("{FRP_RELEASE_DOWNLOAD_BASE}/{tag}/{asset_name}")
}

fn app_release_asset_download_url(tag: &str, asset_name: &str) -> String {
    format!("{APP_RELEASE_DOWNLOAD_BASE}/{tag}/{asset_name}")
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
            return "GitHub request rate limit reached. Please try again later.".into();
        }
        return "GitHub API refused the request (403 Forbidden). Please try again later.".into();
    }

    format!("HTTP status {status}")
}
