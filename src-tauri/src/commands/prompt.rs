use tauri::State;
use crate::commands::settings::AppState;
use crate::prompt::generator::PromptGenerator;
use crate::settings::types::PromptModel;

#[tauri::command]
pub async fn generate_prompt(
    state: State<'_, AppState>,
    selected_text: Option<String>,
    spoken_intent: Option<String>,
    source_app: String,
) -> Result<String, String> {
    let settings = state
        .settings_manager
        .lock()
        .unwrap()
        .load()
        .map_err(|e| e.to_string())?;

    let (provider, key_name) = match settings.prompt_model {
        PromptModel::ClaudeSonnet | PromptModel::ClaudeOpus | PromptModel::ClaudeHaiku => {
            ("claude", "claude")
        }
        PromptModel::Gpt4o | PromptModel::Gpt4oMini => ("openai", "openai"),
    };

    let api_key = state
        .keychain
        .retrieve(key_name)
        .map_err(|e| e.to_string())?
        .ok_or(format!("{} API key not configured", provider))?;

    let generator = PromptGenerator::new(provider, &api_key);
    let prompt = generator
        .generate(
            selected_text.as_deref(),
            spoken_intent.as_deref(),
            &source_app,
        )
        .await
        .map_err(|e| e.to_string())?;

    // Save to history
    state
        .db
        .lock()
        .unwrap()
        .save_prompt(
            selected_text.as_deref().unwrap_or(""),
            spoken_intent.as_deref().unwrap_or(""),
            &prompt,
            &source_app,
        )
        .map_err(|e| e.to_string())?;

    Ok(prompt)
}

#[tauri::command]
pub async fn paste_to_app(
    state: State<'_, AppState>,
    text: String,
    bundle_id: String,
) -> Result<(), String> {
    state
        .clipboard
        .lock()
        .unwrap()
        .write(&text)
        .map_err(|e| e.to_string())?;

    crate::focus::tracker::FocusTracker::activate_app(&bundle_id).map_err(|e| e.to_string())?;
    std::thread::sleep(std::time::Duration::from_millis(200));

    std::process::Command::new("osascript")
        .arg("-e")
        .arg(r#"tell application "System Events" to keystroke "v" using command down"#)
        .output()
        .map_err(|e| e.to_string())?;

    std::thread::sleep(std::time::Duration::from_millis(500));
    state
        .clipboard
        .lock()
        .unwrap()
        .restore()
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn get_prompt_history(state: State<AppState>, limit: i64) -> Result<String, String> {
    let history = state
        .db
        .lock()
        .unwrap()
        .list_prompts(limit, 0)
        .map_err(|e| e.to_string())?;
    serde_json::to_string(&history).map_err(|e| e.to_string())
}
