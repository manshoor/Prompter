use prompter_lib::settings::manager::SettingsManager;
use prompter_lib::settings::types::*;
use prompter_lib::storage::database::Database;
use std::sync::{Arc, Mutex};

#[test]
fn test_settings_full_round_trip() {
    let db = Arc::new(Mutex::new(Database::new_in_memory().unwrap()));
    let manager = SettingsManager::new(db);

    let settings = AppSettings {
        global_hotkey: "Alt+Shift+R".to_string(),
        whisper_mode: WhisperMode::Local,
        local_whisper_model: "large".to_string(),
        prompt_model: PromptModel::Gpt4o,
        paste_mode: PasteMode::Auto,
        recording_mode: RecordingMode::AutoDetect,
        silence_threshold_secs: 6.0,
        theme: Theme::Dark,
    };

    manager.save(&settings).unwrap();
    let loaded = manager.load().unwrap();

    assert_eq!(loaded.global_hotkey, "Alt+Shift+R");
    assert_eq!(loaded.whisper_mode, WhisperMode::Local);
    assert_eq!(loaded.local_whisper_model, "large");
    assert_eq!(loaded.prompt_model, PromptModel::Gpt4o);
    assert_eq!(loaded.paste_mode, PasteMode::Auto);
    assert_eq!(loaded.recording_mode, RecordingMode::AutoDetect);
    assert_eq!(loaded.silence_threshold_secs, 6.0);
    assert_eq!(loaded.theme, Theme::Dark);
}

#[test]
fn test_onboarding_flow() {
    let db = Database::new_in_memory().unwrap();

    // First run: no onboarding_complete flag
    assert_eq!(db.get_setting("onboarding_complete").unwrap(), None);

    // Complete onboarding
    db.set_setting("onboarding_complete", "true").unwrap();

    // Second run: flag is set
    assert_eq!(
        db.get_setting("onboarding_complete").unwrap(),
        Some("true".to_string())
    );
}

#[test]
fn test_api_key_storage_and_deletion() {
    let db = Database::new_in_memory().unwrap();

    // Store a key
    db.set_setting("api_key_openai", "sk-test-key-12345")
        .unwrap();
    assert_eq!(
        db.get_setting("api_key_openai").unwrap(),
        Some("sk-test-key-12345".to_string())
    );

    // Delete the key using delete_setting
    db.delete_setting("api_key_openai").unwrap();

    // Key should be gone (None, not empty string)
    assert_eq!(db.get_setting("api_key_openai").unwrap(), None);
}

#[test]
fn test_prompt_history_pagination() {
    let db = Database::new_in_memory().unwrap();

    db.save_prompt("code1", "intent1", "prompt1", "vscode")
        .unwrap();
    db.save_prompt("code2", "intent2", "prompt2", "cursor")
        .unwrap();
    db.save_prompt("code3", "intent3", "prompt3", "terminal")
        .unwrap();

    // Limit to 2 — should get the 2 newest (ordered DESC)
    let page1 = db.list_prompts(2, 0).unwrap();
    assert_eq!(page1.len(), 2);
    assert_eq!(page1[0].source_app, "terminal");
    assert_eq!(page1[1].source_app, "cursor");

    // Offset 2 — should get the oldest one
    let page2 = db.list_prompts(2, 2).unwrap();
    assert_eq!(page2.len(), 1);
    assert_eq!(page2[0].source_app, "vscode");
}

#[test]
fn test_prompt_message_redaction_in_selection() {
    use prompter_lib::prompt::generator::PromptGenerator;

    let gen = PromptGenerator::new("claude", "sk-test");
    let code_with_key =
        "let api_key = \"sk-ant-api03-abcdefghijklmnopqrstuvwxyz1234567890\";";
    let messages = gen.build_messages(Some(code_with_key), Some("review this code"), "VS Code");

    let content = messages[1]["content"].as_str().unwrap();

    // The API key should be redacted
    assert!(content.contains("[REDACTED]"));
    assert!(!content.contains("sk-ant-api03"));

    // The spoken intent should still be there
    assert!(content.contains("review this code"));
}
