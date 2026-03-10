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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
