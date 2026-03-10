use rusqlite::{Connection, Result, params};
use super::models::PromptRecord;

pub struct Database {
    pub conn: Connection,
}

impl Database {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.init_tables()?;
        Ok(db)
    }

    pub fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.init_tables()?;
        Ok(db)
    }

    fn init_tables(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS prompt_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                selected_text TEXT NOT NULL,
                spoken_intent TEXT NOT NULL,
                generated_prompt TEXT NOT NULL,
                source_app TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );"
        )?;
        Ok(())
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;
        match rows.next()? {
            Some(row) => Ok(Some(row.get(0)?)),
            None => Ok(None),
        }
    }

    pub fn save_prompt(
        &self,
        selected_text: &str,
        spoken_intent: &str,
        generated_prompt: &str,
        source_app: &str,
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO prompt_history (selected_text, spoken_intent, generated_prompt, source_app)
             VALUES (?1, ?2, ?3, ?4)",
            params![selected_text, spoken_intent, generated_prompt, source_app],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn list_prompts(&self, limit: i64, offset: i64) -> Result<Vec<PromptRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, selected_text, spoken_intent, generated_prompt, source_app, created_at
             FROM prompt_history ORDER BY id DESC LIMIT ?1 OFFSET ?2"
        )?;
        let records = stmt.query_map(params![limit, offset], |row| {
            Ok(PromptRecord {
                id: row.get(0)?,
                selected_text: row.get(1)?,
                spoken_intent: row.get(2)?,
                generated_prompt: row.get(3)?,
                source_app: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;
        records.collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_creates_tables() {
        let db = Database::new_in_memory().unwrap();
        let count: i64 = db.conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='settings'",
            [],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_database_creates_prompt_history_table() {
        let db = Database::new_in_memory().unwrap();
        let count: i64 = db.conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='prompt_history'",
            [],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_save_and_load_setting() {
        let db = Database::new_in_memory().unwrap();
        db.set_setting("whisper_mode", "local").unwrap();
        let val = db.get_setting("whisper_mode").unwrap();
        assert_eq!(val, Some("local".to_string()));
    }

    #[test]
    fn test_get_missing_setting_returns_none() {
        let db = Database::new_in_memory().unwrap();
        let val = db.get_setting("nonexistent").unwrap();
        assert_eq!(val, None);
    }

    #[test]
    fn test_save_and_list_prompt_history() {
        let db = Database::new_in_memory().unwrap();
        db.save_prompt("selected code", "fix this bug", "Fix the null pointer...", "cursor").unwrap();
        db.save_prompt("other code", "refactor", "Refactor the function...", "vscode").unwrap();
        let history = db.list_prompts(10, 0).unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].generated_prompt, "Refactor the function...");
    }
}
