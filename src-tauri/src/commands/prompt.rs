use crate::commands::settings::AppState;
use crate::prompt::generator::PromptGenerator;
use crate::settings::types::PromptModel;
use tauri::State;

#[tauri::command]
pub async fn generate_prompt(
    state: State<'_, AppState>,
    selected_text: Option<String>,
    spoken_intent: Option<String>,
    source_app: String,
) -> Result<String, String> {
    eprintln!("[generate_prompt] called");
    eprintln!(
        "[generate_prompt] selected_text: {:?} ({} chars)",
        selected_text
            .as_deref()
            .map(|s| s.chars().take(80).collect::<String>()),
        selected_text.as_deref().map_or(0, |s| s.len())
    );
    eprintln!("[generate_prompt] spoken_intent: {:?}", spoken_intent);
    eprintln!("[generate_prompt] source_app: {}", source_app);

    if selected_text.as_deref().map_or(0, |s| s.len()) > 100_000 {
        return Err("Selected text too large (max 100KB)".to_string());
    }

    let settings = state
        .settings_manager
        .lock()
        .map_err(|_| "Internal lock error".to_string())?
        .load()
        .map_err(|e| e.to_string())?;

    let (provider, key_name) = match settings.prompt_model {
        PromptModel::ClaudeSonnet | PromptModel::ClaudeOpus | PromptModel::ClaudeHaiku => {
            ("claude", "claude")
        }
        PromptModel::Gpt4o | PromptModel::Gpt4oMini => ("openai", "openai"),
    };

    eprintln!(
        "[generate_prompt] model: {:?}, provider: {}, key_name: {}",
        settings.prompt_model, provider, key_name
    );

    let db_key = format!("api_key_{}", key_name);
    let db_result = state.db.lock().map_err(|_| "Internal lock error".to_string())?.get_setting(&db_key);
    let api_key = if let Ok(Some(key)) = db_result {
        if key.is_empty() {
            return Err(format!("{} API key not configured", provider));
        }
        key
    } else if let Ok(Some(key)) = state.keychain.retrieve(key_name) {
        // Migrate to database for future reliability
        let _ = state.db.lock().map_err(|_| "Internal lock error".to_string())?.set_setting(&db_key, &key);
        key
    } else {
        eprintln!(
            "[generate_prompt] ERROR: {} API key not configured",
            provider
        );
        return Err(format!("{} API key not configured", provider));
    };

    eprintln!("[generate_prompt] API key configured");

    let generator = PromptGenerator::new(provider, &api_key);
    let prompt = generator
        .generate(
            selected_text.as_deref(),
            spoken_intent.as_deref(),
            &source_app,
        )
        .await
        .map_err(|e| {
            eprintln!("[generate_prompt] ERROR from API: {}", e);
            e.to_string()
        })?;

    eprintln!("[generate_prompt] success ({} chars)", prompt.len());

    // Save to history
    state
        .db
        .lock()
        .map_err(|_| "Internal lock error".to_string())?
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
        .map_err(|_| "Internal lock error".to_string())?
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
        .map_err(|_| "Internal lock error".to_string())?
        .restore()
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn copy_to_clipboard(state: State<AppState>, text: String) -> Result<(), String> {
    state
        .clipboard
        .lock()
        .map_err(|_| "Internal lock error".to_string())?
        .write(&text)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_prompt(state: State<AppState>, id: i64) -> Result<(), String> {
    state
        .db
        .lock()
        .map_err(|_| "Internal lock error".to_string())?
        .delete_prompt(id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn search_prompts(state: State<AppState>, query: String, limit: i64) -> Result<String, String> {
    let history = state
        .db
        .lock()
        .map_err(|_| "Internal lock error".to_string())?
        .search_prompts(&query, limit)
        .map_err(|e| e.to_string())?;
    serde_json::to_string(&history).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_prompt_history(state: State<AppState>, limit: i64) -> Result<String, String> {
    let history = state
        .db
        .lock()
        .map_err(|_| "Internal lock error".to_string())?
        .list_prompts(limit, 0)
        .map_err(|e| e.to_string())?;
    serde_json::to_string(&history).map_err(|e| e.to_string())
}
