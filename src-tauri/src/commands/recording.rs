use crate::commands::settings::AppState;
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

/// Returns pre-captured context (selected text + source app) without starting audio.
#[tauri::command]
pub async fn get_captured_context(state: State<'_, AppState>) -> Result<String, String> {
    eprintln!("[get_captured_context] called");

    let context = state
        .captured_context
        .lock()
        .map_err(|_| "Internal lock error".to_string())?
        .take();

    let (selected_text, source_app) = match context {
        Some(ctx) => {
            eprintln!(
                "[get_captured_context] app={:?}, text={} chars",
                ctx.source_app.as_ref().map(|a| &a.name),
                ctx.selected_text.len()
            );
            (ctx.selected_text, ctx.source_app)
        }
        None => {
            eprintln!("[get_captured_context] WARNING: no pre-captured context");
            (String::new(), None)
        }
    };

    Ok(serde_json::json!({
        "source_app": source_app,
        "selected_text": selected_text,
    })
    .to_string())
}

/// Starts audio recording only (context should be fetched separately via get_captured_context).
#[tauri::command]
pub async fn start_recording(state: State<'_, AppState>) -> Result<(), String> {
    eprintln!("[start_recording] called");

    state.audio_recorder.start().map_err(|e| {
        eprintln!("[start_recording] ERROR starting audio: {}", e);
        e.to_string()
    })?;
    eprintln!("[start_recording] audio recording started");

    Ok(())
}

/// Stops any active recording without transcription.
#[tauri::command]
pub async fn skip_recording(state: State<'_, AppState>) -> Result<(), String> {
    eprintln!("[skip_recording] called");
    let _ = state.audio_recorder.stop();
    Ok(())
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

    // Check for empty/trivial audio — skip transcription if nothing meaningful was recorded
    let sample_rate = audio_data.sample_rate.max(1) as f32;
    let duration_secs = audio_data.samples.len() as f32 / sample_rate;
    if audio_data.samples.is_empty() || duration_secs < 0.5 {
        eprintln!(
            "[stop_recording] trivial audio ({:.2}s), skipping transcription",
            duration_secs
        );
        return Ok(String::new());
    }

    // Check if the entire recording is silence
    let rms = (audio_data
        .samples
        .iter()
        .map(|s| s * s)
        .sum::<f32>()
        / audio_data.samples.len() as f32)
        .sqrt();
    if rms < 0.01 {
        eprintln!(
            "[stop_recording] all silence (rms={:.4}), skipping transcription",
            rms
        );
        return Ok(String::new());
    }

    let wav_bytes = audio_data.to_wav_bytes().map_err(|e| {
        eprintln!("[stop_recording] ERROR converting to WAV: {}", e);
        e.to_string()
    })?;
    eprintln!(
        "[stop_recording] audio captured: {} bytes WAV, {:.1}s",
        wav_bytes.len(),
        duration_secs
    );

    let settings = state
        .settings_manager
        .lock()
        .map_err(|_| "Internal lock error".to_string())?
        .load()
        .map_err(|e| e.to_string())?;

    eprintln!("[stop_recording] whisper mode: {:?}", settings.whisper_mode);

    let transcription = match settings.whisper_mode {
        crate::settings::types::WhisperMode::Cloud => {
            let db_key = "api_key_openai";
            let db_result = state.db.lock().map_err(|_| "Internal lock error".to_string())?.get_setting(db_key);
            let api_key = if let Ok(Some(key)) = db_result {
                if key.is_empty() {
                    return Err("OpenAI API key not configured".to_string());
                }
                key
            } else if let Ok(Some(key)) = state.keychain.retrieve("openai") {
                // Migrate to database for future reliability
                let _ = state.db.lock().map_err(|_| "Internal lock error".to_string())?.set_setting(db_key, &key);
                key
            } else {
                return Err("OpenAI API key not configured".to_string());
            };
            eprintln!("[stop_recording] API key configured");
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

/// Returns the current audio RMS level (0.0–1.0) for the visualizer.
#[tauri::command]
pub fn get_audio_level(state: State<'_, AppState>) -> Result<f32, String> {
    Ok(state.audio_recorder.current_rms())
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
