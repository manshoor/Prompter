use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptRecord {
    pub id: i64,
    pub selected_text: String,
    pub spoken_intent: String,
    pub generated_prompt: String,
    pub source_app: String,
    pub created_at: String,
}
