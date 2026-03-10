# Prompter — Design Document

## Overview

Prompter is a macOS desktop app that turns voice + selected code into polished AI prompts. When working in any coding tool (Claude Code, Cursor, VS Code, etc.), users press a global hotkey, speak their intent, and Prompter generates a ready-to-paste prompt using AI — then pastes it back into the original app.

## Core Workflow

1. User selects text in any app (code, error, etc.)
2. User presses global hotkey (default: `Cmd+Shift+P`)
3. Prompter simulates `Cmd+C` to grab the selection, stores original clipboard content
4. Records source app bundle ID via macOS accessibility APIs
5. Floating widget appears, audio recording begins
6. Speech transcribed via Whisper (local or cloud, user's choice)
7. Selected text + transcription sent to AI (Claude or GPT) for prompt generation
8. Auto-paste mode: focuses source app and pastes generated prompt
9. Preview mode: shows prompt in floating widget for editing, then pastes on confirm
10. Original clipboard content restored

## Architecture

Monolithic Tauri 2.x app — Rust backend + Vite/React/TypeScript frontend.

### Rust Backend Modules

- **Hotkey Manager** — global shortcuts via `global-hotkey` crate, user-configurable
- **Clipboard Manager** — read/write clipboard via `arboard`, save/restore original content
- **App Focus Tracker** — macOS accessibility APIs to detect active app on hotkey press
- **Audio Recorder** — mic capture via `cpal`, outputs audio buffer
- **Whisper Engine** — local: `whisper-rs` (whisper.cpp bindings), cloud: OpenAI Whisper API
- **AI Prompt Generator** — sends context to Claude or GPT API, returns generated prompt
- **Settings Manager** — reads/writes user config
- **Storage** — SQLite via `rusqlite` for prompt history and settings

### React Frontend Windows

- **Floating Widget** — small overlay: recording state, transcription preview, generated prompt preview + edit, confirm/cancel
- **Settings Panel** — full config: API keys, Whisper mode, model selection, hotkey, paste mode, recording mode
- **Menu Bar Dropdown** — recent prompts, start recording, open settings

## Settings

| Setting | Options | Default |
|---|---|---|
| Global Hotkey | User-configurable key combo | `Cmd+Shift+P` |
| Whisper Mode | Local / Cloud | Cloud |
| Local Whisper Model | tiny, base, small, medium, large | medium |
| OpenAI API Key | string | — |
| Claude API Key | string | — |
| Prompt Generation Model | Claude (Sonnet/Opus/Haiku) / GPT (4o/4o-mini) | Claude Sonnet |
| Paste Mode | Auto-paste / Preview first | Preview first |
| Recording Mode | Push-to-talk / Toggle / Auto-detect silence | Toggle |
| Silence Threshold | seconds (for auto-detect) | 4s (user-configurable) |
| Theme | Light / Dark / System | System |

## API Key Security

Keys stored encrypted via macOS Keychain using the `security-framework` crate. SQLite stores references only; actual secrets live in the system keychain.

## Prompt Generation Strategy

### System Prompt

```
You are a prompt engineer. The user has selected content from their
coding tool and described what they want to do with it via voice.

Generate a clear, actionable prompt that the user can paste directly
into an AI coding assistant (Claude Code, Cursor, etc.).

Rules:
- Be specific and unambiguous
- Include relevant context from the selected text
- Structure the prompt for best AI response
- Match the complexity of the request (simple ask = short prompt)
```

### Input

```
Selected text: {clipboard content}
User's spoken intent: {whisper transcription}
Source app: {e.g., "Cursor", "Terminal - Claude Code"}
```

### Edge Cases

- No text selected: use speech input only for a standalone prompt
- No speech: use selected text with generic "help me with this" intent
- Neither: show error in floating widget

## First-Run Experience

On first launch, Settings Panel opens automatically:
1. Add at least one API key (OpenAI or Claude)
2. Choose Whisper mode (if local, triggers model download)
3. Confirm global hotkey has no conflicts

## Tech Stack

- **App framework:** Tauri 2.x
- **Backend:** Rust
- **Frontend:** Vite + React + TypeScript
- **Storage:** SQLite (rusqlite)
- **Local STT:** whisper-rs
- **Secrets:** macOS Keychain (security-framework)
- **Key crates:** `global-hotkey`, `arboard`, `cpal`, `whisper-rs`, `security-framework`, `rusqlite`

## V1 Scope

**In:**
- Global hotkey -> grab selection -> speech -> AI prompt generation -> paste back
- Floating widget (recording, preview/edit, confirm)
- Menu bar app with dropdown (recent prompts, start recording, open settings)
- Settings panel (API keys, Whisper mode, model, hotkey, paste mode, recording mode)
- Local Whisper + Cloud Whisper
- Prompt generation via Claude or GPT
- Auto-paste and preview modes
- All three recording modes (push-to-talk, toggle, auto-detect silence)
- Prompt history in SQLite
- API keys in macOS Keychain
- First-run onboarding

**Out (V2+):**
- Built-in + custom prompt templates
- Cloud sync for history/templates
- Cross-platform (Windows, Linux)
- Advanced features (prompt chains, batch processing)
