pub struct HotkeyManager;

impl HotkeyManager {
    /// Parses and validates a hotkey string (e.g. "CmdOrCtrl+Shift+P").
    ///
    /// The string must be a `+`-separated sequence where every token except the
    /// last is a recognised modifier and the last token is a non-empty key name.
    pub fn parse_hotkey(shortcut: &str) -> Result<String, Box<dyn std::error::Error>> {
        let parts: Vec<&str> = shortcut.split('+').collect();
        if parts.is_empty() {
            return Err("Empty hotkey string".into());
        }

        let valid_modifiers = ["CmdOrCtrl", "Cmd", "Ctrl", "Alt", "Shift", "Super"];
        let key = parts.last().ok_or("No key specified")?;

        // Every token before the final key must be a valid modifier.
        for part in &parts[..parts.len() - 1] {
            if !valid_modifiers.contains(part) {
                return Err(format!("Invalid modifier: {}", part).into());
            }
        }

        if key.is_empty() {
            return Err("Empty key".into());
        }

        Ok(shortcut.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hotkey_string() {
        let parsed = HotkeyManager::parse_hotkey("CmdOrCtrl+Shift+P");
        assert!(parsed.is_ok());
    }

    #[test]
    fn test_parse_invalid_hotkey() {
        let parsed = HotkeyManager::parse_hotkey("InvalidKey+++");
        assert!(parsed.is_err());
    }

    #[test]
    fn test_parse_single_key() {
        let parsed = HotkeyManager::parse_hotkey("F12");
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap(), "F12");
    }

    #[test]
    fn test_parse_all_modifiers() {
        let parsed = HotkeyManager::parse_hotkey("Cmd+Ctrl+Alt+Shift+Super+A");
        assert!(parsed.is_ok());
    }

    #[test]
    fn test_parse_empty_trailing_key() {
        // Trailing '+' produces an empty key
        let parsed = HotkeyManager::parse_hotkey("Ctrl+");
        assert!(parsed.is_err());
    }
}
