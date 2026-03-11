## Commit Preferences

- Do NOT add "Co-Authored-By: Claude" or any co-author trailer to commit messages.
- Do NOT add any AI attribution to commits. The user will credit AI contributions as they see fit.

## Project: Prompter

A macOS-first desktop app (Tauri 2.x + Rust + Vite/React/TypeScript) that captures voice + selected code to generate polished AI prompts.

### Key Architecture Decisions

- **Tech stack:** Tauri 2.x, Rust backend, Vite + React + TypeScript frontend
- **Storage:** SQLite via rusqlite (local-first, cloud sync later in v2)
- **Speech-to-text:** Whisper — both local (whisper-rs, default model: medium) and cloud (OpenAI API), user's choice
- **Prompt generation:** Claude or OpenAI GPT, user selects model
- **API key security:** macOS Keychain via security-framework crate
- **Audio:** cpal crate for mic capture, 16kHz mono WAV for Whisper

### Design & Plan Docs

- Design: `docs/plans/2026-03-10-prompter-design.md`
- Implementation plan: `docs/plans/2026-03-10-prompter-implementation.md`

### Settings Defaults

- Whisper mode: Cloud
- Local Whisper model: medium
- Paste mode: Preview first
- Recording mode: Toggle
- Silence threshold: 4s
- Global hotkey: Cmd+Shift+P
- Theme: System

### Development Commands

```bash
make setup            # One-command dev environment setup
make dev              # Start Tauri dev server
make build            # Build release .dmg
make test             # Run Rust tests
make test-frontend    # TypeScript check + Vite build
make fmt              # Format Rust code
make lint             # Run clippy
make check            # cargo check + frontend build
```

### Docker (CI Only)

Docker handles CI testing in Linux containers. `tauri dev` and `tauri build` require native macOS.

```bash
make docker-build     # Build Docker image
make docker-test      # Run tests in Docker
make docker-ci        # Full CI pipeline in Docker
```

### CI Pipeline

GitHub Actions on push/PR to main:
1. Linux/Docker: cargo check, test, fmt, clippy, npm build
2. macOS: Full `tauri build` → .dmg artifact
