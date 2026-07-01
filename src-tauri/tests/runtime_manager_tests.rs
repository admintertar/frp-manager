use frp_manager_lib::github_release::{select_platform_asset, ReleaseAsset};
use frp_manager_lib::runtime_manager::{sha256_hex, RuntimeManager};
use tempfile::tempdir;

#[test]
fn selects_darwin_arm64_asset() {
    let assets = vec![
        ReleaseAsset {
            name: "frp_0.69.1_darwin_amd64.tar.gz".into(),
            browser_download_url: "amd64".into(),
        },
        ReleaseAsset {
            name: "frp_0.69.1_darwin_arm64.tar.gz".into(),
            browser_download_url: "arm64".into(),
        },
        ReleaseAsset {
            name: "frp_sha256_checksums.txt".into(),
            browser_download_url: "checksums".into(),
        },
    ];

    let asset = select_platform_asset("0.69.1", "darwin", "arm64", &assets).unwrap();

    assert_eq!(asset.name, "frp_0.69.1_darwin_arm64.tar.gz");
}

#[test]
fn calculates_sha256_hex() {
    assert_eq!(
        sha256_hex(b"frp"),
        "0c4243c45502e6c965536517cc132e91c66ebfe8155a47aaa6cfdeab35b24664"
    );
}

#[test]
fn discovers_bundled_runtime_version() {
    let dir = tempdir().unwrap();
    let manager = RuntimeManager::new(dir.path().to_path_buf());

    let info = manager.runtime_info_for_version("0.69.1", "darwin", "arm64");

    assert_eq!(info.version, "0.69.1");
    assert!(info.path.ends_with("frpc-0.69.1-darwin-arm64"));
}
