use flate2::write::GzEncoder;
use flate2::Compression;
use frp_manager_lib::github_release::{select_platform_asset, verify_asset_checksum, ReleaseAsset};
use frp_manager_lib::runtime_manager::{current_platform, sha256_hex, RuntimeManager};
use std::io::{Cursor, Write};
use tar::{Builder, Header};
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
fn selects_windows_amd64_zip_asset() {
    let assets = vec![
        ReleaseAsset {
            name: "frp_0.69.1_windows_amd64.zip".into(),
            browser_download_url: "windows".into(),
        },
        ReleaseAsset {
            name: "frp_0.69.1_windows_arm64.zip".into(),
            browser_download_url: "windows-arm".into(),
        },
        ReleaseAsset {
            name: "frp_0.69.1_linux_amd64.tar.gz".into(),
            browser_download_url: "linux".into(),
        },
    ];

    let asset = select_platform_asset("0.69.1", "windows", "amd64", &assets).unwrap();

    assert_eq!(asset.name, "frp_0.69.1_windows_amd64.zip");
}

#[test]
fn calculates_sha256_hex() {
    assert_eq!(
        sha256_hex(b"frp"),
        "0c4243c45502e6c965536517cc132e91c66ebfe8155a47aaa6cfdeab35b24664"
    );
}

#[test]
fn reports_runtime_not_installed_until_metadata_exists() {
    let dir = tempdir().unwrap();
    let manager = RuntimeManager::new(dir.path().to_path_buf());

    let status = manager.runtime_status().unwrap();

    assert!(!status.installed);
    assert_eq!(status.current_version, None);
    assert_eq!(status.runtime_path, None);
    let (os, arch) = current_platform();
    assert_eq!(status.platform.os, os);
    assert_eq!(status.platform.arch, arch);
}

#[test]
fn installs_frpc_from_tar_gz_archive() {
    let dir = tempdir().unwrap();
    let manager = RuntimeManager::new(dir.path().to_path_buf());
    let (os, arch) = current_platform();
    let asset_name = format!("frp_0.69.1_{os}_{arch}.tar.gz");
    let archive = make_tar_gz("frp_0.69.1/frpc", b"#!/bin/sh\necho 0.69.1\n");

    let status = manager
        .install_runtime_archive("0.69.1", os, arch, &asset_name, &archive)
        .unwrap();

    let runtime_path = status.runtime_path.unwrap();
    assert!(status.installed);
    assert_eq!(status.current_version.as_deref(), Some("0.69.1"));
    assert!(runtime_path.exists());
    assert_eq!(
        std::fs::read_to_string(runtime_path).unwrap(),
        "#!/bin/sh\necho 0.69.1\n"
    );
}

#[test]
fn clears_stale_metadata_when_installed_runtime_file_is_missing() {
    let dir = tempdir().unwrap();
    let manager = RuntimeManager::new(dir.path().to_path_buf());
    let (os, arch) = current_platform();
    let asset_name = format!("frp_0.69.1_{os}_{arch}.tar.gz");
    let archive = make_tar_gz("frp_0.69.1/frpc", b"#!/bin/sh\necho 0.69.1\n");
    let status = manager
        .install_runtime_archive("0.69.1", os, arch, &asset_name, &archive)
        .unwrap();
    let runtime_path = status.runtime_path.unwrap();
    std::fs::remove_file(runtime_path).unwrap();

    let refreshed = manager.runtime_status().unwrap();

    assert!(!refreshed.installed);
    assert_eq!(refreshed.current_version, None);
    assert_eq!(refreshed.runtime_path, None);
    assert_eq!(manager.read_runtime_metadata().unwrap(), None);
}

#[test]
fn installs_frpc_from_windows_zip_archive() {
    let dir = tempdir().unwrap();
    let manager = RuntimeManager::new(dir.path().to_path_buf());
    let archive = make_zip("frp_0.69.1_windows_amd64/frpc.exe", b"windows-frpc");

    let status = manager
        .install_runtime_archive(
            "0.69.1",
            "windows",
            "amd64",
            "frp_0.69.1_windows_amd64.zip",
            &archive,
        )
        .unwrap();

    let metadata = manager.read_runtime_metadata().unwrap().unwrap();
    assert_eq!(metadata.version, "0.69.1");
    assert_eq!(metadata.os, "windows");
    assert_eq!(metadata.arch, "amd64");
    assert!(metadata.path.ends_with("frpc-0.69.1-windows-amd64.exe"));
    assert_eq!(std::fs::read(metadata.path).unwrap(), b"windows-frpc");
    assert!(!status.installed);
}

#[test]
fn verifies_release_asset_checksum() {
    let bytes = b"frpc archive";
    let checksum = sha256_hex(bytes);
    let checksums =
        format!("{checksum}  frp_0.69.1_darwin_arm64.tar.gz\n0000  other_asset.tar.gz\n");

    verify_asset_checksum("frp_0.69.1_darwin_arm64.tar.gz", bytes, &checksums).unwrap();
}

#[test]
fn rejects_release_asset_checksum_mismatch() {
    let err = verify_asset_checksum(
        "frp_0.69.1_darwin_arm64.tar.gz",
        b"actual archive",
        "000000  frp_0.69.1_darwin_arm64.tar.gz\n",
    )
    .unwrap_err();

    assert!(err.to_string().contains("checksum mismatch"));
}

fn make_tar_gz(path: &str, bytes: &[u8]) -> Vec<u8> {
    let mut tar_bytes = Vec::new();
    {
        let mut builder = Builder::new(&mut tar_bytes);
        let mut header = Header::new_gnu();
        header.set_size(bytes.len() as u64);
        header.set_mode(0o755);
        header.set_cksum();
        builder
            .append_data(&mut header, path, Cursor::new(bytes))
            .unwrap();
        builder.finish().unwrap();
    }

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&tar_bytes).unwrap();
    encoder.finish().unwrap()
}

fn make_zip(path: &str, bytes: &[u8]) -> Vec<u8> {
    let cursor = Cursor::new(Vec::new());
    let mut writer = zip::ZipWriter::new(cursor);
    writer
        .start_file(path, zip::write::SimpleFileOptions::default())
        .unwrap();
    writer.write_all(bytes).unwrap();
    writer.finish().unwrap().into_inner()
}
