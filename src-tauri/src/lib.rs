mod storage;
mod settings;
mod keychain;
mod hotkey;
mod clipboard;
mod focus;
mod audio;
mod whisper;
mod prompt;
mod commands;

use std::sync::{Arc, Mutex};
use commands::settings::AppState;
use keychain::macos::KeychainManager;
use settings::manager::SettingsManager;
use storage::database::Database;
use clipboard::manager::ClipboardManager;
use audio::recorder::AudioRecorder;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
    Emitter,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db_path = dirs::data_dir()
        .unwrap()
        .join("com.prompter.app")
        .join("prompter.db");
    std::fs::create_dir_all(db_path.parent().unwrap()).unwrap();

    let db = Database::new(db_path.to_str().unwrap()).unwrap();
    let db2 = Database::new(db_path.to_str().unwrap()).unwrap();
    let settings_manager = SettingsManager::new(db);
    let keychain = KeychainManager::new("com.prompter.app");
    let clipboard_manager = ClipboardManager::new();
    let audio_recorder = Arc::new(AudioRecorder::new().unwrap());

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(AppState {
            settings_manager: Mutex::new(settings_manager),
            keychain,
            clipboard: Mutex::new(clipboard_manager),
            audio_recorder,
            db: Mutex::new(db2),
        })
        .invoke_handler(tauri::generate_handler![
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::settings::store_api_key,
            commands::settings::get_api_key,
            commands::settings::delete_api_key,
            commands::recording::start_recording,
            commands::recording::stop_recording,
            commands::prompt::generate_prompt,
            commands::prompt::paste_to_app,
            commands::prompt::get_prompt_history,
        ])
        .setup(|app| {
            // Build tray menu
            let start = MenuItem::with_id(app, "start", "Start Recording", true, None::<&str>)?;
            let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

            let menu = Menu::with_items(app, &[&start, &settings_item, &quit])?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .icon(app.default_window_icon().unwrap().clone())
                .on_menu_event(move |app, event| {
                    match event.id.as_ref() {
                        "start" => {
                            let _ = app.emit("hotkey-triggered", ());
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "settings" => {
                            let _ = app.emit("open-settings", ());
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            // Register global hotkey from settings
            let app_handle = app.handle().clone();
            let settings_state = app.state::<AppState>();
            let hotkey_str = settings_state.settings_manager.lock().unwrap()
                .load().unwrap_or_default().global_hotkey;

            use tauri_plugin_global_shortcut::GlobalShortcutExt;
            if let Ok(shortcut) = hotkey_str.parse::<tauri_plugin_global_shortcut::Shortcut>() {
                let _ = app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, event| {
                    if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        let _ = app_handle.emit("hotkey-triggered", ());
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                });
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
