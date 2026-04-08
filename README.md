# Prompter

A macOS desktop app that captures your voice and selected code to generate polished AI prompts. Select text in any app, press a hotkey, speak your intent, and get a ready-to-paste prompt in seconds.

Built with Tauri 2, Rust, React, and TypeScript.

## How It Works

```
1. Select text in any app (code, docs, chat)
2. Press Cmd+Shift+P (or your custom hotkey)
3. Choose: Record Voice or use Text Only
4. Prompter generates a polished prompt using Claude or GPT
5. Prompt is pasted back into the source app (or copied to clipboard)
```

The app runs as a compact floating overlay with a system tray icon. It stays out of your way until you need it.

## Features

- **Voice + text to prompt** — Combines selected text with spoken intent to generate precise AI prompts
- **Voice is optional** — Use text-only mode when you don't need speech
- **Cloud or local transcription** — OpenAI Whisper API (cloud) or whisper.cpp (local, fully private)
- **Multiple AI models** — Claude Sonnet/Opus/Haiku, GPT-4o, GPT-4o Mini
- **Smart clipboard** — Captures selection via Cmd+C, restores original clipboard after paste
- **Global hotkey** — Works from any app, customizable shortcut
- **Floating overlay** — Compact, translucent, always-on-top window with native macOS feel
- **Three recording modes** — Toggle, Push-to-Talk, or Auto-detect (silence detection)
- **Auto-paste or preview** — Skip the preview and paste immediately, or review/edit first
- **Prompt history** — Searchable, with date grouping, expand/copy/delete
- **API key security** — Keys stored in macOS Keychain + local SQLite
- **Dark/Light/System theme** — Follows your macOS appearance
- **First-run onboarding** — Guided setup wizard for API keys and preferences

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Framework | [Tauri 2.x](https://v2.tauri.app/) |
| Backend | Rust |
| Frontend | React 19 + TypeScript |
| Styling | Tailwind CSS 4 + Framer Motion |
| State | Zustand |
| Database | SQLite (rusqlite) |
| Speech-to-text | Whisper (cloud: OpenAI API, local: whisper-rs) |
| Prompt generation | Claude API / OpenAI API |
| Key storage | macOS Keychain (security-framework) |
| Audio | cpal (mic capture) + hound (WAV encoding) |
| Icons | Lucide React |

## Prerequisites

- **macOS** (required — uses macOS Keychain, AppleScript for clipboard/focus)
- **Rust** (1.75+)
- **Node.js** (18+) with npm
- **cmake** (required by whisper-rs, install via `brew install cmake`)
- **API keys** — At minimum, a Claude or OpenAI key for prompt generation. If using cloud transcription, an OpenAI key.

## Getting Started

### One-command setup

```bash
make setup
```

This runs `scripts/setup.sh` which checks and installs all dependencies (Rust, Node.js, cmake, npm packages, Cargo crates).

### Manual setup

```bash
# Install npm dependencies
npm install

# Fetch Rust crates
cd src-tauri && cargo fetch && cd ..
```

### Development

```bash
make dev          # Start Tauri dev server with hot reload
```

The app opens in a native window. Use the system tray icon or the global hotkey (default: Cmd+Shift+P) to interact.

### Build

```bash
make build        # Build release .dmg
```

The .dmg is output to `src-tauri/target/release/bundle/dmg/`.

## Usage

### Core Workflow

1. **Select text** in any app (VS Code, Cursor, terminal, browser, etc.)
2. **Press Cmd+Shift+P** — Prompter captures the selection and shows the floating widget
3. **Choose an action:**
   - **Record Voice** — Speak your intent ("write tests for this", "refactor to use async", "explain this function")
   - **Text Only** — Skip voice, generate a prompt from the selected text alone
4. **Review the generated prompt** — Edit if needed, then:
   - **Paste to App** — Sends the prompt back to the source app
   - **Copy** — Copies to clipboard
   - **Cancel** — Discards and hides the widget

### System Tray

Right-click (or click) the tray icon for:
- Start Recording
- Show History
- Settings
- Quit Prompter

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Cmd+Shift+P | Trigger Prompter (default, customizable) |
| Escape | Hide window / go back |

## Configuration

Access settings via the gear icon in the widget or through the tray menu.

| Setting | Options | Default |
|---------|---------|---------|
| Whisper Mode | Cloud (OpenAI) / Local (on-device) | Cloud |
| Local Model | tiny / base / small / medium / large | medium |
| Prompt Model | Claude Sonnet / Opus / Haiku / GPT-4o / GPT-4o Mini | Claude Sonnet |
| Paste Mode | Auto (immediate) / Preview (review first) | Preview |
| Recording Mode | Toggle / Push-to-Talk / Auto-detect (silence) | Toggle |
| Silence Threshold | 1s – 10s (for Auto-detect mode) | 4s |
| Global Hotkey | Any key combination | Cmd+Shift+P |
| Theme | System / Light / Dark | System |

Hotkey changes take effect immediately — no restart required.

## Development

### Makefile Targets

```bash
# Setup
make setup            # One-command dev environment setup
make install          # Install npm + Rust dependencies

# Development
make dev              # Start Tauri dev server
make build            # Build release .dmg

# Testing
make test             # Run all Rust tests (34 unit + 5 integration)
make test-frontend    # TypeScript type check + Vite build
make smoke-test       # Verify build artifacts

# Code Quality
make fmt              # Format Rust code
make lint             # Run clippy
make check            # cargo check + frontend build

# Docker (CI)
make docker-build     # Build Docker image
make docker-test      # Run tests in Docker
make docker-ci        # Full CI pipeline in Docker

# Cleanup
make clean            # Remove dist/
make clean-all        # Remove dist/, node_modules/, target/
make clean-docker     # Remove Docker images and volumes

# Info
make info             # Show tool versions
make status           # Git status
```

### Project Structure

```
Prompter/
├── src/                          # Frontend (React + TypeScript)
│   ├── App.tsx                   # Root — routing, theme, event listeners
│   ├── App.css                   # Design system (CSS variables, glass, animations)
│   ├── stores/
│   │   ├── recordingStore.ts     # Recording workflow state machine (Zustand)
│   │   └── settingsStore.ts      # Settings state (Zustand)
│   └── components/
│       ├── FloatingWidget/
│       │   ├── FloatingWidget.tsx # Main widget container (AnimatePresence)
│       │   ├── IdleView.tsx      # Compact idle state
│       │   ├── ReadyView.tsx     # "Record Voice" / "Text Only" choice
│       │   ├── RecordingView.tsx  # Audio visualizer + timer + stop button
│       │   ├── PreviewView.tsx   # Generated prompt review/edit
│       │   └── HistoryView.tsx   # Searchable prompt history
│       ├── SettingsPanel/
│       │   ├── SettingsPanel.tsx  # Sidebar settings layout
│       │   └── ApiKeyInput.tsx   # API key field component
│       └── Onboarding/
│           └── Onboarding.tsx    # First-run setup wizard
├── src-tauri/                    # Backend (Rust)
│   ├── src/
│   │   ├── lib.rs               # App setup, tray, hotkey registration
│   │   ├── main.rs              # Entry point
│   │   ├── audio/               # Mic capture (cpal), WAV encoding, silence detection
│   │   ├── clipboard/           # Read/write/save/restore clipboard
│   │   ├── commands/            # Tauri IPC command handlers
│   │   │   ├── settings.rs      # AppState, settings CRUD, API keys, hotkey re-register
│   │   │   ├── recording.rs     # Start/stop/skip recording, context capture, audio level
│   │   │   ├── prompt.rs        # AI generation, paste, copy, history, search, delete
│   │   │   └── onboarding.rs    # First-run detection
│   │   ├── focus/               # macOS frontmost app detection + activation
│   │   ├── hotkey/              # Hotkey string parsing
│   │   ├── keychain/            # macOS Keychain integration
│   │   ├── prompt/              # Claude/OpenAI prompt generation + secret redaction
│   │   ├── settings/            # Settings types + persistence manager
│   │   ├── storage/             # SQLite database (settings + prompt history)
│   │   └── whisper/             # Cloud (OpenAI API) + Local (whisper-rs) transcription
│   ├── tauri.conf.json          # Tauri window + app configuration
│   ├── Cargo.toml               # Rust dependencies
│   └── tests/
│       └── integration.rs       # Integration tests
├── Makefile                      # All dev/test/build/CI commands
├── package.json                  # Frontend dependencies
├── Dockerfile                    # CI testing container
├── docker-compose.yml            # Docker Compose for CI
└── .github/workflows/ci.yml     # GitHub Actions CI
```

### Testing

```bash
# Rust tests (34 unit + 5 integration)
make test

# Frontend type check + production build
make test-frontend

# Both
make test && make test-frontend
```

### Docker / CI

Docker is used for CI testing only. Development and builds require native macOS (Tauri needs macOS frameworks for window management, Keychain, etc.).

```bash
make docker-ci    # Full CI pipeline (check, test, fmt, clippy, npm build)
```

GitHub Actions runs two jobs:
1. **Linux/Docker** — cargo check, test, fmt, clippy, npm build
2. **macOS** — Full `tauri build` producing a .dmg artifact

## Architecture

### State Machine

The recording workflow follows a linear state machine:

```
idle → ready → recording → transcribing → generating → preview → pasting → idle
                    ↓                                      ↓
               (skip voice)                            (cancel)
                    ↓                                      ↓
               generating → preview → pasting → idle      idle
```

- `idle` — Waiting for hotkey
- `ready` — Text captured, choose Record Voice or Text Only
- `recording` — Audio capture active, real-time visualizer
- `transcribing` — Sending audio to Whisper
- `generating` — Sending text + transcription to Claude/GPT
- `preview` — Shows generated prompt for review/edit
- `pasting` — Writing to clipboard and simulating Cmd+V in source app

### Backend Modules

| Module | Responsibility |
|--------|---------------|
| `audio` | Mic capture via cpal, 16kHz mono WAV encoding, RMS-based silence detection |
| `clipboard` | Clipboard read/write with save/restore to preserve user's clipboard |
| `commands` | Tauri IPC command handlers — bridges frontend to backend |
| `focus` | AppleScript-based frontmost app detection and activation |
| `hotkey` | Hotkey string parsing and validation |
| `keychain` | macOS Keychain store/retrieve/delete via security-framework |
| `prompt` | AI prompt generation (Claude + OpenAI) with API secret redaction |
| `settings` | App settings types and SQLite-backed persistence |
| `storage` | SQLite database — settings key-value store + prompt history |
| `whisper` | Speech-to-text — cloud (OpenAI Whisper API) and local (whisper-rs) |

### Frontend State

- **recordingStore** (Zustand) — The core state machine. Manages the recording workflow, holds captured text, transcription, generated prompt, and source app info.
- **settingsStore** (Zustand) — Settings persistence. Loads from and saves to the Rust backend via IPC.

### IPC Commands

The frontend communicates with the Rust backend via Tauri's `invoke()`:

| Command | Purpose |
|---------|---------|
| `get_captured_context` | Get pre-captured selected text + source app |
| `start_recording` | Start audio capture |
| `stop_recording` | Stop capture, transcribe, return text |
| `skip_recording` | Stop capture without transcription |
| `get_audio_level` | Current mic RMS level (for visualizer) |
| `get_silence_duration` | Seconds since last detected voice |
| `generate_prompt` | Send text + intent to AI, get prompt back |
| `paste_to_app` | Write to clipboard, activate source app, Cmd+V |
| `copy_to_clipboard` | Write text to clipboard |
| `get/save_settings` | Settings CRUD |
| `store/get/delete_api_key` | API key management (DB + Keychain) |
| `get_prompt_history` | List recent prompts |
| `search_prompts` | Search prompts by text |
| `delete_prompt` | Delete a prompt from history |
| `hide_window` / `resize_window` | Window management |

## License

Private. All rights reserved.
