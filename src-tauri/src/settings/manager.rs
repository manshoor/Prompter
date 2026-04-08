use super::types::AppSettings;
use crate::storage::database::Database;
use std::sync::{Arc, Mutex};

pub struct SettingsManager {
    db: Arc<Mutex<Database>>,
}

impl SettingsManager {
    pub fn new(db: Arc<Mutex<Database>>) -> Self {
        Self { db }
    }

    pub fn save(&self, settings: &AppSettings) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string(settings)?;
        self.db
            .lock()
            .map_err(|_| "Lock error")?
            .set_setting("app_settings", &json)?;
        Ok(())
    }

    pub fn load(&self) -> Result<AppSettings, Box<dyn std::error::Error>> {
        match self
            .db
            .lock()
            .map_err(|_| "Lock error")?
            .get_setting("app_settings")?
        {
            Some(json) => Ok(serde_json::from_str(&json)?),
            None => Ok(AppSettings::default()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::types::*;
    use crate::storage::database::Database;

    #[test]
    fn test_default_settings() {
        let settings = AppSettings::default();
        assert_eq!(settings.whisper_mode, WhisperMode::Cloud);
        assert_eq!(settings.local_whisper_model, "medium");
        assert_eq!(settings.paste_mode, PasteMode::Preview);
        assert_eq!(settings.recording_mode, RecordingMode::Toggle);
        assert_eq!(settings.silence_threshold_secs, 4.0);
        assert_eq!(settings.global_hotkey, "CmdOrCtrl+Shift+P");
        assert_eq!(settings.theme, Theme::System);
    }

    #[test]
    fn test_save_and_load_settings() {
        let db = Arc::new(Mutex::new(Database::new_in_memory().unwrap()));
        let manager = SettingsManager::new(db);

        let mut settings = AppSettings::default();
        settings.whisper_mode = WhisperMode::Local;
        settings.silence_threshold_secs = 3.0;

        manager.save(&settings).unwrap();
        let loaded = manager.load().unwrap();

        assert_eq!(loaded.whisper_mode, WhisperMode::Local);
        assert_eq!(loaded.silence_threshold_secs, 3.0);
        assert_eq!(loaded.paste_mode, PasteMode::Preview);
    }

    #[test]
    fn test_load_returns_defaults_when_empty() {
        let db = Arc::new(Mutex::new(Database::new_in_memory().unwrap()));
        let manager = SettingsManager::new(db);
        let loaded = manager.load().unwrap();
        assert_eq!(loaded, AppSettings::default());
    }
}
