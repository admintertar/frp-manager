pub mod app_state;
pub mod commands;
pub mod config_toml;
pub mod diagnostics;
pub mod error;
pub mod github_release;
pub mod models;
pub mod process_manager;
pub mod profile_store;
pub mod runtime_manager;
pub mod tray;

use app_state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data directory");
            std::fs::create_dir_all(&app_data_dir)?;
            app.manage(AppState::new(app_data_dir));
            tray::build(app.handle())?;
            if let Some(window) = app.get_webview_window("main") {
                let window_clone = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = window_clone.hide();
                        #[cfg(target_os = "macos")]
                        let _ = window_clone
                            .app_handle()
                            .set_activation_policy(tauri::ActivationPolicy::Accessory);
                    }
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_profiles,
            commands::get_profile,
            commands::import_profile_from_text,
            commands::create_profile,
            commands::update_profile,
            commands::delete_profile,
            commands::save_profile,
            commands::start_profile,
            commands::stop_profile,
            commands::toggle_proxy,
            commands::add_proxy,
            commands::update_proxy,
            commands::delete_proxy,
            commands::read_app_log,
            commands::read_profile_logs,
            commands::get_runtime_info,
            commands::get_runtime_status,
            commands::check_runtime_update,
            commands::install_runtime,
            commands::check_app_update,
            commands::download_app_update,
            commands::open_app_update_installer
        ])
        .build(tauri::generate_context!())
        .expect("error while running FRP Manager");

    app.run(|app, event| match event {
        tauri::RunEvent::ExitRequested { api, code, .. } => {
            if !app.state::<AppState>().inner().is_exiting() {
                api.prevent_exit();
                tray::quit_app(app, code.unwrap_or(0));
            }
        }
        #[cfg(target_os = "macos")]
        tauri::RunEvent::Reopen {
            has_visible_windows: false,
            ..
        } => {
            tray::show_main_window(app);
        }
        _ => {}
    });
}
