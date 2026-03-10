use tauri::State;
use crate::commands::settings::AppState;

#[tauri::command]
pub fn is_first_run(state: State<AppState>) -> Result<bool, String> {
    let has_run = state.db.lock().unwrap()
        .get_setting("onboarding_complete")
        .map_err(|e| e.to_string())?;
    Ok(has_run.is_none())
}

#[tauri::command]
pub fn complete_onboarding(state: State<AppState>) -> Result<(), String> {
    state.db.lock().unwrap()
        .set_setting("onboarding_complete", "true")
        .map_err(|e| e.to_string())
}
