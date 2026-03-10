use tauri::State;
use std::sync::Mutex;
use crate::settings::manager::SettingsManager;
use crate::settings::types::AppSettings;
use crate::keychain::macos::KeychainManager;

pub struct AppState {
    pub settings_manager: Mutex<SettingsManager>,
    pub keychain: KeychainManager,
}

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<AppSettings, String> {
    state.settings_manager.lock().unwrap().load().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_settings(state: State<AppState>, settings: AppSettings) -> Result<(), String> {
    state.settings_manager.lock().unwrap().save(&settings).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn store_api_key(state: State<AppState>, provider: String, key: String) -> Result<(), String> {
    state.keychain.store(&provider, &key).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_api_key(state: State<AppState>, provider: String) -> Result<Option<String>, String> {
    state.keychain.retrieve(&provider).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_api_key(state: State<AppState>, provider: String) -> Result<(), String> {
    state.keychain.delete(&provider).map_err(|e| e.to_string())
}
