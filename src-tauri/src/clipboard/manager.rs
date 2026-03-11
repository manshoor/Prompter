use arboard::Clipboard;

pub struct ClipboardManager {
    pub clipboard: Clipboard,
    saved_content: Option<String>,
}

impl ClipboardManager {
    pub fn new() -> Self {
        Self {
            clipboard: Clipboard::new().unwrap(),
            saved_content: None,
        }
    }

    pub fn save_current(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.saved_content = self.clipboard.get_text().ok();
        Ok(())
    }

    pub fn restore(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref content) = self.saved_content {
            self.clipboard.set_text(content)?;
        }
        self.saved_content = None;
        Ok(())
    }

    pub fn read(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(self.clipboard.get_text()?)
    }

    pub fn write(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.clipboard.set_text(text)?;
        Ok(())
    }
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;

    #[test]
    fn test_save_and_restore_clipboard() {
        let mut manager = ClipboardManager::new();
        manager.clipboard.set_text("original content").unwrap();
        manager.save_current().unwrap();
        manager.clipboard.set_text("new content").unwrap();
        manager.restore().unwrap();
        let restored = manager.clipboard.get_text().unwrap();
        assert_eq!(restored, "original content");
    }

    #[test]
    fn test_read_clipboard() {
        let mut manager = ClipboardManager::new();
        manager.clipboard.set_text("test content").unwrap();
        let content = manager.read().unwrap();
        assert_eq!(content, "test content");
    }

    #[test]
    fn test_write_clipboard() {
        let mut manager = ClipboardManager::new();
        manager.write("written by prompter").unwrap();
        let content = manager.clipboard.get_text().unwrap();
        assert_eq!(content, "written by prompter");
    }
}
