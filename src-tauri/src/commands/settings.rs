use crate::audio::recorder::AudioRecorder;
use crate::clipboard::manager::ClipboardManager;
use crate::focus::tracker::AppInfo;
use crate::keychain::macos::KeychainManager;
use crate::settings::manager::SettingsManager;
use crate::settings::types::AppSettings;
use crate::storage::database::Database;
use std::sync::{Arc, Mutex};
use tauri::State;

pub struct CapturedContext {
    pub selected_text: String,
    pub source_app: Option<AppInfo>,
}

pub struct AppState {
    pub settings_manager: Mutex<SettingsManager>,
    pub keychain: KeychainManager,
    pub clipboard: Mutex<ClipboardManager>,
    pub audio_recorder: Arc<AudioRecorder>,
    pub db: Mutex<Database>,
    pub captured_context: Mutex<Option<CapturedContext>>,
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
    let db_key = format!("api_key_{}", provider);
    // Store in database (reliable across rebuilds)
    state
        .db
        .lock()
        .unwrap()
        .set_setting(&db_key, &key)
        .map_err(|e| e.to_string())?;
    // Also store in keychain (best-effort, may fail on dev rebuilds)
    let _ = state.keychain.store(&provider, &key);
    Ok(())
}

#[tauri::command]
pub fn get_api_key(state: State<AppState>, provider: String) -> Result<Option<String>, String> {
    let db_key = format!("api_key_{}", provider);
    // Try database first (reliable)
    if let Ok(Some(key)) = state.db.lock().unwrap().get_setting(&db_key) {
        return Ok(Some(key));
    }
    // Fall back to keychain (may fail after rebuild)
    match state.keychain.retrieve(&provider) {
        Ok(Some(key)) => {
            // Migrate to database for future reliability
            let _ = state.db.lock().unwrap().set_setting(&db_key, &key);
            Ok(Some(key))
        }
        Ok(None) => Ok(None),
        Err(_) => Ok(None),
    }
}

#[tauri::command]
pub fn delete_api_key(state: State<AppState>, provider: String) -> Result<(), String> {
    let db_key = format!("api_key_{}", provider);
    // Delete from database
    state
        .db
        .lock()
        .unwrap()
        .set_setting(&db_key, "")
        .map_err(|e| e.to_string())?;
    // Also delete from keychain (best-effort)
    let _ = state.keychain.delete(&provider);
    Ok(())
}
