use crate::audio::recorder::AudioRecorder;
use crate::clipboard::manager::ClipboardManager;
use crate::focus::tracker::AppInfo;
use crate::keychain::macos::KeychainManager;
use crate::settings::manager::SettingsManager;
use crate::settings::types::AppSettings;
use crate::storage::database::Database;
use std::sync::{Arc, Mutex};
use tauri::{Manager, State};

pub struct CapturedContext {
    pub selected_text: String,
    pub source_app: Option<AppInfo>,
}

pub struct AppState {
    pub settings_manager: Mutex<SettingsManager>,
    pub keychain: KeychainManager,
    pub clipboard: Mutex<ClipboardManager>,
    pub audio_recorder: Arc<AudioRecorder>,
    pub db: Arc<Mutex<Database>>,
    pub captured_context: Mutex<Option<CapturedContext>>,
}

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<AppSettings, String> {
    state
        .settings_manager
        .lock()
        .map_err(|_| "Internal lock error".to_string())?
        .load()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_settings(
    state: State<AppState>,
    app: tauri::AppHandle,
    settings: AppSettings,
) -> Result<(), String> {
    // Load old settings to check if hotkey changed
    let old_settings = state
        .settings_manager
        .lock()
        .map_err(|_| "Internal lock error".to_string())?
        .load()
        .map_err(|e| e.to_string())?;

    // Save new settings
    state
        .settings_manager
        .lock()
        .map_err(|_| "Internal lock error".to_string())?
        .save(&settings)
        .map_err(|e| e.to_string())?;

    // Re-register hotkey if changed
    if old_settings.global_hotkey != settings.global_hotkey {
        re_register_hotkey(&app, &old_settings.global_hotkey, &settings.global_hotkey)?;
    }

    Ok(())
}

fn re_register_hotkey(
    app: &tauri::AppHandle,
    old_hotkey: &str,
    new_hotkey: &str,
) -> Result<(), String> {
    use tauri::Emitter;
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    // Validate new hotkey first
    let new_shortcut = new_hotkey
        .parse::<tauri_plugin_global_shortcut::Shortcut>()
        .map_err(|e| format!("Invalid hotkey '{}': {}", new_hotkey, e))?;

    // Unregister old hotkey
    if let Ok(old_shortcut) = old_hotkey.parse::<tauri_plugin_global_shortcut::Shortcut>() {
        let _ = app.global_shortcut().unregister(old_shortcut);
    }

    // Register new hotkey
    let app_handle = app.clone();
    app.global_shortcut()
        .on_shortcut(new_shortcut, move |_app, _shortcut, event| {
            if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                crate::capture_context(&app_handle);
                let _ = app_handle.emit("hotkey-triggered", ());
                if let Some(window) = app_handle.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .map_err(|e| format!("Failed to register hotkey: {}", e))?;

    eprintln!("[settings] hotkey re-registered: {} -> {}", old_hotkey, new_hotkey);
    Ok(())
}

#[tauri::command]
pub fn store_api_key(state: State<AppState>, provider: String, key: String) -> Result<(), String> {
    let db_key = format!("api_key_{}", provider);
    // Store in database (reliable across rebuilds)
    state
        .db
        .lock()
        .map_err(|_| "Internal lock error".to_string())?
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
    let db_result = state.db.lock().map_err(|_| "Internal lock error".to_string())?.get_setting(&db_key);
    if let Ok(Some(key)) = db_result {
        if !key.is_empty() {
            return Ok(Some(key));
        }
    }
    // Fall back to keychain (may fail after rebuild)
    match state.keychain.retrieve(&provider) {
        Ok(Some(key)) => {
            // Migrate to database for future reliability
            let _ = state.db.lock().map_err(|_| "Internal lock error".to_string())?.set_setting(&db_key, &key);
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
        .map_err(|_| "Internal lock error".to_string())?
        .delete_setting(&db_key)
        .map_err(|e| e.to_string())?;
    // Also delete from keychain (best-effort)
    let _ = state.keychain.delete(&provider);
    Ok(())
}
