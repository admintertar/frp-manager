use std::fs;

use frp_manager_lib::config_toml::{parse_profile_toml, set_proxy_enabled};
use frp_manager_lib::error::AppError;
use frp_manager_lib::models::ProfileMeta;
use frp_manager_lib::profile_store::ProfileStore;
use tempfile::tempdir;

const SAMPLE_TOML: &str = r#"
serverAddr = "frp.ala4.com"
serverPort = 7000

auth.method = "token"
auth.token = "secret"

[[proxies]]
name = "zwd"
type = "http"
localIP = "0.0.0.0"
localPort = 8123
subdomain = "zwd"
"#;

#[test]
fn parses_existing_frpc_toml() {
    let profile = parse_profile_toml("frp-ala4-com", "frp.ala4.com", SAMPLE_TOML).unwrap();
    assert_eq!(profile.server_addr, "frp.ala4.com");
    assert_eq!(profile.server_port, 7000);
    assert_eq!(profile.proxies.len(), 1);
    assert_eq!(profile.proxies[0].name, "zwd");
    assert!(profile.proxies[0].enabled);
}

#[test]
fn toggles_proxy_enabled_in_native_toml() {
    let updated = set_proxy_enabled(SAMPLE_TOML, "zwd", false).unwrap();
    assert!(updated.contains("enabled = false"));
    let profile = parse_profile_toml("frp-ala4-com", "frp.ala4.com", &updated).unwrap();
    assert!(!profile.proxies[0].enabled);
}

#[test]
fn imports_profile_into_app_data_directory() {
    let dir = tempdir().unwrap();
    let store = ProfileStore::new(dir.path().to_path_buf());
    let summary = store.import_from_text("frp.ala4.com", SAMPLE_TOML).unwrap();
    assert_eq!(summary.display_name, "frp.ala4.com");

    let profile_path = dir
        .path()
        .join("profiles")
        .join(&summary.id)
        .join("profile.toml");
    let meta_path = dir
        .path()
        .join("profiles")
        .join(&summary.id)
        .join("meta.json");
    assert!(profile_path.exists());
    assert!(meta_path.exists());

    let saved = fs::read_to_string(profile_path).unwrap();
    assert!(saved.contains("serverAddr = \"frp.ala4.com\""));
}

#[test]
fn import_rejects_duplicate_profile_slug_without_overwriting_existing_profile() {
    let dir = tempdir().unwrap();
    let store = ProfileStore::new(dir.path().to_path_buf());
    let first = store.import_from_text("frp.ala4.com", SAMPLE_TOML).unwrap();
    let profile_path = dir
        .path()
        .join("profiles")
        .join(&first.id)
        .join("profile.toml");
    let original = fs::read_to_string(&profile_path).unwrap();

    let duplicate = store.import_from_text("frp.ala4.com", SAMPLE_TOML);

    assert!(matches!(duplicate, Err(AppError::Validation(_))));
    assert_eq!(fs::read_to_string(profile_path).unwrap(), original);
}

#[test]
fn invalid_import_does_not_create_profile_directory() {
    let dir = tempdir().unwrap();
    let store = ProfileStore::new(dir.path().to_path_buf());
    let invalid = SAMPLE_TOML.replace("serverAddr", "missingServerAddr");

    let result = store.import_from_text("bad profile", &invalid);

    assert!(matches!(result, Err(AppError::Validation(_))));
    assert!(!dir.path().join("profiles").join("bad-profile").exists());
}

#[test]
fn load_rejects_path_traversal_profile_id() {
    let dir = tempdir().unwrap();
    let store = ProfileStore::new(dir.path().to_path_buf());

    let result = store.load("../outside");

    assert!(matches!(result, Err(AppError::Validation(_))));
}

#[test]
fn profile_toml_path_rejects_path_traversal_profile_id() {
    let dir = tempdir().unwrap();
    let store = ProfileStore::new(dir.path().to_path_buf());

    let result = store.profile_toml_path("../outside");

    assert!(matches!(result, Err(AppError::Validation(_))));
}

#[test]
fn parse_rejects_wrong_port_type() {
    let invalid = SAMPLE_TOML.replace("localPort = 8123", "localPort = \"8123\"");

    let result = parse_profile_toml("frp-ala4-com", "frp.ala4.com", &invalid);

    assert!(matches!(result, Err(AppError::Validation(_))));
}

#[test]
fn parse_rejects_wrong_enabled_type() {
    let invalid = SAMPLE_TOML.replace(
        "subdomain = \"zwd\"",
        "subdomain = \"zwd\"\nenabled = \"false\"",
    );

    let result = parse_profile_toml("frp-ala4-com", "frp.ala4.com", &invalid);

    assert!(matches!(result, Err(AppError::Validation(_))));
}

#[test]
fn parse_rejects_non_string_custom_domain_entries() {
    let invalid = SAMPLE_TOML.replace(
        "subdomain = \"zwd\"",
        "customDomains = [\"ok.example.com\", 42]",
    );

    let result = parse_profile_toml("frp-ala4-com", "frp.ala4.com", &invalid);

    assert!(matches!(result, Err(AppError::Validation(_))));
}

#[test]
fn parse_rejects_malformed_proxies_field() {
    let invalid = r#"
serverAddr = "frp.ala4.com"
serverPort = 7000

proxies = "not an array of proxy tables"
"#;

    let result = parse_profile_toml("frp-ala4-com", "frp.ala4.com", invalid);

    assert!(matches!(result, Err(AppError::Validation(_))));
}

#[test]
fn parse_rejects_wrong_optional_string_types() {
    let invalid = SAMPLE_TOML
        .replace("auth.method = \"token\"", "auth.method = 42")
        .replace("localIP = \"0.0.0.0\"", "localIP = false")
        .replace("subdomain = \"zwd\"", "subdomain = 42");

    let result = parse_profile_toml("frp-ala4-com", "frp.ala4.com", &invalid);

    assert!(matches!(result, Err(AppError::Validation(_))));
}

#[test]
fn parse_rejects_malformed_optional_parent_tables() {
    let invalid = SAMPLE_TOML.replace(
        "auth.method = \"token\"\nauth.token = \"secret\"",
        "auth = \"bad\"",
    );

    let result = parse_profile_toml("frp-ala4-com", "frp.ala4.com", &invalid);

    assert!(matches!(result, Err(AppError::Validation(_))));
}

#[test]
fn save_raw_toml_rejects_invalid_toml_without_changing_existing_file() {
    let dir = tempdir().unwrap();
    let store = ProfileStore::new(dir.path().to_path_buf());
    let summary = store.import_from_text("frp.ala4.com", SAMPLE_TOML).unwrap();
    let path = store.profile_toml_path(&summary.id).unwrap();
    let original = fs::read_to_string(&path).unwrap();
    let invalid = SAMPLE_TOML.replace("serverAddr", "missingServerAddr");

    let result = store.save_raw_toml(&summary.id, &invalid);

    assert!(matches!(result, Err(AppError::Validation(_))));
    assert_eq!(fs::read_to_string(path).unwrap(), original);
}

#[test]
fn save_raw_toml_does_not_depend_on_fixed_tmp_path() {
    let dir = tempdir().unwrap();
    let store = ProfileStore::new(dir.path().to_path_buf());
    let summary = store.import_from_text("frp.ala4.com", SAMPLE_TOML).unwrap();
    let path = store.profile_toml_path(&summary.id).unwrap();
    fs::create_dir(path.with_extension("tmp")).unwrap();
    let updated = SAMPLE_TOML.replace("localPort = 8123", "localPort = 9000");

    store.save_raw_toml(&summary.id, &updated).unwrap();

    assert!(fs::read_to_string(path)
        .unwrap()
        .contains("localPort = 9000"));
}

#[test]
fn update_display_name_changes_metadata_without_renaming_profile_id() {
    let dir = tempdir().unwrap();
    let store = ProfileStore::new(dir.path().to_path_buf());
    let summary = store.import_from_text("frp.ala4.com", SAMPLE_TOML).unwrap();

    store
        .update_display_name(&summary.id, "Production Edge")
        .unwrap();
    let profile = store.load(&summary.id).unwrap();

    assert_eq!(profile.id, summary.id);
    assert_eq!(profile.display_name, "Production Edge");
    assert_eq!(profile.meta.display_name, "Production Edge");
}

#[test]
fn delete_profile_removes_profile_directory() {
    let dir = tempdir().unwrap();
    let store = ProfileStore::new(dir.path().to_path_buf());
    let summary = store.import_from_text("frp.ala4.com", SAMPLE_TOML).unwrap();
    let profile_dir = dir.path().join("profiles").join(&summary.id);
    assert!(profile_dir.exists());

    store.delete(&summary.id).unwrap();

    assert!(!profile_dir.exists());
    assert!(matches!(
        store.load(&summary.id),
        Err(AppError::ProfileNotFound(_))
    ));
}

#[test]
fn delete_profile_rejects_path_traversal_profile_id() {
    let dir = tempdir().unwrap();
    let store = ProfileStore::new(dir.path().to_path_buf());

    let result = store.delete("../outside");

    assert!(matches!(result, Err(AppError::Validation(_))));
}

#[test]
fn load_rejects_meta_id_that_does_not_match_directory_id() {
    let dir = tempdir().unwrap();
    let store = ProfileStore::new(dir.path().to_path_buf());
    let summary = store.import_from_text("frp.ala4.com", SAMPLE_TOML).unwrap();
    let meta_path = dir
        .path()
        .join("profiles")
        .join(&summary.id)
        .join("meta.json");
    let mut meta: ProfileMeta =
        serde_json::from_str(&fs::read_to_string(&meta_path).unwrap()).unwrap();
    meta.id = "other-id".to_string();
    fs::write(meta_path, serde_json::to_string_pretty(&meta).unwrap()).unwrap();

    let result = store.load(&summary.id);

    assert!(matches!(result, Err(AppError::Validation(_))));
}
