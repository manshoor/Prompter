# Prompter Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a macOS desktop app that captures selected text + voice input, generates polished AI prompts, and pastes them back into the source app.

**Architecture:** Monolithic Tauri 2.x app with Rust backend handling system interactions (hotkeys, clipboard, audio, APIs) and Vite+React+TypeScript frontend for the floating widget, settings panel, and menu bar dropdown. SQLite for local storage, macOS Keychain for secrets.

**Tech Stack:** Tauri 2.x, Rust, Vite, React, TypeScript, SQLite (rusqlite), whisper-rs, cpal, arboard, global-hotkey, security-framework

---

### Task 1: Project Scaffolding

**Files:**
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/src/main.rs`
- Create: `src-tauri/src/lib.rs`
- Create: `package.json`
- Create: `vite.config.ts`
- Create: `tsconfig.json`
- Create: `src/main.tsx`
- Create: `src/App.tsx`

**Step 1: Install Tauri CLI and create project**

```bash
npm create tauri-app@latest . -- --template react-ts --manager npm
```

If the directory is not empty, move the `.claude` and `docs` directories aside, scaffold, then move them back.

**Step 2: Add Rust dependencies to Cargo.toml**

Add to `src-tauri/Cargo.toml` `[dependencies]`:

```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-shell = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.31", features = ["bundled"] }
arboard = "3"
cpal = "0.15"
whisper-rs = "0.12"
reqwest = { version = "0.12", features = ["json"] }
tokio = { version = "1", features = ["full"] }
security-framework = "2"
hound = "3.5"
chrono = "0.4"
dirs = "5"
```

**Step 3: Add frontend dependencies**

```bash
npm install zustand react-router-dom lucide-react
npm install -D tailwindcss @tailwindcss/vite
```

**Step 4: Verify the app builds and runs**

```bash
npm run tauri dev
```

Expected: A blank Tauri window opens.

**Step 5: Set up project directory structure**

Create these directories in `src-tauri/src/`:
```
src-tauri/src/
├── storage/
├── settings/
├── keychain/
├── hotkey/
├── clipboard/
├── focus/
├── audio/
├── whisper/
├── prompt/
└── commands/
```

Create these directories in `src/`:
```
src/
├── components/
│   ├── FloatingWidget/
│   ├── SettingsPanel/
│   └── MenuBar/
├── hooks/
├── stores/
└── styles/
```

Add `mod.rs` files for each Rust module directory. Wire them into `lib.rs`:

```rust
// src-tauri/src/lib.rs
mod storage;
mod settings;
mod keychain;
mod hotkey;
mod clipboard;
mod focus;
mod audio;
mod whisper;
mod prompt;
mod commands;
```

**Step 6: Commit**

```bash
git add -A
git commit -m "chore: scaffold Tauri 2.x project with Rust + React + TypeScript"
```

---

### Task 2: SQLite Storage Layer

**Files:**
- Create: `src-tauri/src/storage/mod.rs`
- Create: `src-tauri/src/storage/database.rs`
- Create: `src-tauri/src/storage/models.rs`

**Step 1: Write failing tests for database initialization and CRUD**

```rust
// src-tauri/src/storage/database.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_creates_tables() {
        let db = Database::new_in_memory().unwrap();
        // Verify settings table exists
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
```

**Step 2: Run tests to verify they fail**

```bash
cd src-tauri && cargo test storage -- --nocapture
```

Expected: FAIL — structs and methods don't exist yet.

**Step 3: Define models**

```rust
// src-tauri/src/storage/models.rs
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
```

**Step 4: Implement Database struct**

```rust
// src-tauri/src/storage/database.rs
use rusqlite::{Connection, Result, params};
use super::models::PromptRecord;

pub struct Database {
    conn: Connection,
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
             FROM prompt_history ORDER BY created_at DESC LIMIT ?1 OFFSET ?2"
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
```

**Step 5: Wire up mod.rs**

```rust
// src-tauri/src/storage/mod.rs
pub mod database;
pub mod models;
```

**Step 6: Run tests to verify they pass**

```bash
cd src-tauri && cargo test storage -- --nocapture
```

Expected: All 5 tests PASS.

**Step 7: Commit**

```bash
git add src-tauri/src/storage/
git commit -m "feat: add SQLite storage layer with settings and prompt history"
```

---

### Task 3: Settings Manager

**Files:**
- Create: `src-tauri/src/settings/mod.rs`
- Create: `src-tauri/src/settings/manager.rs`
- Create: `src-tauri/src/settings/types.rs`

**Step 1: Write failing tests**

```rust
// src-tauri/src/settings/manager.rs
#[cfg(test)]
mod tests {
    use super::*;
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
        let db = Database::new_in_memory().unwrap();
        let manager = SettingsManager::new(db);

        let mut settings = AppSettings::default();
        settings.whisper_mode = WhisperMode::Local;
        settings.silence_threshold_secs = 3.0;

        manager.save(&settings).unwrap();
        let loaded = manager.load().unwrap();

        assert_eq!(loaded.whisper_mode, WhisperMode::Local);
        assert_eq!(loaded.silence_threshold_secs, 3.0);
        // Other fields should be default
        assert_eq!(loaded.paste_mode, PasteMode::Preview);
    }

    #[test]
    fn test_load_returns_defaults_when_empty() {
        let db = Database::new_in_memory().unwrap();
        let manager = SettingsManager::new(db);
        let loaded = manager.load().unwrap();
        assert_eq!(loaded, AppSettings::default());
    }
}
```

**Step 2: Run tests to verify they fail**

```bash
cd src-tauri && cargo test settings -- --nocapture
```

Expected: FAIL — types don't exist yet.

**Step 3: Define settings types**

```rust
// src-tauri/src/settings/types.rs
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
```

**Step 4: Implement SettingsManager**

```rust
// src-tauri/src/settings/manager.rs
use crate::storage::database::Database;
use super::types::AppSettings;

pub struct SettingsManager {
    db: Database,
}

impl SettingsManager {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn save(&self, settings: &AppSettings) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string(settings)?;
        self.db.set_setting("app_settings", &json)?;
        Ok(())
    }

    pub fn load(&self) -> Result<AppSettings, Box<dyn std::error::Error>> {
        match self.db.get_setting("app_settings")? {
            Some(json) => Ok(serde_json::from_str(&json)?),
            None => Ok(AppSettings::default()),
        }
    }
}
```

**Step 5: Wire up mod.rs**

```rust
// src-tauri/src/settings/mod.rs
pub mod manager;
pub mod types;
```

**Step 6: Run tests to verify they pass**

```bash
cd src-tauri && cargo test settings -- --nocapture
```

Expected: All 3 tests PASS.

**Step 7: Commit**

```bash
git add src-tauri/src/settings/
git commit -m "feat: add settings manager with typed config and SQLite persistence"
```

---

### Task 4: macOS Keychain Integration

**Files:**
- Create: `src-tauri/src/keychain/mod.rs`
- Create: `src-tauri/src/keychain/macos.rs`

**Step 1: Write failing tests**

```rust
// src-tauri/src/keychain/macos.rs
#[cfg(test)]
mod tests {
    use super::*;

    // Integration tests — these interact with the real macOS Keychain.
    // Use a unique service name to avoid collisions.
    const TEST_SERVICE: &str = "com.prompter.test";

    #[test]
    fn test_store_and_retrieve_key() {
        let keychain = KeychainManager::new(TEST_SERVICE);
        keychain.store("test_openai_key", "sk-test-12345").unwrap();
        let retrieved = keychain.retrieve("test_openai_key").unwrap();
        assert_eq!(retrieved, Some("sk-test-12345".to_string()));
        // Cleanup
        let _ = keychain.delete("test_openai_key");
    }

    #[test]
    fn test_retrieve_missing_key_returns_none() {
        let keychain = KeychainManager::new(TEST_SERVICE);
        let result = keychain.retrieve("nonexistent_key").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_delete_key() {
        let keychain = KeychainManager::new(TEST_SERVICE);
        keychain.store("test_delete_key", "value").unwrap();
        keychain.delete("test_delete_key").unwrap();
        let result = keychain.retrieve("test_delete_key").unwrap();
        assert_eq!(result, None);
    }
}
```

**Step 2: Run tests to verify they fail**

```bash
cd src-tauri && cargo test keychain -- --nocapture
```

Expected: FAIL — `KeychainManager` doesn't exist.

**Step 3: Implement KeychainManager**

```rust
// src-tauri/src/keychain/macos.rs
use security_framework::passwords::{
    set_generic_password, get_generic_password, delete_generic_password,
};

pub struct KeychainManager {
    service: String,
}

impl KeychainManager {
    pub fn new(service: &str) -> Self {
        Self {
            service: service.to_string(),
        }
    }

    pub fn store(&self, account: &str, password: &str) -> Result<(), Box<dyn std::error::Error>> {
        set_generic_password(&self.service, account, password.as_bytes())?;
        Ok(())
    }

    pub fn retrieve(&self, account: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
        match get_generic_password(&self.service, account) {
            Ok(bytes) => Ok(Some(String::from_utf8(bytes)?)),
            Err(e) if e.code() == -25300 => Ok(None), // errSecItemNotFound
            Err(e) => Err(Box::new(e)),
        }
    }

    pub fn delete(&self, account: &str) -> Result<(), Box<dyn std::error::Error>> {
        match delete_generic_password(&self.service, account) {
            Ok(()) => Ok(()),
            Err(e) if e.code() == -25300 => Ok(()), // Already gone
            Err(e) => Err(Box::new(e)),
        }
    }
}
```

**Step 4: Wire up mod.rs**

```rust
// src-tauri/src/keychain/mod.rs
pub mod macos;
```

**Step 5: Run tests to verify they pass**

```bash
cd src-tauri && cargo test keychain -- --nocapture
```

Expected: All 3 tests PASS. (May prompt for Keychain access on first run.)

**Step 6: Commit**

```bash
git add src-tauri/src/keychain/
git commit -m "feat: add macOS Keychain integration for secure API key storage"
```

---

### Task 5: Tauri IPC Commands — Settings & Keychain

**Files:**
- Create: `src-tauri/src/commands/mod.rs`
- Create: `src-tauri/src/commands/settings.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/main.rs`

**Step 1: Define Tauri commands for settings**

```rust
// src-tauri/src/commands/settings.rs
use tauri::State;
use std::sync::Mutex;
use crate::settings::manager::SettingsManager;
use crate::settings::types::AppSettings;
use crate::keychain::macos::KeychainManager;

pub struct AppState {
    pub settings_manager: Mutex<SettingsManager>,
    pub keychain: KeychainManager,
}

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<AppSettings, String> {
    state.settings_manager.lock().unwrap().load().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_settings(state: State<AppState>, settings: AppSettings) -> Result<(), String> {
    state.settings_manager.lock().unwrap().save(&settings).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn store_api_key(state: State<AppState>, provider: String, key: String) -> Result<(), String> {
    state.keychain.store(&provider, &key).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_api_key(state: State<AppState>, provider: String) -> Result<Option<String>, String> {
    state.keychain.retrieve(&provider).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_api_key(state: State<AppState>, provider: String) -> Result<(), String> {
    state.keychain.delete(&provider).map_err(|e| e.to_string())
}
```

**Step 2: Wire commands into mod.rs**

```rust
// src-tauri/src/commands/mod.rs
pub mod settings;
```

**Step 3: Register commands in lib.rs**

Update `src-tauri/src/lib.rs` to initialize the app state and register commands:

```rust
// src-tauri/src/lib.rs
mod storage;
mod settings;
mod keychain;
mod commands;

use commands::settings::AppState;
use keychain::macos::KeychainManager;
use settings::manager::SettingsManager;
use storage::database::Database;
use std::sync::Mutex;

pub fn run() {
    let db_path = dirs::data_dir()
        .unwrap()
        .join("com.prompter.app")
        .join("prompter.db");

    std::fs::create_dir_all(db_path.parent().unwrap()).unwrap();

    let db = Database::new(db_path.to_str().unwrap()).unwrap();
    let settings_manager = SettingsManager::new(db);
    let keychain = KeychainManager::new("com.prompter.app");

    tauri::Builder::default()
        .manage(AppState {
            settings_manager: Mutex::new(settings_manager),
            keychain,
        })
        .invoke_handler(tauri::generate_handler![
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::settings::store_api_key,
            commands::settings::get_api_key,
            commands::settings::delete_api_key,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Step 4: Verify it compiles**

```bash
cd src-tauri && cargo build
```

Expected: Compiles without errors.

**Step 5: Commit**

```bash
git add src-tauri/src/commands/ src-tauri/src/lib.rs
git commit -m "feat: add Tauri IPC commands for settings and API key management"
```

---

### Task 6: Settings Panel UI

**Files:**
- Create: `src/components/SettingsPanel/SettingsPanel.tsx`
- Create: `src/components/SettingsPanel/ApiKeyInput.tsx`
- Create: `src/components/SettingsPanel/index.ts`
- Create: `src/stores/settingsStore.ts`
- Modify: `src/App.tsx`

**Step 1: Create settings Zustand store**

```typescript
// src/stores/settingsStore.ts
import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

export type WhisperMode = "Local" | "Cloud";
export type PasteMode = "Auto" | "Preview";
export type RecordingMode = "PushToTalk" | "Toggle" | "AutoDetect";
export type PromptModel = "ClaudeSonnet" | "ClaudeOpus" | "ClaudeHaiku" | "Gpt4o" | "Gpt4oMini";
export type Theme = "Light" | "Dark" | "System";

export interface AppSettings {
  global_hotkey: string;
  whisper_mode: WhisperMode;
  local_whisper_model: string;
  prompt_model: PromptModel;
  paste_mode: PasteMode;
  recording_mode: RecordingMode;
  silence_threshold_secs: number;
  theme: Theme;
}

interface SettingsState {
  settings: AppSettings | null;
  loading: boolean;
  load: () => Promise<void>;
  save: (settings: AppSettings) => Promise<void>;
}

export const useSettingsStore = create<SettingsState>((set) => ({
  settings: null,
  loading: true,
  load: async () => {
    const settings = await invoke<AppSettings>("get_settings");
    set({ settings, loading: false });
  },
  save: async (settings: AppSettings) => {
    await invoke("save_settings", { settings });
    set({ settings });
  },
}));
```

**Step 2: Create ApiKeyInput component**

```tsx
// src/components/SettingsPanel/ApiKeyInput.tsx
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Props {
  provider: string;
  label: string;
}

export function ApiKeyInput({ provider, label }: Props) {
  const [key, setKey] = useState("");
  const [saved, setSaved] = useState(false);
  const [hasKey, setHasKey] = useState(false);

  useEffect(() => {
    invoke<string | null>("get_api_key", { provider }).then((existing) => {
      if (existing) {
        setHasKey(true);
        setKey("••••••••••••••••");
      }
    });
  }, [provider]);

  const handleSave = async () => {
    if (key && !key.startsWith("••")) {
      await invoke("store_api_key", { provider, key });
      setHasKey(true);
      setSaved(true);
      setKey("••••••••••••••••");
      setTimeout(() => setSaved(false), 2000);
    }
  };

  const handleDelete = async () => {
    await invoke("delete_api_key", { provider });
    setHasKey(false);
    setKey("");
  };

  return (
    <div className="api-key-input">
      <label>{label}</label>
      <div className="input-row">
        <input
          type="password"
          value={key}
          onChange={(e) => setKey(e.target.value)}
          placeholder={`Enter ${label}`}
          onFocus={() => { if (hasKey) setKey(""); }}
        />
        <button onClick={handleSave} disabled={!key || key.startsWith("••")}>
          {saved ? "Saved" : "Save"}
        </button>
        {hasKey && <button onClick={handleDelete} className="danger">Remove</button>}
      </div>
    </div>
  );
}
```

**Step 3: Create SettingsPanel component**

```tsx
// src/components/SettingsPanel/SettingsPanel.tsx
import { useEffect } from "react";
import { useSettingsStore, type AppSettings } from "../../stores/settingsStore";
import { ApiKeyInput } from "./ApiKeyInput";

export function SettingsPanel() {
  const { settings, loading, load, save } = useSettingsStore();

  useEffect(() => { load(); }, [load]);

  if (loading || !settings) return <div className="loading">Loading...</div>;

  const update = (partial: Partial<AppSettings>) => {
    save({ ...settings, ...partial });
  };

  return (
    <div className="settings-panel">
      <h1>Prompter Settings</h1>

      <section>
        <h2>API Keys</h2>
        <ApiKeyInput provider="openai" label="OpenAI API Key" />
        <ApiKeyInput provider="claude" label="Claude API Key" />
      </section>

      <section>
        <h2>Speech-to-Text</h2>
        <label>Whisper Mode</label>
        <select value={settings.whisper_mode} onChange={(e) => update({ whisper_mode: e.target.value as any })}>
          <option value="Cloud">Cloud (OpenAI API)</option>
          <option value="Local">Local (on-device)</option>
        </select>

        {settings.whisper_mode === "Local" && (
          <>
            <label>Local Model</label>
            <select value={settings.local_whisper_model} onChange={(e) => update({ local_whisper_model: e.target.value })}>
              <option value="tiny">Tiny (fastest, least accurate)</option>
              <option value="base">Base</option>
              <option value="small">Small</option>
              <option value="medium">Medium (recommended)</option>
              <option value="large">Large (slowest, most accurate)</option>
            </select>
          </>
        )}
      </section>

      <section>
        <h2>Prompt Generation</h2>
        <label>Model</label>
        <select value={settings.prompt_model} onChange={(e) => update({ prompt_model: e.target.value as any })}>
          <option value="ClaudeSonnet">Claude Sonnet</option>
          <option value="ClaudeOpus">Claude Opus</option>
          <option value="ClaudeHaiku">Claude Haiku</option>
          <option value="Gpt4o">GPT-4o</option>
          <option value="Gpt4oMini">GPT-4o Mini</option>
        </select>
      </section>

      <section>
        <h2>Behavior</h2>
        <label>Paste Mode</label>
        <select value={settings.paste_mode} onChange={(e) => update({ paste_mode: e.target.value as any })}>
          <option value="Preview">Preview first</option>
          <option value="Auto">Auto-paste</option>
        </select>

        <label>Recording Mode</label>
        <select value={settings.recording_mode} onChange={(e) => update({ recording_mode: e.target.value as any })}>
          <option value="Toggle">Toggle (press to start/stop)</option>
          <option value="PushToTalk">Push-to-talk (hold to record)</option>
          <option value="AutoDetect">Auto-detect silence</option>
        </select>

        {settings.recording_mode === "AutoDetect" && (
          <>
            <label>Silence Threshold: {settings.silence_threshold_secs}s</label>
            <input
              type="range"
              min="1"
              max="10"
              step="0.5"
              value={settings.silence_threshold_secs}
              onChange={(e) => update({ silence_threshold_secs: parseFloat(e.target.value) })}
            />
          </>
        )}
      </section>

      <section>
        <h2>Global Hotkey</h2>
        <label>Shortcut</label>
        <input
          type="text"
          value={settings.global_hotkey}
          onChange={(e) => update({ global_hotkey: e.target.value })}
          placeholder="e.g., CmdOrCtrl+Shift+P"
        />
      </section>

      <section>
        <h2>Appearance</h2>
        <label>Theme</label>
        <select value={settings.theme} onChange={(e) => update({ theme: e.target.value as any })}>
          <option value="System">System</option>
          <option value="Light">Light</option>
          <option value="Dark">Dark</option>
        </select>
      </section>
    </div>
  );
}
```

**Step 4: Create index.ts**

```typescript
// src/components/SettingsPanel/index.ts
export { SettingsPanel } from "./SettingsPanel";
```

**Step 5: Update App.tsx to show Settings Panel**

```tsx
// src/App.tsx
import { SettingsPanel } from "./components/SettingsPanel";

function App() {
  return <SettingsPanel />;
}

export default App;
```

**Step 6: Verify it renders**

```bash
npm run tauri dev
```

Expected: Settings panel renders with all form fields. Saving settings persists to SQLite. API keys save to Keychain.

**Step 7: Commit**

```bash
git add src/components/SettingsPanel/ src/stores/ src/App.tsx
git commit -m "feat: add settings panel UI with API key management"
```

---

### Task 7: Clipboard Manager

**Files:**
- Create: `src-tauri/src/clipboard/mod.rs`
- Create: `src-tauri/src/clipboard/manager.rs`

**Step 1: Write failing tests**

```rust
// src-tauri/src/clipboard/manager.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_and_restore_clipboard() {
        let mut manager = ClipboardManager::new();
        // Set something known on the clipboard
        manager.clipboard.set_text("original content").unwrap();
        // Save it
        manager.save_current().unwrap();
        // Overwrite clipboard
        manager.clipboard.set_text("new content").unwrap();
        // Restore
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
```

**Step 2: Run tests to verify they fail**

```bash
cd src-tauri && cargo test clipboard -- --nocapture
```

Expected: FAIL.

**Step 3: Implement ClipboardManager**

```rust
// src-tauri/src/clipboard/manager.rs
use arboard::Clipboard;

pub struct ClipboardManager {
    clipboard: Clipboard,
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
```

**Step 4: Wire up mod.rs**

```rust
// src-tauri/src/clipboard/mod.rs
pub mod manager;
```

**Step 5: Run tests to verify they pass**

```bash
cd src-tauri && cargo test clipboard -- --nocapture
```

Expected: All 3 tests PASS.

**Step 6: Commit**

```bash
git add src-tauri/src/clipboard/
git commit -m "feat: add clipboard manager with save/restore for paste-back flow"
```

---

### Task 8: App Focus Tracker (macOS)

**Files:**
- Create: `src-tauri/src/focus/mod.rs`
- Create: `src-tauri/src/focus/tracker.rs`

**Step 1: Write failing test**

```rust
// src-tauri/src/focus/tracker.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_frontmost_app_returns_something() {
        // This is an integration test — it should return whatever app
        // is currently focused (likely the terminal running tests)
        let app = FocusTracker::get_frontmost_app();
        assert!(app.is_ok());
        let info = app.unwrap();
        assert!(!info.bundle_id.is_empty() || !info.name.is_empty());
    }
}
```

**Step 2: Run test to verify it fails**

```bash
cd src-tauri && cargo test focus -- --nocapture
```

Expected: FAIL.

**Step 3: Implement FocusTracker using macOS NSWorkspace**

```rust
// src-tauri/src/focus/tracker.rs
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
    /// Get the frontmost (active) application using osascript.
    pub fn get_frontmost_app() -> Result<AppInfo, Box<dyn std::error::Error>> {
        let output = Command::new("osascript")
            .arg("-e")
            .arg(r#"
                tell application "System Events"
                    set frontApp to first application process whose frontmost is true
                    set appName to name of frontApp
                    set appID to bundle identifier of frontApp
                    set appPID to unix id of frontApp
                    return appName & "|" & appID & "|" & (appPID as text)
                end tell
            "#)
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

    /// Activate (bring to front) an app by bundle ID
    pub fn activate_app(bundle_id: &str) -> Result<(), Box<dyn std::error::Error>> {
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
```

**Step 4: Wire up mod.rs**

```rust
// src-tauri/src/focus/mod.rs
pub mod tracker;
```

**Step 5: Run tests to verify they pass**

```bash
cd src-tauri && cargo test focus -- --nocapture
```

Expected: PASS. Returns info about the terminal app running the tests.

**Step 6: Commit**

```bash
git add src-tauri/src/focus/
git commit -m "feat: add macOS focus tracker to detect and activate apps"
```

---

### Task 9: Audio Recorder

**Files:**
- Create: `src-tauri/src/audio/mod.rs`
- Create: `src-tauri/src/audio/recorder.rs`

**Step 1: Write failing tests**

```rust
// src-tauri/src/audio/recorder.rs
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use std::thread;

    #[test]
    fn test_recorder_starts_and_stops() {
        let recorder = AudioRecorder::new().unwrap();
        recorder.start().unwrap();
        thread::sleep(Duration::from_millis(500));
        let audio_data = recorder.stop().unwrap();
        // Should have captured some samples (even if silence)
        assert!(!audio_data.samples.is_empty());
        assert!(audio_data.sample_rate > 0);
    }

    #[test]
    fn test_recorder_returns_valid_sample_rate() {
        let recorder = AudioRecorder::new().unwrap();
        recorder.start().unwrap();
        thread::sleep(Duration::from_millis(200));
        let audio_data = recorder.stop().unwrap();
        // Common sample rates: 44100, 48000, etc.
        assert!(audio_data.sample_rate >= 8000);
    }
}
```

**Step 2: Run tests to verify they fail**

```bash
cd src-tauri && cargo test audio -- --nocapture
```

Expected: FAIL.

**Step 3: Implement AudioRecorder**

```rust
// src-tauri/src/audio/recorder.rs
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct AudioData {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
}

pub struct AudioRecorder {
    samples: Arc<Mutex<Vec<f32>>>,
    stream: Mutex<Option<cpal::Stream>>,
    sample_rate: Mutex<u32>,
    channels: Mutex<u16>,
}

impl AudioRecorder {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            samples: Arc::new(Mutex::new(Vec::new())),
            stream: Mutex::new(None),
            sample_rate: Mutex::new(0),
            channels: Mutex::new(0),
        })
    }

    pub fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let device = host.default_input_device()
            .ok_or("No input device available")?;
        let config = device.default_input_config()?;

        *self.sample_rate.lock().unwrap() = config.sample_rate().0;
        *self.channels.lock().unwrap() = config.channels();

        let samples = Arc::clone(&self.samples);
        samples.lock().unwrap().clear();

        let stream = device.build_input_stream(
            &config.into(),
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                samples.lock().unwrap().extend_from_slice(data);
            },
            |err| eprintln!("Audio stream error: {}", err),
            None,
        )?;

        stream.play()?;
        *self.stream.lock().unwrap() = Some(stream);
        Ok(())
    }

    pub fn stop(&self) -> Result<AudioData, Box<dyn std::error::Error>> {
        // Drop the stream to stop recording
        *self.stream.lock().unwrap() = None;

        let samples = self.samples.lock().unwrap().clone();
        let sample_rate = *self.sample_rate.lock().unwrap();
        let channels = *self.channels.lock().unwrap();

        Ok(AudioData {
            samples,
            sample_rate,
            channels,
        })
    }
}
```

**Step 4: Add WAV export utility**

```rust
// Append to src-tauri/src/audio/recorder.rs

impl AudioData {
    /// Convert to mono 16kHz WAV bytes (required by Whisper)
    pub fn to_wav_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mono_samples: Vec<f32> = if self.channels > 1 {
            self.samples
                .chunks(self.channels as usize)
                .map(|chunk| chunk.iter().sum::<f32>() / chunk.len() as f32)
                .collect()
        } else {
            self.samples.clone()
        };

        // Resample to 16kHz if needed (simple linear interpolation)
        let target_rate = 16000u32;
        let resampled = if self.sample_rate != target_rate {
            let ratio = self.sample_rate as f64 / target_rate as f64;
            let new_len = (mono_samples.len() as f64 / ratio) as usize;
            (0..new_len)
                .map(|i| {
                    let src_idx = i as f64 * ratio;
                    let idx = src_idx as usize;
                    let frac = src_idx - idx as f64;
                    let s1 = mono_samples.get(idx).copied().unwrap_or(0.0);
                    let s2 = mono_samples.get(idx + 1).copied().unwrap_or(s1);
                    s1 + (s2 - s1) * frac as f32
                })
                .collect()
        } else {
            mono_samples
        };

        let mut cursor = std::io::Cursor::new(Vec::new());
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: target_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::new(&mut cursor, spec)?;
        for sample in &resampled {
            let s = (sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
            writer.write_sample(s)?;
        }
        writer.finalize()?;
        Ok(cursor.into_inner())
    }
}
```

**Step 5: Wire up mod.rs**

```rust
// src-tauri/src/audio/mod.rs
pub mod recorder;
```

**Step 6: Run tests to verify they pass**

```bash
cd src-tauri && cargo test audio -- --nocapture
```

Expected: PASS (requires microphone permission on macOS — may prompt).

**Step 7: Commit**

```bash
git add src-tauri/src/audio/
git commit -m "feat: add audio recorder with mic capture and WAV export for Whisper"
```

---

### Task 10: Whisper Engine — Cloud (OpenAI API)

**Files:**
- Create: `src-tauri/src/whisper/mod.rs`
- Create: `src-tauri/src/whisper/cloud.rs`

**Step 1: Write failing test**

```rust
// src-tauri/src/whisper/cloud.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cloud_whisper_builds_request() {
        let whisper = CloudWhisper::new("sk-test-key".to_string());
        // Test that the struct initializes correctly
        assert_eq!(whisper.api_key, "sk-test-key");
    }

    #[tokio::test]
    async fn test_cloud_whisper_rejects_empty_audio() {
        let whisper = CloudWhisper::new("sk-test-key".to_string());
        let result = whisper.transcribe(&[]).await;
        assert!(result.is_err());
    }
}
```

**Step 2: Run tests to verify they fail**

```bash
cd src-tauri && cargo test whisper -- --nocapture
```

Expected: FAIL.

**Step 3: Implement CloudWhisper**

```rust
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
```

**Step 4: Wire up mod.rs**

```rust
// src-tauri/src/whisper/mod.rs
pub mod cloud;
```

**Step 5: Run tests to verify they pass**

```bash
cd src-tauri && cargo test whisper -- --nocapture
```

Expected: Both tests PASS (no actual API call made).

**Step 6: Commit**

```bash
git add src-tauri/src/whisper/
git commit -m "feat: add cloud Whisper transcription via OpenAI API"
```

---

### Task 11: Whisper Engine — Local (whisper-rs)

**Files:**
- Create: `src-tauri/src/whisper/local.rs`
- Modify: `src-tauri/src/whisper/mod.rs`

**Step 1: Write failing tests**

```rust
// src-tauri/src/whisper/local.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_path_construction() {
        let path = LocalWhisper::model_path("medium");
        assert!(path.ends_with("ggml-medium.bin"));
        assert!(path.contains("prompter"));
    }

    #[test]
    fn test_model_url_construction() {
        let url = LocalWhisper::model_url("medium");
        assert!(url.contains("ggml-medium.bin"));
        assert!(url.starts_with("https://"));
    }
}
```

**Step 2: Run tests to verify they fail**

```bash
cd src-tauri && cargo test whisper::local -- --nocapture
```

Expected: FAIL.

**Step 3: Implement LocalWhisper**

```rust
// src-tauri/src/whisper/local.rs
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};
use std::path::PathBuf;

pub struct LocalWhisper {
    model_name: String,
}

impl LocalWhisper {
    pub fn new(model_name: &str) -> Self {
        Self {
            model_name: model_name.to_string(),
        }
    }

    pub fn model_path(model_name: &str) -> String {
        let data_dir = dirs::data_dir().unwrap().join("com.prompter.app").join("models");
        data_dir.join(format!("ggml-{}.bin", model_name)).to_string_lossy().to_string()
    }

    pub fn model_url(model_name: &str) -> String {
        format!(
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-{}.bin",
            model_name
        )
    }

    pub fn is_model_downloaded(&self) -> bool {
        PathBuf::from(Self::model_path(&self.model_name)).exists()
    }

    pub async fn download_model(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = PathBuf::from(Self::model_path(&self.model_name));
        if path.exists() {
            return Ok(());
        }

        std::fs::create_dir_all(path.parent().unwrap())?;
        let url = Self::model_url(&self.model_name);

        let client = reqwest::Client::new();
        let response = client.get(&url).send().await?;
        let bytes = response.bytes().await?;
        std::fs::write(&path, &bytes)?;
        Ok(())
    }

    pub fn transcribe(&self, wav_bytes: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
        if wav_bytes.is_empty() {
            return Err("Empty audio data".into());
        }

        let model_path = Self::model_path(&self.model_name);
        if !PathBuf::from(&model_path).exists() {
            return Err("Model not downloaded. Please download the model first.".into());
        }

        let ctx = WhisperContext::new_with_params(&model_path, WhisperContextParameters::default())?;
        let mut state = ctx.create_state()?;

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_language(Some("en"));
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        // Decode WAV bytes to f32 samples
        let reader = hound::WavReader::new(std::io::Cursor::new(wav_bytes))?;
        let samples: Vec<f32> = reader
            .into_samples::<i16>()
            .filter_map(|s| s.ok())
            .map(|s| s as f32 / 32768.0)
            .collect();

        state.full(params, &samples)?;

        let num_segments = state.full_n_segments()?;
        let mut text = String::new();
        for i in 0..num_segments {
            if let Ok(segment) = state.full_get_segment_text(i) {
                text.push_str(&segment);
            }
        }

        Ok(text.trim().to_string())
    }
}
```

**Step 4: Update mod.rs**

```rust
// src-tauri/src/whisper/mod.rs
pub mod cloud;
pub mod local;
```

**Step 5: Run tests to verify they pass**

```bash
cd src-tauri && cargo test whisper -- --nocapture
```

Expected: All whisper tests PASS.

**Step 6: Commit**

```bash
git add src-tauri/src/whisper/
git commit -m "feat: add local Whisper transcription via whisper-rs with model download"
```

---

### Task 12: AI Prompt Generator

**Files:**
- Create: `src-tauri/src/prompt/mod.rs`
- Create: `src-tauri/src/prompt/generator.rs`

**Step 1: Write failing tests**

```rust
// src-tauri/src/prompt/generator.rs
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
        assert_eq!(messages.len(), 2); // system + user
        assert!(messages[1]["content"].as_str().unwrap().contains("fn add"));
        assert!(messages[1]["content"].as_str().unwrap().contains("write tests"));
    }

    #[test]
    fn test_build_messages_no_selection() {
        let gen = PromptGenerator::new("claude", "sk-test");
        let messages = gen.build_messages(None, Some("create a new react component"), "VS Code");
        assert_eq!(messages.len(), 2);
        assert!(!messages[1]["content"].as_str().unwrap().contains("Selected text"));
    }

    #[test]
    fn test_build_messages_no_speech() {
        let gen = PromptGenerator::new("claude", "sk-test");
        let messages = gen.build_messages(Some("broken code here"), None, "Terminal");
        assert_eq!(messages.len(), 2);
        assert!(messages[1]["content"].as_str().unwrap().contains("help me with this"));
    }
}
```

**Step 2: Run tests to verify they fail**

```bash
cd src-tauri && cargo test prompt -- --nocapture
```

Expected: FAIL.

**Step 3: Implement PromptGenerator**

```rust
// src-tauri/src/prompt/generator.rs
use serde_json::json;

const SYSTEM_PROMPT: &str = r#"You are a prompt engineer. The user has selected content from their coding tool and described what they want to do with it via voice.

Generate a clear, actionable prompt that the user can paste directly into an AI coding assistant (Claude Code, Cursor, etc.).

Rules:
- Be specific and unambiguous
- Include relevant context from the selected text
- Structure the prompt for best AI response
- Match the complexity of the request (simple ask = short prompt)
- Output ONLY the prompt text, no explanations or meta-commentary"#;

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
            user_content.push_str(&format!("Selected text:\n```\n{}\n```\n\n", text));
        }

        let intent = spoken_intent.unwrap_or("help me with this");
        user_content.push_str(&format!("User's spoken intent: {}\n\n", intent));
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
        let client = reqwest::Client::new();

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

        let json: serde_json::Value = response.json().await?;
        let text = json["content"][0]["text"]
            .as_str()
            .unwrap_or("Failed to generate prompt")
            .to_string();
        Ok(text)
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

        let json: serde_json::Value = response.json().await?;
        let text = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("Failed to generate prompt")
            .to_string();
        Ok(text)
    }
}
```

**Step 4: Wire up mod.rs**

```rust
// src-tauri/src/prompt/mod.rs
pub mod generator;
```

**Step 5: Run tests to verify they pass**

```bash
cd src-tauri && cargo test prompt -- --nocapture
```

Expected: All 3 tests PASS.

**Step 6: Commit**

```bash
git add src-tauri/src/prompt/
git commit -m "feat: add AI prompt generator with Claude and OpenAI support"
```

---

### Task 13: Global Hotkey Manager

**Files:**
- Create: `src-tauri/src/hotkey/mod.rs`
- Create: `src-tauri/src/hotkey/manager.rs`

**Step 1: Write failing test**

```rust
// src-tauri/src/hotkey/manager.rs
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
}
```

**Step 2: Run tests to verify they fail**

```bash
cd src-tauri && cargo test hotkey -- --nocapture
```

Expected: FAIL.

**Step 3: Implement HotkeyManager**

Note: Tauri 2.x has built-in global shortcut support via the `tauri-plugin-global-shortcut` plugin. Use that instead of the raw `global-hotkey` crate for better integration.

```bash
cd src-tauri && cargo add tauri-plugin-global-shortcut@2
```

```rust
// src-tauri/src/hotkey/manager.rs
use tauri::AppHandle;

pub struct HotkeyManager;

impl HotkeyManager {
    pub fn parse_hotkey(shortcut: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Validate the shortcut string format
        let parts: Vec<&str> = shortcut.split('+').collect();
        if parts.is_empty() {
            return Err("Empty hotkey string".into());
        }

        let valid_modifiers = ["CmdOrCtrl", "Cmd", "Ctrl", "Alt", "Shift", "Super"];
        let key = parts.last().ok_or("No key specified")?;

        // All parts except last should be modifiers
        for part in &parts[..parts.len() - 1] {
            if !valid_modifiers.contains(part) {
                return Err(format!("Invalid modifier: {}", part).into());
            }
        }

        // Key should be a single character or known key name
        if key.is_empty() {
            return Err("Empty key".into());
        }

        Ok(shortcut.to_string())
    }

    pub fn register(
        app: &AppHandle,
        shortcut: &str,
        handler: impl Fn() + Send + Sync + 'static,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use tauri_plugin_global_shortcut::GlobalShortcutExt;

        let shortcut_str = Self::parse_hotkey(shortcut)?;
        app.global_shortcut().on_shortcut(
            shortcut_str.parse()?,
            move |_app, _shortcut, _event| {
                handler();
            },
        )?;

        Ok(())
    }

    pub fn unregister_all(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
        use tauri_plugin_global_shortcut::GlobalShortcutExt;
        app.global_shortcut().unregister_all()?;
        Ok(())
    }
}
```

**Step 4: Wire up mod.rs**

```rust
// src-tauri/src/hotkey/mod.rs
pub mod manager;
```

**Step 5: Run tests to verify they pass**

```bash
cd src-tauri && cargo test hotkey -- --nocapture
```

Expected: PASS.

**Step 6: Commit**

```bash
git add src-tauri/src/hotkey/
git commit -m "feat: add global hotkey manager with shortcut parsing and registration"
```

---

### Task 14: Tauri IPC Commands — Core Workflow

**Files:**
- Create: `src-tauri/src/commands/recording.rs`
- Create: `src-tauri/src/commands/prompt.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/commands/settings.rs` (extend AppState)
- Modify: `src-tauri/src/lib.rs`

**Step 1: Extend AppState with all managers**

Update `src-tauri/src/commands/settings.rs` AppState:

```rust
pub struct AppState {
    pub settings_manager: Mutex<SettingsManager>,
    pub keychain: KeychainManager,
    pub clipboard: Mutex<ClipboardManager>,
    pub audio_recorder: Arc<AudioRecorder>,
    pub db: Mutex<Database>,
}
```

**Step 2: Create recording commands**

```rust
// src-tauri/src/commands/recording.rs
use tauri::State;
use crate::commands::settings::AppState;
use crate::focus::tracker::FocusTracker;

#[tauri::command]
pub async fn start_recording(state: State<'_, AppState>) -> Result<String, String> {
    // Capture frontmost app before we take focus
    let app_info = FocusTracker::get_frontmost_app().map_err(|e| e.to_string())?;

    // Save current clipboard
    state.clipboard.lock().unwrap().save_current().map_err(|e| e.to_string())?;

    // Simulate Cmd+C to copy selection (small delay for OS to process)
    std::process::Command::new("osascript")
        .arg("-e")
        .arg(r#"tell application "System Events" to keystroke "c" using command down"#)
        .output()
        .map_err(|e| e.to_string())?;

    std::thread::sleep(std::time::Duration::from_millis(100));

    // Read clipboard (selected text)
    let selected_text = state.clipboard.lock().unwrap().read().unwrap_or_default();

    // Start audio recording
    state.audio_recorder.start().map_err(|e| e.to_string())?;

    // Return source app info + selected text as JSON
    Ok(serde_json::json!({
        "source_app": app_info,
        "selected_text": selected_text,
    }).to_string())
}

#[tauri::command]
pub async fn stop_recording(state: State<'_, AppState>) -> Result<String, String> {
    let audio_data = state.audio_recorder.stop().map_err(|e| e.to_string())?;
    let wav_bytes = audio_data.to_wav_bytes().map_err(|e| e.to_string())?;

    // Get settings for whisper mode
    let settings = state.settings_manager.lock().unwrap().load().map_err(|e| e.to_string())?;

    let transcription = match settings.whisper_mode {
        crate::settings::types::WhisperMode::Cloud => {
            let api_key = state.keychain.retrieve("openai").map_err(|e| e.to_string())?
                .ok_or("OpenAI API key not configured")?;
            let whisper = crate::whisper::cloud::CloudWhisper::new(api_key);
            whisper.transcribe(&wav_bytes).await.map_err(|e| e.to_string())?
        }
        crate::settings::types::WhisperMode::Local => {
            let whisper = crate::whisper::local::LocalWhisper::new(&settings.local_whisper_model);
            whisper.transcribe(&wav_bytes).map_err(|e| e.to_string())?
        }
    };

    Ok(transcription)
}
```

**Step 3: Create prompt generation commands**

```rust
// src-tauri/src/commands/prompt.rs
use tauri::State;
use crate::commands::settings::AppState;
use crate::prompt::generator::PromptGenerator;
use crate::settings::types::PromptModel;

#[tauri::command]
pub async fn generate_prompt(
    state: State<'_, AppState>,
    selected_text: Option<String>,
    spoken_intent: Option<String>,
    source_app: String,
) -> Result<String, String> {
    let settings = state.settings_manager.lock().unwrap().load().map_err(|e| e.to_string())?;

    let (provider, key_name) = match settings.prompt_model {
        PromptModel::ClaudeSonnet | PromptModel::ClaudeOpus | PromptModel::ClaudeHaiku => ("claude", "claude"),
        PromptModel::Gpt4o | PromptModel::Gpt4oMini => ("openai", "openai"),
    };

    let api_key = state.keychain.retrieve(key_name).map_err(|e| e.to_string())?
        .ok_or(format!("{} API key not configured", provider))?;

    let generator = PromptGenerator::new(provider, &api_key);
    let prompt = generator.generate(
        selected_text.as_deref(),
        spoken_intent.as_deref(),
        &source_app,
    ).await.map_err(|e| e.to_string())?;

    // Save to history
    state.db.lock().unwrap().save_prompt(
        selected_text.as_deref().unwrap_or(""),
        spoken_intent.as_deref().unwrap_or(""),
        &prompt,
        &source_app,
    ).map_err(|e| e.to_string())?;

    Ok(prompt)
}

#[tauri::command]
pub async fn paste_to_app(
    state: State<'_, AppState>,
    text: String,
    bundle_id: String,
) -> Result<(), String> {
    // Write prompt to clipboard
    state.clipboard.lock().unwrap().write(&text).map_err(|e| e.to_string())?;

    // Activate source app
    crate::focus::tracker::FocusTracker::activate_app(&bundle_id).map_err(|e| e.to_string())?;
    std::thread::sleep(std::time::Duration::from_millis(200));

    // Simulate Cmd+V to paste
    std::process::Command::new("osascript")
        .arg("-e")
        .arg(r#"tell application "System Events" to keystroke "v" using command down"#)
        .output()
        .map_err(|e| e.to_string())?;

    // Restore original clipboard after a delay
    std::thread::sleep(std::time::Duration::from_millis(500));
    state.clipboard.lock().unwrap().restore().map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn get_prompt_history(state: State<AppState>, limit: i64) -> Result<String, String> {
    let history = state.db.lock().unwrap().list_prompts(limit, 0).map_err(|e| e.to_string())?;
    serde_json::to_string(&history).map_err(|e| e.to_string())
}
```

**Step 4: Update commands/mod.rs**

```rust
// src-tauri/src/commands/mod.rs
pub mod settings;
pub mod recording;
pub mod prompt;
```

**Step 5: Update lib.rs to register all commands**

Update the `run()` function in `src-tauri/src/lib.rs` to include the new commands and state fields. Add the `AudioRecorder` and a second `Database` to AppState, and register all commands.

**Step 6: Verify it compiles**

```bash
cd src-tauri && cargo build
```

Expected: Compiles without errors.

**Step 7: Commit**

```bash
git add src-tauri/src/commands/ src-tauri/src/lib.rs
git commit -m "feat: add Tauri IPC commands for recording, transcription, and prompt generation"
```

---

### Task 15: Floating Widget UI

**Files:**
- Create: `src/components/FloatingWidget/FloatingWidget.tsx`
- Create: `src/components/FloatingWidget/RecordingView.tsx`
- Create: `src/components/FloatingWidget/PreviewView.tsx`
- Create: `src/components/FloatingWidget/index.ts`
- Create: `src/stores/recordingStore.ts`
- Modify: `src/App.tsx`

**Step 1: Create recording Zustand store**

```typescript
// src/stores/recordingStore.ts
import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

type RecordingState = "idle" | "recording" | "transcribing" | "generating" | "preview" | "pasting";

interface SourceApp {
  bundle_id: string;
  name: string;
  pid: number;
}

interface RecordingStore {
  state: RecordingState;
  selectedText: string;
  transcription: string;
  generatedPrompt: string;
  sourceApp: SourceApp | null;
  error: string | null;

  startRecording: () => Promise<void>;
  stopRecording: () => Promise<void>;
  generatePrompt: () => Promise<void>;
  pastePrompt: (text: string) => Promise<void>;
  reset: () => void;
}

export const useRecordingStore = create<RecordingStore>((set, get) => ({
  state: "idle",
  selectedText: "",
  transcription: "",
  generatedPrompt: "",
  sourceApp: null,
  error: null,

  startRecording: async () => {
    try {
      set({ state: "recording", error: null });
      const result = await invoke<string>("start_recording");
      const data = JSON.parse(result);
      set({
        selectedText: data.selected_text,
        sourceApp: data.source_app,
      });
    } catch (e) {
      set({ state: "idle", error: String(e) });
    }
  },

  stopRecording: async () => {
    try {
      set({ state: "transcribing" });
      const transcription = await invoke<string>("stop_recording");
      set({ transcription, state: "generating" });
      await get().generatePrompt();
    } catch (e) {
      set({ state: "idle", error: String(e) });
    }
  },

  generatePrompt: async () => {
    try {
      set({ state: "generating" });
      const { selectedText, transcription, sourceApp } = get();
      const prompt = await invoke<string>("generate_prompt", {
        selectedText: selectedText || null,
        spokenIntent: transcription || null,
        sourceApp: sourceApp?.name || "Unknown",
      });
      set({ generatedPrompt: prompt, state: "preview" });
    } catch (e) {
      set({ state: "idle", error: String(e) });
    }
  },

  pastePrompt: async (text: string) => {
    try {
      set({ state: "pasting" });
      const { sourceApp } = get();
      if (sourceApp) {
        await invoke("paste_to_app", {
          text,
          bundleId: sourceApp.bundle_id,
        });
      }
      set({ state: "idle" });
      get().reset();
    } catch (e) {
      set({ state: "idle", error: String(e) });
    }
  },

  reset: () => set({
    state: "idle",
    selectedText: "",
    transcription: "",
    generatedPrompt: "",
    sourceApp: null,
    error: null,
  }),
}));
```

**Step 2: Create RecordingView component**

```tsx
// src/components/FloatingWidget/RecordingView.tsx
import { useRecordingStore } from "../../stores/recordingStore";

export function RecordingView() {
  const { state, stopRecording } = useRecordingStore();

  return (
    <div className="recording-view">
      <div className="recording-indicator">
        <div className="pulse-dot" />
        <span>
          {state === "recording" && "Recording... Speak your intent"}
          {state === "transcribing" && "Transcribing..."}
          {state === "generating" && "Generating prompt..."}
        </span>
      </div>
      {state === "recording" && (
        <button onClick={stopRecording} className="stop-btn">
          Stop Recording
        </button>
      )}
    </div>
  );
}
```

**Step 3: Create PreviewView component**

```tsx
// src/components/FloatingWidget/PreviewView.tsx
import { useState } from "react";
import { useRecordingStore } from "../../stores/recordingStore";

export function PreviewView() {
  const { generatedPrompt, pastePrompt, reset } = useRecordingStore();
  const [editedPrompt, setEditedPrompt] = useState(generatedPrompt);

  return (
    <div className="preview-view">
      <h3>Generated Prompt</h3>
      <textarea
        value={editedPrompt}
        onChange={(e) => setEditedPrompt(e.target.value)}
        rows={10}
      />
      <div className="actions">
        <button onClick={() => pastePrompt(editedPrompt)} className="primary">
          Paste to App
        </button>
        <button onClick={() => navigator.clipboard.writeText(editedPrompt)}>
          Copy
        </button>
        <button onClick={reset}>Cancel</button>
      </div>
    </div>
  );
}
```

**Step 4: Create FloatingWidget container**

```tsx
// src/components/FloatingWidget/FloatingWidget.tsx
import { useRecordingStore } from "../../stores/recordingStore";
import { RecordingView } from "./RecordingView";
import { PreviewView } from "./PreviewView";

export function FloatingWidget() {
  const { state, error } = useRecordingStore();

  if (state === "idle") return null;

  return (
    <div className="floating-widget">
      {error && <div className="error">{error}</div>}
      {(state === "recording" || state === "transcribing" || state === "generating") && (
        <RecordingView />
      )}
      {state === "preview" && <PreviewView />}
      {state === "pasting" && <div className="pasting">Pasting...</div>}
    </div>
  );
}
```

**Step 5: Create index.ts**

```typescript
// src/components/FloatingWidget/index.ts
export { FloatingWidget } from "./FloatingWidget";
```

**Step 6: Update App.tsx to route between settings and widget**

```tsx
// src/App.tsx
import { useEffect, useState } from "react";
import { SettingsPanel } from "./components/SettingsPanel";
import { FloatingWidget } from "./components/FloatingWidget";
import { listen } from "@tauri-apps/api/event";
import { useRecordingStore } from "./stores/recordingStore";

function App() {
  const [view, setView] = useState<"widget" | "settings">("widget");
  const { startRecording } = useRecordingStore();

  useEffect(() => {
    const unlisten = listen("hotkey-triggered", () => {
      startRecording();
    });
    const unlistenSettings = listen("open-settings", () => {
      setView("settings");
    });
    return () => {
      unlisten.then((fn) => fn());
      unlistenSettings.then((fn) => fn());
    };
  }, [startRecording]);

  if (view === "settings") {
    return (
      <div>
        <button onClick={() => setView("widget")}>Back</button>
        <SettingsPanel />
      </div>
    );
  }

  return <FloatingWidget />;
}

export default App;
```

**Step 7: Verify it renders**

```bash
npm run tauri dev
```

Expected: App runs. Floating widget appears when recording state changes.

**Step 8: Commit**

```bash
git add src/components/FloatingWidget/ src/stores/recordingStore.ts src/App.tsx
git commit -m "feat: add floating widget UI with recording, transcription, and preview views"
```

---

### Task 16: System Tray / Menu Bar

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/tauri.conf.json`

**Step 1: Add system tray configuration to tauri.conf.json**

Ensure the `tray-icon` feature is enabled in `tauri.conf.json` (Tauri 2.x uses the plugin system). Add a tray icon image to `src-tauri/icons/tray-icon.png` (32x32 PNG).

**Step 2: Implement system tray in lib.rs**

```rust
// Add to src-tauri/src/lib.rs run() function, before .run()

use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};

// Inside the Builder chain, after .invoke_handler():
.setup(|app| {
    // Build tray menu
    let start = MenuItem::with_id(app, "start", "Start Recording", true, None::<&str>)?;
    let history = MenuItem::with_id(app, "history", "Recent Prompts", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&start, &history, &settings, &quit])?;

    let _tray = TrayIconBuilder::new()
        .menu(&menu)
        .icon(app.default_window_icon().unwrap().clone())
        .on_menu_event(move |app, event| {
            match event.id.as_ref() {
                "start" => {
                    let _ = app.emit("hotkey-triggered", ());
                }
                "settings" => {
                    let _ = app.emit("open-settings", ());
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .build(app)?;

    // Register global hotkey
    let app_handle = app.handle().clone();
    let settings = app.state::<AppState>()
        .settings_manager.lock().unwrap()
        .load().unwrap_or_default();

    hotkey::manager::HotkeyManager::register(
        &app_handle,
        &settings.global_hotkey,
        move || {
            let _ = app_handle.emit("hotkey-triggered", ());
            if let Some(window) = app_handle.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        },
    ).unwrap_or_else(|e| eprintln!("Failed to register hotkey: {}", e));

    Ok(())
})
```

**Step 3: Verify it builds and shows tray icon**

```bash
npm run tauri dev
```

Expected: App shows in menu bar with tray icon. Clicking shows dropdown with Start Recording, Recent Prompts, Settings, Quit.

**Step 4: Commit**

```bash
git add src-tauri/src/lib.rs src-tauri/tauri.conf.json src-tauri/icons/
git commit -m "feat: add system tray menu bar with recording, settings, and quit options"
```

---

### Task 17: First-Run Onboarding

**Files:**
- Create: `src/components/Onboarding/Onboarding.tsx`
- Create: `src/components/Onboarding/index.ts`
- Create: `src-tauri/src/commands/onboarding.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src/App.tsx`

**Step 1: Add onboarding check command**

```rust
// src-tauri/src/commands/onboarding.rs
use tauri::State;
use crate::commands::settings::AppState;

#[tauri::command]
pub fn is_first_run(state: State<AppState>) -> Result<bool, String> {
    let has_run = state.db.lock().unwrap()
        .get_setting("onboarding_complete")
        .map_err(|e| e.to_string())?;
    Ok(has_run.is_none())
}

#[tauri::command]
pub fn complete_onboarding(state: State<AppState>) -> Result<(), String> {
    state.db.lock().unwrap()
        .set_setting("onboarding_complete", "true")
        .map_err(|e| e.to_string())
}
```

**Step 2: Create Onboarding component**

```tsx
// src/components/Onboarding/Onboarding.tsx
import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ApiKeyInput } from "../SettingsPanel/ApiKeyInput";

interface Props {
  onComplete: () => void;
}

export function Onboarding({ onComplete }: Props) {
  const [step, setStep] = useState(0);

  const steps = [
    {
      title: "Welcome to Prompter",
      content: (
        <div>
          <p>Prompter turns your voice + selected code into polished AI prompts.</p>
          <p>Let's get you set up in 3 quick steps.</p>
        </div>
      ),
    },
    {
      title: "Add an API Key",
      content: (
        <div>
          <p>You need at least one API key for prompt generation.</p>
          <ApiKeyInput provider="openai" label="OpenAI API Key" />
          <ApiKeyInput provider="claude" label="Claude API Key" />
        </div>
      ),
    },
    {
      title: "You're Ready!",
      content: (
        <div>
          <p>Press <kbd>Cmd+Shift+P</kbd> anywhere to start.</p>
          <p>Select text, speak your intent, and get a polished prompt.</p>
        </div>
      ),
    },
  ];

  const handleNext = async () => {
    if (step < steps.length - 1) {
      setStep(step + 1);
    } else {
      await invoke("complete_onboarding");
      onComplete();
    }
  };

  return (
    <div className="onboarding">
      <h1>{steps[step].title}</h1>
      {steps[step].content}
      <div className="nav">
        {step > 0 && <button onClick={() => setStep(step - 1)}>Back</button>}
        <button onClick={handleNext} className="primary">
          {step < steps.length - 1 ? "Next" : "Get Started"}
        </button>
      </div>
    </div>
  );
}
```

**Step 3: Create index.ts**

```typescript
// src/components/Onboarding/index.ts
export { Onboarding } from "./Onboarding";
```

**Step 4: Update App.tsx to show onboarding on first run**

```tsx
// src/App.tsx — update to check first run
import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { SettingsPanel } from "./components/SettingsPanel";
import { FloatingWidget } from "./components/FloatingWidget";
import { Onboarding } from "./components/Onboarding";
import { useRecordingStore } from "./stores/recordingStore";

function App() {
  const [view, setView] = useState<"loading" | "onboarding" | "widget" | "settings">("loading");
  const { startRecording } = useRecordingStore();

  useEffect(() => {
    invoke<boolean>("is_first_run").then((isFirst) => {
      setView(isFirst ? "onboarding" : "widget");
    });
  }, []);

  useEffect(() => {
    const unlisten = listen("hotkey-triggered", () => {
      startRecording();
      setView("widget");
    });
    const unlistenSettings = listen("open-settings", () => {
      setView("settings");
    });
    return () => {
      unlisten.then((fn) => fn());
      unlistenSettings.then((fn) => fn());
    };
  }, [startRecording]);

  if (view === "loading") return <div className="loading">Loading...</div>;
  if (view === "onboarding") return <Onboarding onComplete={() => setView("widget")} />;
  if (view === "settings") {
    return (
      <div>
        <button onClick={() => setView("widget")}>Back</button>
        <SettingsPanel />
      </div>
    );
  }

  return <FloatingWidget />;
}

export default App;
```

**Step 5: Register new commands in lib.rs**

Add `commands::onboarding::is_first_run` and `commands::onboarding::complete_onboarding` to the `invoke_handler`.

**Step 6: Verify onboarding flow**

```bash
npm run tauri dev
```

Expected: First launch shows onboarding wizard. After completing, subsequent launches go straight to the widget.

**Step 7: Commit**

```bash
git add src/components/Onboarding/ src/App.tsx src-tauri/src/commands/onboarding.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat: add first-run onboarding wizard with API key setup"
```

---

### Task 18: Styling & Polish

**Files:**
- Create: `src/styles/global.css`
- Create: `src/styles/floating-widget.css`
- Create: `src/styles/settings.css`
- Create: `src/styles/onboarding.css`
- Modify: `src/main.tsx`

**Step 1: Create base styles with dark/light theme support**

Create CSS files that style all components with a clean, native-feeling macOS aesthetic. Use CSS custom properties for theming (light/dark/system). Style the floating widget to be compact, translucent (vibrancy-like), and unobtrusive. The settings panel should be a standard preferences window layout. Import all styles in `main.tsx`.

**Step 2: Verify visual appearance**

```bash
npm run tauri dev
```

Expected: All views styled cleanly. Floating widget is compact and translucent. Settings panel looks like native macOS preferences.

**Step 3: Commit**

```bash
git add src/styles/ src/main.tsx
git commit -m "feat: add styling with dark/light theme support and macOS-native aesthetics"
```

---

### Task 19: End-to-End Integration Test

**Files:**
- No new files — manual testing

**Step 1: Build the app**

```bash
npm run tauri build
```

Expected: Produces a `.dmg` in `src-tauri/target/release/bundle/dmg/`.

**Step 2: Manual end-to-end test**

1. Launch the app — verify onboarding appears (if first run)
2. Add an API key via Settings
3. Open VS Code or any editor, select some code
4. Press `Cmd+Shift+P` — verify floating widget appears and starts recording
5. Speak an intent (e.g., "write tests for this function")
6. Verify transcription completes
7. Verify prompt is generated and shown in preview
8. Click "Paste to App" — verify prompt is pasted back into the editor
9. Check menu bar — verify tray icon shows, dropdown works
10. Check Settings — verify all options save and persist

**Step 3: Commit any fixes**

```bash
git add -A
git commit -m "fix: integration test fixes and polish"
```
