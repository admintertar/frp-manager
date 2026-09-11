use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager};

use crate::app_state::AppState;
use crate::commands::{list_profiles_with_runtime_state, start_profile_by_id, stop_profile_by_id};
use crate::models::{Profile, ProfileSummary, ProxyConfig, ProxyType, RuntimeState};
use crate::profile_store::ProfileStore;
use crate::settings::Locale;

const TRAY_ID: &str = "frp-manager";
pub const PROFILE_STATE_CHANGED_EVENT: &str = "profile-state-changed";
pub const APP_UPDATE_CHECK_REQUESTED_EVENT: &str = "app-update-check-requested";

pub fn build(app: &AppHandle) -> tauri::Result<()> {
    let state = app.state::<AppState>().inner().clone();
    let menu = build_menu(app, load_stored_tray_profiles(&state))?;

    TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("FRP Manager")
        .icon(
            app.default_window_icon()
                .expect("window icon is configured")
                .clone(),
        )
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| handle_menu_event(app, event.id().as_ref()))
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

pub async fn refresh_menu(app: &AppHandle) -> tauri::Result<()> {
    let state = app.state::<AppState>().inner().clone();
    let menu = build_menu(app, load_tray_profiles(&state).await)?;
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        tray.set_menu(Some(menu))?;
    }
    Ok(())
}

pub async fn sync_profile_state(app: &AppHandle) {
    let _ = refresh_menu(app).await;
    let _ = app.emit(PROFILE_STATE_CHANGED_EVENT, ());
}

pub fn quit_app(app: &AppHandle, exit_code: i32) {
    let state = app.state::<AppState>().inner().clone();
    if !state.request_exit() {
        return;
    }

    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(err) = state.registry.write().await.stop_all().await {
            eprintln!("failed to stop frpc processes before exit: {err}");
        }
        app.exit(exit_code);
    });
}

pub fn show_main_window(app: &AppHandle) {
    #[cfg(target_os = "macos")]
    let _ = app.set_activation_policy(tauri::ActivationPolicy::Regular);

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
    let _ = app.emit(PROFILE_STATE_CHANGED_EVENT, ());
}

/// Localized tray labels. `{...}` placeholders are filled by [`fill`].
struct TrayText {
    running: &'static str,
    runtime: &'static str,
    profiles: &'static str,
    no_profiles: &'static str,
    open: &'static str,
    check_update: &'static str,
    quit: &'static str,
    start: &'static str,
    stop: &'static str,
    no_proxies: &'static str,
    unknown: &'static str,
}

fn tray_text(locale: Locale) -> TrayText {
    match locale {
        Locale::ZhCn => TrayText {
            running: "{count} 个运行中",
            runtime: "frpc 运行时 {version}",
            profiles: "配置",
            no_profiles: "尚未配置任何配置",
            open: "打开主窗口",
            check_update: "检查 FRP Manager 更新",
            quit: "退出 FRP Manager",
            start: "启动",
            stop: "停止",
            no_proxies: "无映射",
            unknown: "未知",
        },
        Locale::En => TrayText {
            running: "{count} running",
            runtime: "frpc runtime {version}",
            profiles: "Profiles",
            no_profiles: "No profiles configured",
            open: "Open Main Window",
            check_update: "Check FRP Manager Update",
            quit: "Quit FRP Manager",
            start: "Start",
            stop: "Stop",
            no_proxies: "No proxies",
            unknown: "unknown",
        },
    }
}

fn fill(template: &str, pairs: &[(&str, String)]) -> String {
    pairs.iter().fold(template.to_string(), |text, (key, value)| {
        text.replace(&format!("{{{key}}}"), value)
    })
}

fn build_menu(app: &AppHandle, profiles: Vec<TrayProfile>) -> tauri::Result<Menu<tauri::Wry>> {
    let state = app.state::<AppState>().inner().clone();
    let text = tray_text(state.locale());
    let runtime_version = state
        .runtime_manager()
        .current_runtime_version()
        .unwrap_or_else(|_| text.unknown.to_string());
    let running_count = profiles
        .iter()
        .filter(|profile| profile.summary.runtime_state == RuntimeState::Running)
        .count();

    let menu = Menu::new(app)?;
    append_disabled_text(
        &menu,
        app,
        "tray-header",
        &format!(
            "FRP Manager    {}",
            fill(text.running, &[("count", running_count.to_string())])
        ),
    )?;
    append_disabled_text(
        &menu,
        app,
        "tray-runtime",
        &fill(text.runtime, &[("version", runtime_version)]),
    )?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;

    append_disabled_text(&menu, app, "tray-profiles-label", text.profiles)?;
    if profiles.is_empty() {
        append_disabled_text(&menu, app, "tray-empty-profiles", text.no_profiles)?;
    } else {
        for profile in profiles {
            let submenu = build_profile_menu(app, &profile)?;
            menu.append(&submenu)?;
        }
    }

    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&MenuItem::with_id(
        app,
        "open",
        text.open,
        true,
        Some("CmdOrCtrl+O"),
    )?)?;
    menu.append(&MenuItem::with_id(
        app,
        "check_update",
        text.check_update,
        true,
        None::<&str>,
    )?)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&MenuItem::with_id(
        app,
        "quit",
        text.quit,
        true,
        Some("CmdOrCtrl+Q"),
    )?)?;

    Ok(menu)
}

fn build_profile_menu(
    app: &AppHandle,
    tray_profile: &TrayProfile,
) -> tauri::Result<Submenu<tauri::Wry>> {
    let state = app.state::<AppState>().inner().clone();
    let locale = state.locale();
    let text = tray_text(locale);
    let summary = &tray_profile.summary;
    let state_label = state_label(locale, &summary.runtime_state);
    let submenu = Submenu::with_id(
        app,
        format!("profile:{}", encode_menu_id_segment(&summary.id)),
        format!("{} - {state_label}", summary.display_name),
        true,
    )?;

    submenu.append(&MenuItem::with_id(
        app,
        format!("profile-info:{}", encode_menu_id_segment(&summary.id)),
        format!("{} · {state_label}", summary.server_port),
        false,
        None::<&str>,
    )?)?;

    if summary.runtime_state == RuntimeState::Running {
        submenu.append(&MenuItem::with_id(
            app,
            format!("stop-profile:{}", encode_menu_id_segment(&summary.id)),
            text.stop,
            true,
            None::<&str>,
        )?)?;
    } else {
        submenu.append(&MenuItem::with_id(
            app,
            format!("start-profile:{}", encode_menu_id_segment(&summary.id)),
            text.start,
            true,
            None::<&str>,
        )?)?;
    }

    submenu.append(&PredefinedMenuItem::separator(app)?)?;
    if tray_profile.profile.proxies.is_empty() {
        submenu.append(&MenuItem::with_id(
            app,
            format!("empty-proxies:{}", encode_menu_id_segment(&summary.id)),
            text.no_proxies,
            false,
            None::<&str>,
        )?)?;
    } else {
        for proxy in &tray_profile.profile.proxies {
            submenu.append(&build_proxy_item(app, &summary.id, proxy)?)?;
        }
    }

    Ok(submenu)
}

fn build_proxy_item(
    app: &AppHandle,
    profile_id: &str,
    proxy: &ProxyConfig,
) -> tauri::Result<CheckMenuItem<tauri::Wry>> {
    let target_state = if proxy.enabled { "off" } else { "on" };
    let profile_id = encode_menu_id_segment(profile_id);
    let proxy_name = encode_menu_id_segment(&proxy.name);
    CheckMenuItem::with_id(
        app,
        format!("toggle-proxy:{profile_id}:{proxy_name}:{target_state}"),
        proxy_menu_text(proxy),
        true,
        proxy.enabled,
        None::<&str>,
    )
}

fn append_disabled_text(
    menu: &Menu<tauri::Wry>,
    app: &AppHandle,
    id: &str,
    text: &str,
) -> tauri::Result<()> {
    menu.append(&MenuItem::with_id(app, id, text, false, None::<&str>)?)
}

async fn load_tray_profiles(state: &AppState) -> Vec<TrayProfile> {
    let store = state.profile_store();
    let summaries = {
        let mut registry = state.registry.write().await;
        list_profiles_with_runtime_state(&store, &mut registry).unwrap_or_default()
    };

    hydrate_tray_profiles(&store, summaries)
}

fn load_stored_tray_profiles(state: &AppState) -> Vec<TrayProfile> {
    let store = state.profile_store();
    let summaries = store.list().unwrap_or_default();
    hydrate_tray_profiles(&store, summaries)
}

fn hydrate_tray_profiles(store: &ProfileStore, summaries: Vec<ProfileSummary>) -> Vec<TrayProfile> {
    summaries
        .into_iter()
        .filter_map(|summary| {
            let profile = store.load(&summary.id).ok()?;
            Some(TrayProfile { summary, profile })
        })
        .collect()
}

fn handle_menu_event(app: &AppHandle, id: &str) {
    match id {
        "open" => show_main_window(app),
        "check_update" => {
            show_main_window(app);
            let _ = app.emit(APP_UPDATE_CHECK_REQUESTED_EVENT, ());
        }
        "quit" => quit_app(app, 0),
        _ if id.starts_with("start-profile:") => {
            if let Some(profile_id) =
                decode_menu_id_segment(id.trim_start_matches("start-profile:"))
            {
                run_profile_task(app, move |state| async move {
                    start_profile_by_id(&state, &profile_id).await
                });
            }
        }
        _ if id.starts_with("stop-profile:") => {
            if let Some(profile_id) = decode_menu_id_segment(id.trim_start_matches("stop-profile:"))
            {
                run_profile_task(app, move |state| async move {
                    stop_profile_by_id(&state, &profile_id).await
                });
            }
        }
        _ if id.starts_with("toggle-proxy:") => {
            if let Some((profile_id, proxy_name, enabled)) = parse_toggle_proxy_id(id) {
                run_profile_task(app, move |state| async move {
                    crate::commands::toggle_proxy_by_name(&state, &profile_id, &proxy_name, enabled)
                        .await
                });
            }
        }
        _ => {}
    }
}

fn run_profile_task<F, Fut>(app: &AppHandle, task: F)
where
    F: FnOnce(AppState) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = crate::error::AppResult<()>> + Send + 'static,
{
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let state = app.state::<AppState>().inner().clone();
        if let Err(err) = task(state).await {
            eprintln!("tray action failed: {err}");
        }
        sync_profile_state(&app).await;
    });
}

fn parse_toggle_proxy_id(id: &str) -> Option<(String, String, bool)> {
    let rest = id.strip_prefix("toggle-proxy:")?;
    let (profile_and_proxy, target) = rest.rsplit_once(':')?;
    let (profile_id, proxy_name) = profile_and_proxy.split_once(':')?;
    let enabled = match target {
        "on" => true,
        "off" => false,
        _ => return None,
    };
    Some((
        decode_menu_id_segment(profile_id)?,
        decode_menu_id_segment(proxy_name)?,
        enabled,
    ))
}

fn encode_menu_id_segment(segment: &str) -> String {
    segment.replace('%', "%25").replace(':', "%3A")
}

fn decode_menu_id_segment(segment: &str) -> Option<String> {
    let mut decoded = String::with_capacity(segment.len());
    let mut chars = segment.chars();

    while let Some(ch) = chars.next() {
        if ch != '%' {
            decoded.push(ch);
            continue;
        }

        let first = chars.next()?;
        let second = chars.next()?;
        match (first, second) {
            ('2', '5') => decoded.push('%'),
            ('3', 'A') | ('3', 'a') => decoded.push(':'),
            _ => return None,
        }
    }

    Some(decoded)
}

fn state_label(locale: Locale, state: &RuntimeState) -> &'static str {
    match (locale, state) {
        (Locale::ZhCn, RuntimeState::Stopped) => "已停止",
        (Locale::ZhCn, RuntimeState::Starting) => "启动中",
        (Locale::ZhCn, RuntimeState::Running) => "运行中",
        (Locale::ZhCn, RuntimeState::Reloading) => "重载中",
        (Locale::ZhCn, RuntimeState::Degraded) => "降级",
        (Locale::ZhCn, RuntimeState::Failed) => "失败",
        (Locale::En, RuntimeState::Stopped) => "stopped",
        (Locale::En, RuntimeState::Starting) => "starting",
        (Locale::En, RuntimeState::Running) => "running",
        (Locale::En, RuntimeState::Reloading) => "reloading",
        (Locale::En, RuntimeState::Degraded) => "degraded",
        (Locale::En, RuntimeState::Failed) => "failed",
    }
}

fn proxy_type_label(proxy_type: &ProxyType) -> &'static str {
    match proxy_type {
        ProxyType::Http => "http",
        ProxyType::Https => "https",
        ProxyType::Tcp => "tcp",
        ProxyType::Udp => "udp",
    }
}

fn proxy_menu_text(proxy: &ProxyConfig) -> String {
    let port = proxy
        .remote_port
        .or(proxy.local_port)
        .map(|port| port.to_string())
        .unwrap_or_else(|| "-".to_string());
    format!(
        "{} · {} · {port}",
        proxy.name,
        proxy_type_label(&proxy.proxy_type)
    )
}

struct TrayProfile {
    summary: ProfileSummary,
    profile: Profile,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_encoded_proxy_toggle_menu_id() {
        let id = format!(
            "toggle-proxy:{}:{}:off",
            encode_menu_id_segment("profile:one"),
            encode_menu_id_segment("api%proxy:blue")
        );

        assert_eq!(
            parse_toggle_proxy_id(&id),
            Some((
                "profile:one".to_string(),
                "api%proxy:blue".to_string(),
                false
            ))
        );
    }

    #[test]
    fn fill_substitutes_every_placeholder() {
        assert_eq!(
            fill(
                "{count} running · {version}",
                &[("count", "2".to_string()), ("version", "0.69.1".to_string())]
            ),
            "2 running · 0.69.1"
        );
    }

    #[test]
    fn fill_leaves_unknown_placeholders_alone() {
        assert_eq!(fill("{missing}", &[("count", "1".to_string())]), "{missing}");
    }

    #[test]
    fn state_labels_are_translated_per_locale() {
        assert_eq!(state_label(Locale::En, &RuntimeState::Running), "running");
        assert_eq!(state_label(Locale::ZhCn, &RuntimeState::Running), "运行中");
        assert_eq!(state_label(Locale::En, &RuntimeState::Stopped), "stopped");
        assert_eq!(state_label(Locale::ZhCn, &RuntimeState::Stopped), "已停止");
    }

    #[test]
    fn every_runtime_state_has_a_label_in_both_locales() {
        for state in [
            RuntimeState::Stopped,
            RuntimeState::Starting,
            RuntimeState::Running,
            RuntimeState::Reloading,
            RuntimeState::Degraded,
            RuntimeState::Failed,
        ] {
            for locale in [Locale::En, Locale::ZhCn] {
                assert!(!state_label(locale, &state).is_empty());
            }
        }
    }

    #[test]
    fn tray_text_differs_between_locales() {
        let en = tray_text(Locale::En);
        let zh = tray_text(Locale::ZhCn);

        assert_eq!(en.profiles, "Profiles");
        assert_eq!(zh.profiles, "配置");
        assert_ne!(en.running, zh.running);
        assert_ne!(en.quit, zh.quit);
    }
}
