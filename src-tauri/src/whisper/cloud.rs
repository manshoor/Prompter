// src-tauri/src/whisper/cloud.rs
use reqwest::multipart;

pub struct CloudWhisper {
    pub api_key: String,
}

impl CloudWhisper {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }

    pub async fn transcribe(&self, wav_bytes: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
        if wav_bytes.is_empty() {
            return Err("Empty audio data".into());
        }

        let file_part = multipart::Part::bytes(wav_bytes.to_vec())
            .file_name("audio.wav")
            .mime_str("audio/wav")?;

        let form = multipart::Form::new()
            .text("model", "whisper-1")
            .part("file", file_part);

        let client = reqwest::Client::new();
        let response = client
            .post("https://api.openai.com/v1/audio/transcriptions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Whisper API error: {}", error_text).into());
        }

        let json: serde_json::Value = response.json().await?;
        let text = json["text"].as_str().unwrap_or("").to_string();
        Ok(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cloud_whisper_builds_request() {
        let whisper = CloudWhisper::new("sk-test-key".to_string());
        assert_eq!(whisper.api_key, "sk-test-key");
    }

    #[tokio::test]
    async fn test_cloud_whisper_rejects_empty_audio() {
        let whisper = CloudWhisper::new("sk-test-key".to_string());
        let result = whisper.transcribe(&[]).await;
        assert!(result.is_err());
    }
}
