use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    pub bundle_id: String,
    pub name: String,
    pub pid: i32,
}

pub struct FocusTracker;

impl FocusTracker {
    pub fn get_frontmost_app() -> Result<AppInfo, Box<dyn std::error::Error>> {
        let output = Command::new("osascript")
            .arg("-e")
            .arg(
                r#"
                tell application "System Events"
                    set frontApp to first application process whose frontmost is true
                    set appName to name of frontApp
                    set appID to bundle identifier of frontApp
                    set appPID to unix id of frontApp
                    return appName & "|" & appID & "|" & (appPID as text)
                end tell
            "#,
            )
            .output()?;

        let result = String::from_utf8(output.stdout)?.trim().to_string();
        let parts: Vec<&str> = result.split('|').collect();

        if parts.len() >= 3 {
            Ok(AppInfo {
                name: parts[0].to_string(),
                bundle_id: parts[1].to_string(),
                pid: parts[2].parse().unwrap_or(0),
            })
        } else {
            Err("Failed to parse frontmost app info".into())
        }
    }

    pub fn activate_app(bundle_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !bundle_id.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_') {
            return Err("Invalid bundle ID".into());
        }
        Command::new("osascript")
            .arg("-e")
            .arg(format!(
                r#"tell application id "{}" to activate"#,
                bundle_id
            ))
            .output()?;
        Ok(())
    }
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;

    #[test]
    fn test_get_frontmost_app_returns_something() {
        let app = FocusTracker::get_frontmost_app();
        assert!(app.is_ok());
        let info = app.unwrap();
        assert!(!info.bundle_id.is_empty() || !info.name.is_empty());
    }
}
