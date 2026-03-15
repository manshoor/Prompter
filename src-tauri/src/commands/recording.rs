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
    eprintln!("[start_recording] called");

    // Read pre-captured context (captured by hotkey handler before window was shown)
    let context = state.captured_context.lock().unwrap().take();

    let (selected_text, source_app) = match context {
        Some(ctx) => {
            eprintln!(
                "[start_recording] using pre-captured context: app={:?}, text={} chars",
                ctx.source_app.as_ref().map(|a| &a.name),
                ctx.selected_text.len()
            );
            (ctx.selected_text, ctx.source_app)
        }
        None => {
            eprintln!("[start_recording] WARNING: no pre-captured context, using fallback");
            let app_info = FocusTracker::get_frontmost_app().ok();
            (String::new(), app_info)
        }
    };

    // Start audio recording
    state.audio_recorder.start().map_err(|e| {
        eprintln!("[start_recording] ERROR starting audio: {}", e);
        e.to_string()
    })?;
    eprintln!("[start_recording] audio recording started");

    // Return source app info + selected text
    Ok(serde_json::json!({
        "source_app": source_app,
        "selected_text": selected_text,
    })
    .to_string())
}

#[tauri::command]
pub async fn stop_recording(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    eprintln!("[stop_recording] called");

    let audio_data = state.audio_recorder.stop().map_err(|e| {
        eprintln!("[stop_recording] ERROR stopping audio: {}", e);
        e.to_string()
    })?;

    let wav_bytes = audio_data.to_wav_bytes().map_err(|e| {
        eprintln!("[stop_recording] ERROR converting to WAV: {}", e);
        e.to_string()
    })?;
    eprintln!(
        "[stop_recording] audio captured: {} bytes WAV",
        wav_bytes.len()
    );

    let settings = state
        .settings_manager
        .lock()
        .unwrap()
        .load()
        .map_err(|e| e.to_string())?;

    eprintln!("[stop_recording] whisper mode: {:?}", settings.whisper_mode);

    let transcription = match settings.whisper_mode {
        crate::settings::types::WhisperMode::Cloud => {
            let db_key = "api_key_openai";
            let api_key = if let Ok(Some(key)) = state.db.lock().unwrap().get_setting(db_key) {
                if key.is_empty() {
                    return Err("OpenAI API key not configured".to_string());
                }
                key
            } else if let Ok(Some(key)) = state.keychain.retrieve("openai") {
                // Migrate to database for future reliability
                let _ = state.db.lock().unwrap().set_setting(db_key, &key);
                key
            } else {
                return Err("OpenAI API key not configured".to_string());
            };
            eprintln!(
                "[stop_recording] using Cloud whisper (key: {}...)",
                &api_key[..8.min(api_key.len())]
            );
            let whisper = crate::whisper::cloud::CloudWhisper::new(api_key);
            let result = whisper.transcribe(&wav_bytes).await.map_err(|e| {
                eprintln!("[stop_recording] ERROR cloud transcription: {}", e);
                e.to_string()
            })?;
            eprintln!("[stop_recording] transcription: {:?}", result);
            result
        }
        crate::settings::types::WhisperMode::Local => {
            eprintln!(
                "[stop_recording] using Local whisper (model: {})",
                settings.local_whisper_model
            );
            let whisper = crate::whisper::local::LocalWhisper::new(&settings.local_whisper_model);
            if !whisper.is_model_downloaded() {
                eprintln!("[stop_recording] model not found, downloading...");
                let _ = app.emit("model-download-started", ());
                whisper
                    .download_model_with_progress(|downloaded, total| {
                        emit_download_progress(&app, downloaded, total);
                    })
                    .await
                    .map_err(|e| {
                        eprintln!("[stop_recording] ERROR downloading model: {}", e);
                        e.to_string()
                    })?;
                eprintln!("[stop_recording] model download complete");
            }
            let result = whisper.transcribe(&wav_bytes).map_err(|e| {
                eprintln!("[stop_recording] ERROR local transcription: {}", e);
                e.to_string()
            })?;
            eprintln!("[stop_recording] transcription: {:?}", result);
            result
        }
    };

    Ok(transcription)
}

#[tauri::command]
pub fn get_silence_duration(state: State<'_, AppState>) -> Result<f32, String> {
    Ok(state.audio_recorder.silence_duration_secs())
}

#[tauri::command]
pub async fn download_whisper_model(model: String, app: tauri::AppHandle) -> Result<(), String> {
    eprintln!("[download_model] downloading model: {}", model);
    let whisper = crate::whisper::local::LocalWhisper::new(&model);
    let _ = app.emit("model-download-started", ());
    whisper
        .download_model_with_progress(|downloaded, total| {
            emit_download_progress(&app, downloaded, total);
        })
        .await
        .map_err(|e| {
            eprintln!("[download_model] ERROR: {}", e);
            e.to_string()
        })?;
    eprintln!("[download_model] complete");
    Ok(())
}
