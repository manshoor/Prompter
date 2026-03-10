// src-tauri/src/whisper/local.rs
use std::path::PathBuf;

pub struct LocalWhisper {
    model_name: String,
}

impl LocalWhisper {
    pub fn new(model_name: &str) -> Self {
        Self {
            model_name: model_name.to_string(),
        }
    }

    pub fn model_path(model_name: &str) -> String {
        let data_dir = dirs::data_dir()
            .unwrap()
            .join("com.prompter.app")
            .join("models");
        data_dir
            .join(format!("ggml-{}.bin", model_name))
            .to_string_lossy()
            .to_string()
    }

    pub fn model_url(model_name: &str) -> String {
        format!(
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-{}.bin",
            model_name
        )
    }

    pub fn is_model_downloaded(&self) -> bool {
        PathBuf::from(Self::model_path(&self.model_name)).exists()
    }

    pub async fn download_model(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = PathBuf::from(Self::model_path(&self.model_name));
        if path.exists() {
            return Ok(());
        }
        std::fs::create_dir_all(path.parent().unwrap())?;
        let url = Self::model_url(&self.model_name);
        let client = reqwest::Client::new();
        let response = client.get(&url).send().await?;
        let bytes = response.bytes().await?;
        std::fs::write(&path, &bytes)?;
        Ok(())
    }

    pub fn transcribe(&self, wav_bytes: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
        if wav_bytes.is_empty() {
            return Err("Empty audio data".into());
        }
        let model_path = Self::model_path(&self.model_name);
        if !PathBuf::from(&model_path).exists() {
            return Err("Model not downloaded. Please download the model first.".into());
        }

        let ctx = whisper_rs::WhisperContext::new_with_params(
            &model_path,
            whisper_rs::WhisperContextParameters::default(),
        )?;
        let mut state = ctx.create_state()?;

        let mut params =
            whisper_rs::FullParams::new(whisper_rs::SamplingStrategy::Greedy { best_of: 1 });
        params.set_language(Some("en"));
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        let reader = hound::WavReader::new(std::io::Cursor::new(wav_bytes))?;
        let samples: Vec<f32> = reader
            .into_samples::<i16>()
            .filter_map(|s| s.ok())
            .map(|s| s as f32 / 32768.0)
            .collect();

        state.full(params, &samples)?;

        let num_segments = state.full_n_segments()?;
        let mut text = String::new();
        for i in 0..num_segments {
            if let Ok(segment) = state.full_get_segment_text(i) {
                text.push_str(&segment);
            }
        }
        Ok(text.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_path_construction() {
        let path = LocalWhisper::model_path("medium");
        assert!(path.ends_with("ggml-medium.bin"));
        assert!(path.contains("prompter"));
    }

    #[test]
    fn test_model_url_construction() {
        let url = LocalWhisper::model_url("medium");
        assert!(url.contains("ggml-medium.bin"));
        assert!(url.starts_with("https://"));
    }
}
