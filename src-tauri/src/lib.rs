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

use commands::settings::AppState;
use keychain::macos::KeychainManager;
use settings::manager::SettingsManager;
use storage::database::Database;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db_path = dirs::data_dir()
        .unwrap()
        .join("com.prompter.app")
        .join("prompter.db");
    std::fs::create_dir_all(db_path.parent().unwrap()).unwrap();

    let db = Database::new(db_path.to_str().unwrap()).unwrap();
    let settings_manager = SettingsManager::new(db);
    let keychain = KeychainManager::new("com.prompter.app");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(AppState {
            settings_manager: Mutex::new(settings_manager),
            keychain,
        })
        .invoke_handler(tauri::generate_handler![
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::settings::store_api_key,
            commands::settings::get_api_key,
            commands::settings::delete_api_key,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
