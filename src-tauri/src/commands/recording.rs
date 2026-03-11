use crate::commands::settings::AppState;
use crate::focus::tracker::FocusTracker;
use tauri::{Emitter, State};

fn emit_download_progress(app: &tauri::AppHandle, downloaded: u64, total: u64) {
    let percent = if total > 0 {
        (downloaded as f64 / total as f64 * 100.0) as u32
    } else {
        0
    };
    let _ = app.emit(
        "model-download-progress",
        serde_json::json!({ "downloaded": downloaded, "total": total, "percent": percent }),
    );
}

#[tauri::command]
pub async fn start_recording(state: State<'_, AppState>) -> Result<String, String> {
    // Capture frontmost app before we take focus
    let app_info = FocusTracker::get_frontmost_app().map_err(|e| e.to_string())?;

    // Save current clipboard
    state
        .clipboard
        .lock()
        .unwrap()
        .save_current()
        .map_err(|e| e.to_string())?;

    // Simulate Cmd+C to copy selection
    std::process::Command::new("osascript")
        .arg("-e")
        .arg(r#"tell application "System Events" to keystroke "c" using command down"#)
        .output()
        .map_err(|e| e.to_string())?;

    std::thread::sleep(std::time::Duration::from_millis(100));

    // Read clipboard (selected text)
    let selected_text = state.clipboard.lock().unwrap().read().unwrap_or_default();

    // Start audio recording
    state.audio_recorder.start().map_err(|e| e.to_string())?;

    // Return source app info + selected text
    Ok(serde_json::json!({
        "source_app": app_info,
        "selected_text": selected_text,
    })
    .to_string())
}

#[tauri::command]
pub async fn stop_recording(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let audio_data = state.audio_recorder.stop().map_err(|e| e.to_string())?;
    let wav_bytes = audio_data.to_wav_bytes().map_err(|e| e.to_string())?;

    let settings = state
        .settings_manager
        .lock()
        .unwrap()
        .load()
        .map_err(|e| e.to_string())?;

    let transcription = match settings.whisper_mode {
        crate::settings::types::WhisperMode::Cloud => {
            let api_key = state
                .keychain
                .retrieve("openai")
                .map_err(|e| e.to_string())?
                .ok_or("OpenAI API key not configured")?;
            let whisper = crate::whisper::cloud::CloudWhisper::new(api_key);
            whisper
                .transcribe(&wav_bytes)
                .await
                .map_err(|e| e.to_string())?
        }
        crate::settings::types::WhisperMode::Local => {
            let whisper = crate::whisper::local::LocalWhisper::new(&settings.local_whisper_model);
            if !whisper.is_model_downloaded() {
                let _ = app.emit("model-download-started", ());
                whisper
                    .download_model_with_progress(|downloaded, total| {
                        emit_download_progress(&app, downloaded, total);
                    })
                    .await
                    .map_err(|e| e.to_string())?;
            }
            whisper.transcribe(&wav_bytes).map_err(|e| e.to_string())?
        }
    };

    Ok(transcription)
}

#[tauri::command]
pub async fn download_whisper_model(
    model: String,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let whisper = crate::whisper::local::LocalWhisper::new(&model);
    let _ = app.emit("model-download-started", ());
    whisper
        .download_model_with_progress(|downloaded, total| {
            emit_download_progress(&app, downloaded, total);
        })
        .await
        .map_err(|e| e.to_string())
}
