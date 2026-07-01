use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use tauri::{AppHandle, State};
use toml_edit::{value, ArrayOfTables, DocumentMut, Item, Table};

use crate::app_state::AppState;
use crate::config_toml::{parse_profile_toml, set_proxy_enabled};
use crate::error::{AppError, AppResult};
use crate::github_release::{fetch_latest_release, select_platform_asset};
use crate::models::{Profile, ProfileSummary, ProxyType, RuntimeState};
use crate::process_manager::{ProcessRegistry, ProfileProcessState};
use crate::profile_store::ProfileStore;
use crate::runtime_manager::{current_platform, RuntimeUpdateCheck};

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreateProfileInput {
    pub profile_name: String,
    pub server_addr: String,
    pub server_port: u16,
    pub auth_method: AuthMethodInput,
    pub auth_token: Option<String>,
    pub oidc_client_id: Option<String>,
    pub oidc_client_secret: Option<String>,
    pub oidc_audience: Option<String>,
    pub oidc_token_endpoint_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AddProxyInput {
    pub name: String,
    pub proxy_type: ProxyType,
    pub local_ip: Option<String>,
    pub local_port: Option<u16>,
    pub remote_port: Option<u16>,
    pub subdomain: Option<String>,
    pub custom_domains: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AuthMethodInput {
    Token,
    Oidc,
}

#[tauri::command]
pub async fn list_profiles(state: State<'_, AppState>) -> AppResult<Vec<ProfileSummary>> {
    let store = state.profile_store();
    let mut registry = state.registry.write().await;
    list_profiles_with_runtime_state(&store, &mut registry)
}

#[tauri::command]
pub async fn get_profile(state: State<'_, AppState>, profile_id: String) -> AppResult<Profile> {
    state.profile_store().load(&profile_id)
}

#[tauri::command]
pub async fn import_profile_from_text(
    app: AppHandle,
    state: State<'_, AppState>,
    name: String,
    toml: String,
) -> AppResult<ProfileSummary> {
    let result = state.profile_store().import_from_text(&name, &toml);
    sync_profile_state(&app).await;
    result
}

#[tauri::command]
pub async fn create_profile(
    app: AppHandle,
    state: State<'_, AppState>,
    input: CreateProfileInput,
) -> AppResult<ProfileSummary> {
    let result = (|| {
        let profile_name = required_trimmed(&input.profile_name, "profileName")?.to_string();
        let toml = create_profile_toml(&input)?;
        state.profile_store().import_from_text(&profile_name, &toml)
    })();
    sync_profile_state(&app).await;
    result
}

#[tauri::command]
pub async fn update_profile(
    app: AppHandle,
    state: State<'_, AppState>,
    profile_id: String,
    input: CreateProfileInput,
) -> AppResult<()> {
    let result = async {
        let profile_name = required_trimmed(&input.profile_name, "profileName")?.to_string();
        let store = state.profile_store();
        let path = store.profile_toml_path(&profile_id)?;
        let raw = fs::read_to_string(&path)?;
        let updated = update_profile_toml(&raw, &input)?;
        store.save_raw_toml(&profile_id, &updated)?;
        store.update_display_name(&profile_id, &profile_name)?;
        restart_profile_if_running(&state, &profile_id).await
    }
    .await;
    sync_profile_state(&app).await;
    result
}

#[tauri::command]
pub async fn delete_profile(
    app: AppHandle,
    state: State<'_, AppState>,
    profile_id: String,
) -> AppResult<()> {
    let result = async {
        {
            let mut registry = state.registry.write().await;
            let snapshot = registry.refresh_snapshot(&profile_id)?;
            if !matches!(
                snapshot.state,
                ProfileProcessState::Stopped | ProfileProcessState::Failed
            ) {
                registry.stop(&profile_id).await?;
            }
        }

        state.profile_store().delete(&profile_id)
    }
    .await;
    sync_profile_state(&app).await;
    result
}

#[tauri::command]
pub async fn save_profile(
    app: AppHandle,
    state: State<'_, AppState>,
    profile_id: String,
    raw_toml: String,
) -> AppResult<()> {
    let result = state.profile_store().save_raw_toml(&profile_id, &raw_toml);
    sync_profile_state(&app).await;
    result
}

#[tauri::command]
pub async fn start_profile(
    app: AppHandle,
    state: State<'_, AppState>,
    profile_id: String,
) -> AppResult<()> {
    let result = start_profile_by_id(&state, &profile_id).await;
    sync_profile_state(&app).await;
    result
}

pub async fn start_profile_by_id(state: &AppState, profile_id: &str) -> AppResult<()> {
    let launch = profile_launch_config(state, profile_id)?;
    state
        .registry
        .write()
        .await
        .start(
            profile_id,
            &launch.executable,
            &launch.config_path,
            &launch.working_dir,
        )
        .await
}

#[tauri::command]
pub async fn stop_profile(
    app: AppHandle,
    state: State<'_, AppState>,
    profile_id: String,
) -> AppResult<()> {
    let result = stop_profile_by_id(&state, &profile_id).await;
    sync_profile_state(&app).await;
    result
}

pub async fn stop_profile_by_id(state: &AppState, profile_id: &str) -> AppResult<()> {
    state.registry.write().await.stop(profile_id).await
}

#[tauri::command]
pub async fn toggle_proxy(
    app: AppHandle,
    state: State<'_, AppState>,
    profile_id: String,
    proxy_name: String,
    enabled: bool,
) -> AppResult<()> {
    let result = toggle_proxy_by_name(&state, &profile_id, &proxy_name, enabled).await;
    sync_profile_state(&app).await;
    result
}

pub async fn toggle_proxy_by_name(
    state: &AppState,
    profile_id: &str,
    proxy_name: &str,
    enabled: bool,
) -> AppResult<()> {
    let store = state.profile_store();
    let path = store.profile_toml_path(profile_id)?;
    let raw = fs::read_to_string(&path)?;
    let updated = set_proxy_enabled(&raw, proxy_name, enabled)?;
    store.save_raw_toml(profile_id, &updated)?;
    restart_profile_if_running(state, profile_id).await
}

#[tauri::command]
pub async fn add_proxy(
    app: AppHandle,
    state: State<'_, AppState>,
    profile_id: String,
    input: AddProxyInput,
) -> AppResult<()> {
    let result = async {
        let store = state.profile_store();
        let path = store.profile_toml_path(&profile_id)?;
        let raw = fs::read_to_string(&path)?;
        let updated = append_proxy_toml(&raw, &input)?;
        store.save_raw_toml(&profile_id, &updated)?;
        restart_profile_if_running(&state, &profile_id).await
    }
    .await;
    sync_profile_state(&app).await;
    result
}

#[tauri::command]
pub async fn update_proxy(
    app: AppHandle,
    state: State<'_, AppState>,
    profile_id: String,
    proxy_name: String,
    input: AddProxyInput,
) -> AppResult<()> {
    let result = async {
        let store = state.profile_store();
        let path = store.profile_toml_path(&profile_id)?;
        let raw = fs::read_to_string(&path)?;
        let updated = update_proxy_toml(&raw, &proxy_name, &input)?;
        store.save_raw_toml(&profile_id, &updated)?;
        restart_profile_if_running(&state, &profile_id).await
    }
    .await;
    sync_profile_state(&app).await;
    result
}

#[tauri::command]
pub async fn delete_proxy(
    app: AppHandle,
    state: State<'_, AppState>,
    profile_id: String,
    proxy_name: String,
) -> AppResult<()> {
    let result = async {
        let store = state.profile_store();
        let path = store.profile_toml_path(&profile_id)?;
        let raw = fs::read_to_string(&path)?;
        let updated = delete_proxy_toml(&raw, &proxy_name)?;
        store.save_raw_toml(&profile_id, &updated)?;
        restart_profile_if_running(&state, &profile_id).await
    }
    .await;
    sync_profile_state(&app).await;
    result
}

#[tauri::command]
pub async fn read_profile_logs(
    state: State<'_, AppState>,
    profile_id: String,
) -> AppResult<String> {
    let log_path = profile_log_path(&state.data_dir, &profile_id)?;
    if !log_path.exists() {
        return Ok(String::new());
    }
    Ok(clean_log_output(&fs::read_to_string(log_path)?))
}

#[tauri::command]
pub async fn get_runtime_info(state: State<'_, AppState>) -> AppResult<String> {
    state.runtime_manager().current_runtime_version()
}

#[tauri::command]
pub async fn check_runtime_update(state: State<'_, AppState>) -> AppResult<RuntimeUpdateCheck> {
    let current_version = state.runtime_manager().current_runtime_version()?;
    let release = fetch_latest_release().await?;
    let latest_version = release.tag_name.trim_start_matches('v').to_string();
    let (os, arch) = current_platform();
    let asset = select_platform_asset(&latest_version, os, arch, &release.assets)?;

    Ok(RuntimeUpdateCheck {
        update_available: latest_version != current_version,
        current_version,
        latest_version,
        asset_name: Some(asset.name),
    })
}

pub fn create_profile_toml(input: &CreateProfileInput) -> AppResult<String> {
    let _profile_name = required_trimmed(&input.profile_name, "profileName")?;
    let server_addr = required_trimmed(&input.server_addr, "serverAddr")?;

    let mut doc = DocumentMut::new();
    doc["serverAddr"] = value(server_addr);
    doc["serverPort"] = value(i64::from(input.server_port));
    set_auth_table(&mut doc, input)?;

    Ok(doc.to_string())
}

pub fn update_profile_toml(input: &str, update: &CreateProfileInput) -> AppResult<String> {
    let _profile_name = required_trimmed(&update.profile_name, "profileName")?;
    let server_addr = required_trimmed(&update.server_addr, "serverAddr")?;
    let mut doc = input
        .parse::<DocumentMut>()
        .map_err(|err| AppError::TomlParse(err.to_string()))?;
    doc["serverAddr"] = value(server_addr);
    doc["serverPort"] = value(i64::from(update.server_port));
    set_auth_table(&mut doc, update)?;
    let updated = doc.to_string();
    parse_profile_toml("validation", "validation", &updated)?;
    Ok(updated)
}

pub fn append_proxy_toml(input: &str, proxy: &AddProxyInput) -> AppResult<String> {
    let mut doc = input
        .parse::<DocumentMut>()
        .map_err(|err| AppError::TomlParse(err.to_string()))?;
    if !doc.contains_key("proxies") {
        doc["proxies"] = Item::ArrayOfTables(ArrayOfTables::new());
    }
    let proxy_array = doc["proxies"]
        .as_array_of_tables_mut()
        .ok_or_else(|| AppError::Validation("proxies must be an array of tables".into()))?;

    proxy_array.push(proxy_table(proxy, true)?);
    let updated = doc.to_string();
    parse_profile_toml("validation", "validation", &updated)?;
    Ok(updated)
}

pub fn update_proxy_toml(
    input: &str,
    proxy_name: &str,
    proxy: &AddProxyInput,
) -> AppResult<String> {
    let mut doc = input
        .parse::<DocumentMut>()
        .map_err(|err| AppError::TomlParse(err.to_string()))?;
    let proxy_item = doc
        .get_mut("proxies")
        .ok_or_else(|| AppError::Validation("profile has no proxies".into()))?;
    let proxy_array = proxy_item
        .as_array_of_tables_mut()
        .ok_or_else(|| AppError::Validation("proxies must be an array of tables".into()))?;

    let mut found = false;
    for table in proxy_array.iter_mut() {
        if table.get("name").and_then(|item| item.as_str()) == Some(proxy_name) {
            let enabled = table
                .get("enabled")
                .and_then(|item| item.as_bool())
                .unwrap_or(true);
            *table = proxy_table(proxy, enabled)?;
            found = true;
            break;
        }
    }

    if !found {
        return Err(AppError::Validation(format!(
            "proxy {proxy_name} not found"
        )));
    }

    let updated = doc.to_string();
    parse_profile_toml("validation", "validation", &updated)?;
    Ok(updated)
}

pub fn delete_proxy_toml(input: &str, proxy_name: &str) -> AppResult<String> {
    let mut doc = input
        .parse::<DocumentMut>()
        .map_err(|err| AppError::TomlParse(err.to_string()))?;
    let proxy_item = doc
        .get_mut("proxies")
        .ok_or_else(|| AppError::Validation("profile has no proxies".into()))?;
    let proxy_array = proxy_item
        .as_array_of_tables_mut()
        .ok_or_else(|| AppError::Validation("proxies must be an array of tables".into()))?;

    let Some(index) = proxy_array
        .iter()
        .position(|table| table.get("name").and_then(|item| item.as_str()) == Some(proxy_name))
    else {
        return Err(AppError::Validation(format!(
            "proxy {proxy_name} not found"
        )));
    };
    proxy_array.remove(index);

    let updated = doc.to_string();
    parse_profile_toml("validation", "validation", &updated)?;
    Ok(updated)
}

pub fn list_profiles_with_runtime_state(
    store: &ProfileStore,
    registry: &mut ProcessRegistry,
) -> AppResult<Vec<ProfileSummary>> {
    let mut profiles = store.list()?;
    for profile in &mut profiles {
        let snapshot = registry.refresh_snapshot(&profile.id)?;
        profile.runtime_state = match snapshot.state {
            ProfileProcessState::Stopped => RuntimeState::Stopped,
            ProfileProcessState::Starting => RuntimeState::Starting,
            ProfileProcessState::Running => RuntimeState::Running,
            ProfileProcessState::Reloading => RuntimeState::Reloading,
            ProfileProcessState::Degraded => RuntimeState::Degraded,
            ProfileProcessState::Failed => RuntimeState::Failed,
        };
        profile.runtime_pid = snapshot.pid;
        profile.runtime_started_at = snapshot.started_at;
    }
    Ok(profiles)
}

pub fn profile_log_path(data_dir: &Path, profile_id: &str) -> AppResult<PathBuf> {
    let store = ProfileStore::new(data_dir.to_path_buf());
    store.load(profile_id)?;
    Ok(data_dir
        .join("profiles")
        .join(profile_id)
        .join("logs")
        .join("current.log"))
}

pub fn clean_log_output(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch != '\u{1b}' {
            output.push(ch);
            continue;
        }

        match chars.peek().copied() {
            Some('[') => {
                chars.next();
                for code in chars.by_ref() {
                    if ('@'..='~').contains(&code) {
                        break;
                    }
                }
            }
            _ => {}
        }
    }

    output
}

pub fn bundled_frpc_fallback_path(current_exe: &Path) -> AppResult<PathBuf> {
    let parent = current_exe.parent().ok_or_else(|| {
        crate::error::AppError::Runtime("current executable has no parent".into())
    })?;
    Ok(parent.join("frpc"))
}

struct ProfileLaunchConfig {
    executable: PathBuf,
    config_path: PathBuf,
    working_dir: PathBuf,
}

fn profile_launch_config(state: &AppState, profile_id: &str) -> AppResult<ProfileLaunchConfig> {
    let store = state.profile_store();
    store.load(profile_id)?;
    let config_path = store.profile_toml_path(profile_id)?;
    let working_dir = config_path
        .parent()
        .expect("profile TOML has parent directory")
        .to_path_buf();
    let runtime = state.runtime_manager();
    let (os, arch) = current_platform();
    let frpc_path = runtime.runtime_info_for_version("0.69.1", os, arch).path;
    let fallback_sidecar = bundled_frpc_fallback_path(&std::env::current_exe()?)?;
    let executable = if frpc_path.exists() {
        frpc_path
    } else {
        fallback_sidecar
    };

    Ok(ProfileLaunchConfig {
        executable,
        config_path,
        working_dir,
    })
}

fn required_trimmed<'a>(value: &'a str, label: &str) -> AppResult<&'a str> {
    let value = value.trim();
    if value.is_empty() {
        Err(AppError::Validation(format!("{label} is required")))
    } else {
        Ok(value)
    }
}

fn required_optional_trimmed<'a>(value: &'a Option<String>, label: &str) -> AppResult<&'a str> {
    match value {
        Some(value) => required_trimmed(value, label),
        None => Err(AppError::Validation(format!("{label} is required"))),
    }
}

fn optional_trimmed(value: &Option<String>) -> Option<&str> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn proxy_type_name(proxy_type: &ProxyType) -> &'static str {
    match proxy_type {
        ProxyType::Http => "http",
        ProxyType::Https => "https",
        ProxyType::Tcp => "tcp",
        ProxyType::Udp => "udp",
    }
}

fn proxy_table(proxy: &AddProxyInput, enabled: bool) -> AppResult<Table> {
    let name = required_trimmed(&proxy.name, "proxyName")?;
    let local_ip = optional_trimmed(&proxy.local_ip).unwrap_or("127.0.0.1");
    let local_port = proxy
        .local_port
        .ok_or_else(|| AppError::Validation("localPort is required".into()))?;

    if matches!(proxy.proxy_type, ProxyType::Tcp | ProxyType::Udp) && proxy.remote_port.is_none() {
        return Err(AppError::Validation(
            "remotePort is required for tcp and udp proxies".into(),
        ));
    }

    let mut table = Table::new();
    table["name"] = value(name);
    table["type"] = value(proxy_type_name(&proxy.proxy_type));
    table["enabled"] = value(enabled);
    table["localIP"] = value(local_ip);
    table["localPort"] = value(i64::from(local_port));

    if let Some(remote_port) = proxy.remote_port {
        table["remotePort"] = value(i64::from(remote_port));
    }
    if let Some(subdomain) = optional_trimmed(&proxy.subdomain) {
        table["subdomain"] = value(subdomain);
    }
    let custom_domains = proxy
        .custom_domains
        .iter()
        .map(|domain| domain.trim())
        .filter(|domain| !domain.is_empty())
        .collect::<Vec<_>>();
    if !custom_domains.is_empty() {
        let mut array = toml_edit::Array::new();
        for domain in custom_domains {
            array.push(domain);
        }
        table["customDomains"] = value(array);
    }

    Ok(table)
}

fn set_auth_table(doc: &mut DocumentMut, input: &CreateProfileInput) -> AppResult<()> {
    doc["auth"] = Item::Table(Table::new());

    match input.auth_method {
        AuthMethodInput::Token => {
            doc["auth"]["method"] = value("token");
            if let Some(token) = optional_trimmed(&input.auth_token) {
                doc["auth"]["token"] = value(token);
            }
        }
        AuthMethodInput::Oidc => {
            doc["auth"]["method"] = value("oidc");
            doc["auth"]["oidc"] = Item::Table(Table::new());
            doc["auth"]["oidc"]["clientID"] = value(required_optional_trimmed(
                &input.oidc_client_id,
                "oidcClientId",
            )?);
            doc["auth"]["oidc"]["clientSecret"] = value(required_optional_trimmed(
                &input.oidc_client_secret,
                "oidcClientSecret",
            )?);
            doc["auth"]["oidc"]["audience"] = value(required_optional_trimmed(
                &input.oidc_audience,
                "oidcAudience",
            )?);
            doc["auth"]["oidc"]["tokenEndpointURL"] = value(required_optional_trimmed(
                &input.oidc_token_endpoint_url,
                "oidcTokenEndpointUrl",
            )?);
        }
    }

    Ok(())
}

async fn restart_profile_if_running(state: &AppState, profile_id: &str) -> AppResult<()> {
    let mut registry = state.registry.write().await;
    let snapshot = registry.refresh_snapshot(profile_id)?;
    if snapshot.state != ProfileProcessState::Running {
        return Ok(());
    }

    let launch = profile_launch_config(state, profile_id)?;
    registry
        .restart(
            profile_id,
            &launch.executable,
            &launch.config_path,
            &launch.working_dir,
        )
        .await
}

async fn sync_profile_state(app: &AppHandle) {
    crate::tray::sync_profile_state(app).await;
}
