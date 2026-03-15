mod audio;
mod clipboard;
mod commands;
mod focus;
mod hotkey;
mod keychain;
pub mod prompt;
pub mod settings;
pub mod storage;
mod whisper;

use audio::recorder::AudioRecorder;
use clipboard::manager::ClipboardManager;
use commands::settings::{AppState, CapturedContext};
use keychain::macos::KeychainManager;
use settings::manager::SettingsManager;
use std::sync::{Arc, Mutex};
use storage::database::Database;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager,
};

/// Capture selected text from the frontmost app BEFORE showing the Prompter window.
/// This must run while the source app still has focus.
fn capture_context(app_handle: &tauri::AppHandle) {
    let state = app_handle.state::<AppState>();

    // 1. Get frontmost app (the source app, not Prompter)
    let source_app = focus::tracker::FocusTracker::get_frontmost_app().ok();
    eprintln!(
        "[capture] frontmost app: {:?}",
        source_app.as_ref().map(|a| &a.name)
    );

    // 2. Save current clipboard
    let _ = state.clipboard.lock().unwrap().save_current();

    // 3. Simulate Cmd+C to copy selection from the source app
    let _ = std::process::Command::new("osascript")
        .arg("-e")
        .arg(r#"tell application "System Events" to keystroke "c" using command down"#)
        .output();

    std::thread::sleep(std::time::Duration::from_millis(150));

    // 4. Read clipboard
    let selected_text = state.clipboard.lock().unwrap().read().unwrap_or_default();
    eprintln!(
        "[capture] selected text ({} chars): {:?}",
        selected_text.len(),
        selected_text.chars().take(100).collect::<String>()
    );

    // 5. Store captured context for start_recording to use
    *state.captured_context.lock().unwrap() = Some(CapturedContext {
        selected_text,
        source_app,
    });
}

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
            captured_context: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::settings::store_api_key,
            commands::settings::get_api_key,
            commands::settings::delete_api_key,
            commands::recording::start_recording,
            commands::recording::stop_recording,
            commands::recording::get_silence_duration,
            commands::recording::download_whisper_model,
            commands::prompt::generate_prompt,
            commands::prompt::paste_to_app,
            commands::prompt::copy_to_clipboard,
            commands::prompt::get_prompt_history,
            commands::onboarding::is_first_run,
            commands::onboarding::complete_onboarding,
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
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "start" => {
                        // Capture text BEFORE showing window
                        capture_context(app);
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
                })
                .build(app)?;

            // Register global hotkey from settings
            let app_handle = app.handle().clone();
            let settings_state = app.state::<AppState>();
            let hotkey_str = settings_state
                .settings_manager
                .lock()
                .unwrap()
                .load()
                .unwrap_or_default()
                .global_hotkey;

            use tauri_plugin_global_shortcut::GlobalShortcutExt;
            if let Ok(shortcut) = hotkey_str.parse::<tauri_plugin_global_shortcut::Shortcut>() {
                let _ =
                    app.global_shortcut()
                        .on_shortcut(shortcut, move |_app, _shortcut, event| {
                            if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                                // Capture text BEFORE showing window
                                capture_context(&app_handle);
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
