use std::fs;
use std::time::Duration;

use frp_manager_lib::app_state::AppState;
use frp_manager_lib::commands::{
    app_update_available, append_proxy_toml, clean_log_output, create_profile_toml,
    delete_proxy_toml, list_profiles_with_runtime_state, profile_log_path, start_profile_by_id,
    update_profile_toml, update_proxy_toml, AddProxyInput, AuthMethodInput, CreateProfileInput,
};
use frp_manager_lib::config_toml::parse_profile_toml;
use frp_manager_lib::error::AppError;
use frp_manager_lib::github_release::app_release_tag_from_latest_url;
use frp_manager_lib::models::{ProxyType, RuntimeState};
use frp_manager_lib::process_manager::ProcessRegistry;
use frp_manager_lib::profile_store::ProfileStore;
use tempfile::tempdir;
use tokio::time::sleep;

const SAMPLE_TOML: &str = r#"
serverAddr = "frp.ala4.com"
serverPort = 7000

[[proxies]]
name = "web"
type = "http"
localPort = 8080
"#;

#[test]
fn profile_log_path_rejects_traversal_ids() {
    let dir = tempdir().unwrap();

    let result = profile_log_path(dir.path(), "../outside");

    assert!(matches!(result, Err(AppError::Validation(_))));
}

#[test]
fn profile_log_path_requires_existing_profile() {
    let dir = tempdir().unwrap();

    let result = profile_log_path(dir.path(), "missing-profile");

    assert!(matches!(result, Err(AppError::ProfileNotFound(_))));
}

#[test]
fn profile_log_path_returns_current_log_for_existing_profile() {
    let dir = tempdir().unwrap();
    let store = ProfileStore::new(dir.path().to_path_buf());
    let summary = store.import_from_text("frp.ala4.com", SAMPLE_TOML).unwrap();

    let path = profile_log_path(dir.path(), &summary.id).unwrap();

    assert!(path.ends_with("logs/current.log"));
}

#[test]
fn clean_log_output_strips_ansi_color_sequences() {
    let raw = "\u{1b}[1;34m2026-07-01 17:51:43.499 [I] connected\u{1b}[0m\nplain";

    let cleaned = clean_log_output(raw);

    assert_eq!(cleaned, "2026-07-01 17:51:43.499 [I] connected\nplain");
}

#[test]
fn app_update_available_compares_semver_release_tags() {
    assert!(app_update_available("0.1.0", "v0.1.1"));
    assert!(app_update_available("0.1.0", "0.2.0"));
    assert!(!app_update_available("0.1.0", "v0.1.0"));
    assert!(!app_update_available("0.1.0", "v0.0.4"));
}

#[test]
fn app_release_tag_parses_github_latest_redirect_url() {
    let tag = app_release_tag_from_latest_url(
        "https://github.com/admintertar/frp-manager/releases/tag/v0.1.1",
    )
    .unwrap();

    assert_eq!(tag, "v0.1.1");
}

#[tokio::test]
async fn start_profile_requires_managed_runtime() {
    let dir = tempdir().unwrap();
    let state = AppState::new(dir.path().to_path_buf());
    state
        .profile_store()
        .import_from_text("prod", "serverAddr = \"example.com\"\nserverPort = 7000\n")
        .unwrap();

    let err = start_profile_by_id(&state, "prod").await.unwrap_err();

    assert!(matches!(err, AppError::Runtime(_)));
    assert!(err.to_string().contains("frpc runtime is not installed"));
}

#[test]
fn create_profile_toml_builds_token_auth_profile() {
    let input = CreateProfileInput {
        profile_name: "Prod Edge".into(),
        server_addr: "frp.example.com".into(),
        server_port: 7000,
        auth_method: AuthMethodInput::Token,
        auth_token: Some("secret-token".into()),
        oidc_client_id: None,
        oidc_client_secret: None,
        oidc_audience: None,
        oidc_token_endpoint_url: None,
    };

    let toml = create_profile_toml(&input).unwrap();
    let profile = parse_profile_toml("prod-edge", "Prod Edge", &toml).unwrap();

    assert_eq!(profile.server_addr, "frp.example.com");
    assert_eq!(profile.server_port, 7000);
    assert_eq!(profile.auth_method.as_deref(), Some("token"));
    assert_eq!(profile.auth_token.as_deref(), Some("secret-token"));
    assert!(toml.contains("serverAddr = \"frp.example.com\""));
}

#[test]
fn create_profile_toml_builds_oidc_auth_profile() {
    let input = CreateProfileInput {
        profile_name: "OIDC Edge".into(),
        server_addr: "frp.example.com".into(),
        server_port: 7000,
        auth_method: AuthMethodInput::Oidc,
        auth_token: None,
        oidc_client_id: Some("frpc-client".into()),
        oidc_client_secret: Some("client-secret".into()),
        oidc_audience: Some("frps".into()),
        oidc_token_endpoint_url: Some("https://issuer.example.com/oauth/token".into()),
    };

    let toml = create_profile_toml(&input).unwrap();
    let profile = parse_profile_toml("oidc-edge", "OIDC Edge", &toml).unwrap();

    assert_eq!(profile.auth_method.as_deref(), Some("oidc"));
    assert!(toml.contains("clientID = \"frpc-client\""));
    assert!(toml.contains("clientSecret = \"client-secret\""));
    assert!(toml.contains("audience = \"frps\""));
    assert!(toml.contains("tokenEndpointURL = \"https://issuer.example.com/oauth/token\""));
}

#[test]
fn create_profile_toml_rejects_incomplete_oidc_auth_profile() {
    let input = CreateProfileInput {
        profile_name: "OIDC Edge".into(),
        server_addr: "frp.example.com".into(),
        server_port: 7000,
        auth_method: AuthMethodInput::Oidc,
        auth_token: None,
        oidc_client_id: Some("frpc-client".into()),
        oidc_client_secret: None,
        oidc_audience: Some("frps".into()),
        oidc_token_endpoint_url: Some("https://issuer.example.com/oauth/token".into()),
    };

    let result = create_profile_toml(&input);

    assert!(matches!(result, Err(AppError::Validation(_))));
}

#[test]
fn update_profile_toml_preserves_proxies_and_updates_server_auth() {
    let input = CreateProfileInput {
        profile_name: "Prod Edge".into(),
        server_addr: "frp.changed.com".into(),
        server_port: 7443,
        auth_method: AuthMethodInput::Token,
        auth_token: Some("new-token".into()),
        oidc_client_id: None,
        oidc_client_secret: None,
        oidc_audience: None,
        oidc_token_endpoint_url: None,
    };

    let updated = update_profile_toml(SAMPLE_TOML, &input).unwrap();
    let profile = parse_profile_toml("frp-ala4-com", "Prod Edge", &updated).unwrap();

    assert_eq!(profile.server_addr, "frp.changed.com");
    assert_eq!(profile.server_port, 7443);
    assert_eq!(profile.auth_method.as_deref(), Some("token"));
    assert_eq!(profile.auth_token.as_deref(), Some("new-token"));
    assert_eq!(profile.proxies.len(), 1);
    assert_eq!(profile.proxies[0].name, "web");
    assert_eq!(profile.proxies[0].local_port, Some(8080));
}

#[test]
fn append_proxy_toml_adds_tcp_proxy() {
    let input = AddProxyInput {
        name: "ssh".into(),
        proxy_type: ProxyType::Tcp,
        local_ip: Some("127.0.0.1".into()),
        local_port: Some(22),
        remote_port: Some(6000),
        subdomain: None,
        custom_domains: Vec::new(),
    };

    let updated = append_proxy_toml(SAMPLE_TOML, &input).unwrap();
    let profile = parse_profile_toml("frp-ala4-com", "frp.ala4.com", &updated).unwrap();

    assert_eq!(profile.proxies.len(), 2);
    assert_eq!(profile.proxies[1].name, "ssh");
    assert_eq!(profile.proxies[1].proxy_type, ProxyType::Tcp);
    assert_eq!(profile.proxies[1].remote_port, Some(6000));
}

#[test]
fn append_proxy_toml_rejects_duplicate_proxy_name() {
    let input = AddProxyInput {
        name: "web".into(),
        proxy_type: ProxyType::Http,
        local_ip: Some("127.0.0.1".into()),
        local_port: Some(8080),
        remote_port: None,
        subdomain: Some("web".into()),
        custom_domains: Vec::new(),
    };

    let result = append_proxy_toml(SAMPLE_TOML, &input);

    assert!(matches!(result, Err(AppError::Validation(_))));
}

#[test]
fn update_proxy_toml_replaces_proxy_and_preserves_enabled_state() {
    let disabled =
        frp_manager_lib::config_toml::set_proxy_enabled(SAMPLE_TOML, "web", false).unwrap();
    let input = AddProxyInput {
        name: "web-secure".into(),
        proxy_type: ProxyType::Https,
        local_ip: Some("127.0.0.1".into()),
        local_port: Some(8443),
        remote_port: None,
        subdomain: Some("secure".into()),
        custom_domains: vec!["secure.example.com".into()],
    };

    let updated = update_proxy_toml(&disabled, "web", &input).unwrap();
    let profile = parse_profile_toml("frp-ala4-com", "frp.ala4.com", &updated).unwrap();

    assert_eq!(profile.proxies.len(), 1);
    assert_eq!(profile.proxies[0].name, "web-secure");
    assert_eq!(profile.proxies[0].proxy_type, ProxyType::Https);
    assert_eq!(profile.proxies[0].local_port, Some(8443));
    assert_eq!(profile.proxies[0].subdomain.as_deref(), Some("secure"));
    assert_eq!(
        profile.proxies[0].custom_domains,
        vec!["secure.example.com".to_string()]
    );
    assert!(!profile.proxies[0].enabled);
}

#[test]
fn delete_proxy_toml_removes_named_proxy() {
    let input = AddProxyInput {
        name: "ssh".into(),
        proxy_type: ProxyType::Tcp,
        local_ip: Some("127.0.0.1".into()),
        local_port: Some(22),
        remote_port: Some(6000),
        subdomain: None,
        custom_domains: Vec::new(),
    };
    let with_two = append_proxy_toml(SAMPLE_TOML, &input).unwrap();

    let updated = delete_proxy_toml(&with_two, "web").unwrap();
    let profile = parse_profile_toml("frp-ala4-com", "frp.ala4.com", &updated).unwrap();

    assert_eq!(profile.proxies.len(), 1);
    assert_eq!(profile.proxies[0].name, "ssh");
}

#[tokio::test]
async fn list_profiles_with_runtime_state_reports_running_process() {
    let dir = tempdir().unwrap();
    let store = ProfileStore::new(dir.path().to_path_buf());
    let summary = store.import_from_text("frp.ala4.com", SAMPLE_TOML).unwrap();
    let fake = dir.path().join("fake-frpc");
    write_executable(&fake, "#!/bin/sh\nsleep 30\n");
    let config_path = store.profile_toml_path(&summary.id).unwrap();
    let working_dir = config_path.parent().unwrap();
    let mut registry = ProcessRegistry::default();
    registry
        .start(&summary.id, &fake, &config_path, working_dir)
        .await
        .unwrap();

    let profiles = list_profiles_with_runtime_state(&store, &mut registry).unwrap();

    assert_eq!(profiles[0].runtime_state, RuntimeState::Running);
    registry.stop(&summary.id).await.unwrap();
}

#[tokio::test]
async fn list_profiles_with_runtime_state_marks_exited_process_failed() {
    let dir = tempdir().unwrap();
    let store = ProfileStore::new(dir.path().to_path_buf());
    let summary = store.import_from_text("frp.ala4.com", SAMPLE_TOML).unwrap();
    let fake = dir.path().join("fake-frpc");
    write_executable(
        &fake,
        "#!/bin/sh\ntouch ready\nwhile [ ! -f exit-now ]; do sleep 0.05; done\nexit 3\n",
    );
    let config_path = store.profile_toml_path(&summary.id).unwrap();
    let working_dir = config_path.parent().unwrap();
    let mut registry = ProcessRegistry::default();
    registry
        .start(&summary.id, &fake, &config_path, working_dir)
        .await
        .unwrap();

    assert!(working_dir.join("ready").exists());
    fs::write(working_dir.join("exit-now"), "").unwrap();
    sleep(Duration::from_millis(150)).await;
    let profiles = list_profiles_with_runtime_state(&store, &mut registry).unwrap();

    assert_eq!(profiles[0].runtime_state, RuntimeState::Failed);
}

fn write_executable(path: &std::path::Path, contents: &str) {
    fs::write(path, contents).unwrap();
    let mut permissions = fs::metadata(path).unwrap().permissions();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        permissions.set_mode(0o755);
    }
    fs::set_permissions(path, permissions).unwrap();
}
