use crate::audio::recorder::AudioRecorder;
use crate::clipboard::manager::ClipboardManager;
use crate::keychain::macos::KeychainManager;
use crate::settings::manager::SettingsManager;
use crate::settings::types::AppSettings;
use crate::storage::database::Database;
use std::sync::{Arc, Mutex};
use tauri::State;

pub struct AppState {
    pub settings_manager: Mutex<SettingsManager>,
    pub keychain: KeychainManager,
    pub clipboard: Mutex<ClipboardManager>,
    pub audio_recorder: Arc<AudioRecorder>,
    pub db: Mutex<Database>,
}

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<AppSettings, String> {
    state
        .settings_manager
        .lock()
        .unwrap()
        .load()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_settings(state: State<AppState>, settings: AppSettings) -> Result<(), String> {
    state
        .settings_manager
        .lock()
        .unwrap()
        .save(&settings)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn store_api_key(state: State<AppState>, provider: String, key: String) -> Result<(), String> {
    state
        .keychain
        .store(&provider, &key)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_api_key(state: State<AppState>, provider: String) -> Result<Option<String>, String> {
    state
        .keychain
        .retrieve(&provider)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_api_key(state: State<AppState>, provider: String) -> Result<(), String> {
    state.keychain.delete(&provider).map_err(|e| e.to_string())
}
