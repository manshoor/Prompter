use regex::Regex;
use serde_json::json;

const SYSTEM_PROMPT: &str = r#"You are a prompt engineer. The user has selected content from their coding tool and described what they want to do with it via voice.

Generate a clear, actionable prompt that the user can paste directly into an AI coding assistant (Claude Code, Cursor, etc.).

Rules:
- Be specific and unambiguous
- Include relevant context from the selected text
- Structure the prompt for best AI response
- Match the complexity of the request (simple ask = short prompt)
- Output ONLY the prompt text, no explanations or meta-commentary"#;

/// Redact API keys and secrets from text before sending to AI.
fn redact_secrets(text: &str) -> String {
    let patterns = [
        // Anthropic API keys
        r"sk-ant-[a-zA-Z0-9\-_]{20,}",
        // OpenAI API keys
        r"sk-[a-zA-Z0-9]{20,}",
        // Generic secret/token patterns (key = "value" or key: value)
        r#"(?i)(api[_-]?key|secret[_-]?key|access[_-]?token|auth[_-]?token)\s*[=:]\s*['"]?[a-zA-Z0-9\-_]{16,}['"]?"#,
        // AWS keys
        r"AKIA[0-9A-Z]{16}",
        // GitHub tokens
        r"gh[pousr]_[A-Za-z0-9_]{36,}",
    ];
    let mut result = text.to_string();
    for pattern in &patterns {
        if let Ok(re) = Regex::new(pattern) {
            result = re.replace_all(&result, "[REDACTED]").to_string();
        }
    }
    result
}

pub struct PromptGenerator {
    provider: String,
    api_key: String,
}

impl PromptGenerator {
    pub fn new(provider: &str, api_key: &str) -> Self {
        Self {
            provider: provider.to_string(),
            api_key: api_key.to_string(),
        }
    }

    pub fn build_messages(
        &self,
        selected_text: Option<&str>,
        spoken_intent: Option<&str>,
        source_app: &str,
    ) -> Vec<serde_json::Value> {
        let mut user_content = String::new();
        if let Some(text) = selected_text {
            let safe_text = redact_secrets(text);
            user_content.push_str(&format!("Selected text:\n```\n{}\n```\n\n", safe_text));
        }
        let intent = spoken_intent.unwrap_or("help me with this");
        let safe_intent = redact_secrets(intent);
        user_content.push_str(&format!("User's spoken intent: {}\n\n", safe_intent));
        user_content.push_str(&format!("Source app: {}", source_app));

        vec![
            json!({"role": "system", "content": SYSTEM_PROMPT}),
            json!({"role": "user", "content": user_content}),
        ]
    }

    pub async fn generate(
        &self,
        selected_text: Option<&str>,
        spoken_intent: Option<&str>,
        source_app: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let messages = self.build_messages(selected_text, spoken_intent, source_app);
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        match self.provider.as_str() {
            "claude" => self.call_claude(&client, &messages).await,
            "openai" => self.call_openai(&client, &messages).await,
            _ => Err("Unknown provider".into()),
        }
    }

    async fn call_claude(
        &self,
        client: &reqwest::Client,
        messages: &[serde_json::Value],
    ) -> Result<String, Box<dyn std::error::Error>> {
        let body = json!({
            "model": "claude-sonnet-4-20250514",
            "max_tokens": 2048,
            "system": messages[0]["content"].as_str().unwrap(),
            "messages": [{"role": "user", "content": messages[1]["content"].as_str().unwrap()}],
        });
        let response = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        let json: serde_json::Value = response.json().await?;

        if !status.is_success() {
            let err_msg = json["error"]["message"]
                .as_str()
                .unwrap_or("Unknown API error");
            return Err(format!("Claude API error ({}): {}", status, err_msg).into());
        }

        json["content"][0]["text"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "Unexpected API response format".into())
    }

    async fn call_openai(
        &self,
        client: &reqwest::Client,
        messages: &[serde_json::Value],
    ) -> Result<String, Box<dyn std::error::Error>> {
        let body = json!({
            "model": "gpt-4o",
            "messages": messages,
            "max_tokens": 2048,
        });
        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        let json: serde_json::Value = response.json().await?;

        if !status.is_success() {
            let err_msg = json["error"]["message"]
                .as_str()
                .unwrap_or("Unknown API error");
            return Err(format!("OpenAI API error ({}): {}", status, err_msg).into());
        }

        json["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "Unexpected API response format".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_messages_with_both_inputs() {
        let gen = PromptGenerator::new("claude", "sk-test");
        let messages = gen.build_messages(
            Some("fn add(a: i32, b: i32) -> i32 { a + b }"),
            Some("write tests for this function"),
            "Cursor",
        );
        assert_eq!(messages.len(), 2);
        assert!(messages[1]["content"].as_str().unwrap().contains("fn add"));
        assert!(messages[1]["content"]
            .as_str()
            .unwrap()
            .contains("write tests"));
    }

    #[test]
    fn test_build_messages_no_selection() {
        let gen = PromptGenerator::new("claude", "sk-test");
        let messages = gen.build_messages(None, Some("create a new react component"), "VS Code");
        assert_eq!(messages.len(), 2);
        assert!(!messages[1]["content"]
            .as_str()
            .unwrap()
            .contains("Selected text"));
    }

    #[test]
    fn test_redact_anthropic_key() {
        let text = "my key is sk-ant-api03-abcdefghijklmnopqrstuvwx and more";
        let redacted = redact_secrets(text);
        assert!(!redacted.contains("sk-ant-"));
        assert!(redacted.contains("[REDACTED]"));
        assert!(redacted.contains("and more"));
    }

    #[test]
    fn test_redact_openai_key() {
        let text = "OPENAI_KEY=sk-proj1234567890abcdefghij done";
        let redacted = redact_secrets(text);
        assert!(!redacted.contains("sk-proj"));
        assert!(redacted.contains("[REDACTED]"));
    }

    #[test]
    fn test_redact_generic_api_key() {
        let text = r#"api_key = "abc123def456ghi789jkl012""#;
        let redacted = redact_secrets(text);
        assert!(redacted.contains("[REDACTED]"));
    }

    #[test]
    fn test_no_false_redaction() {
        let text = "fn hello_world() { println!(\"hi\"); }";
        let redacted = redact_secrets(text);
        assert_eq!(text, redacted);
    }

    #[test]
    fn test_build_messages_no_speech() {
        let gen = PromptGenerator::new("claude", "sk-test");
        let messages = gen.build_messages(Some("broken code here"), None, "Terminal");
        assert_eq!(messages.len(), 2);
        assert!(messages[1]["content"]
            .as_str()
            .unwrap()
            .contains("help me with this"));
    }
}
