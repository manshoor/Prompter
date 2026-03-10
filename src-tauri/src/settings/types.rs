use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WhisperMode {
    Local,
    Cloud,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PasteMode {
    Auto,
    Preview,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RecordingMode {
    PushToTalk,
    Toggle,
    AutoDetect,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PromptModel {
    ClaudeSonnet,
    ClaudeOpus,
    ClaudeHaiku,
    Gpt4o,
    Gpt4oMini,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
    System,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppSettings {
    pub global_hotkey: String,
    pub whisper_mode: WhisperMode,
    pub local_whisper_model: String,
    pub prompt_model: PromptModel,
    pub paste_mode: PasteMode,
    pub recording_mode: RecordingMode,
    pub silence_threshold_secs: f64,
    pub theme: Theme,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            global_hotkey: "CmdOrCtrl+Shift+P".to_string(),
            whisper_mode: WhisperMode::Cloud,
            local_whisper_model: "medium".to_string(),
            prompt_model: PromptModel::ClaudeSonnet,
            paste_mode: PasteMode::Preview,
            recording_mode: RecordingMode::Toggle,
            silence_threshold_secs: 4.0,
            theme: Theme::System,
        }
    }
}
