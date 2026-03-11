# Onboarding Redesign

## Problem

The current 3-step onboarding (Welcome, API Keys, Ready) has critical UX issues:

1. **Asks for both API keys without context** — Users don't understand that OpenAI key is for Whisper transcription and Claude key is for prompt generation.
2. **No Whisper mode choice** — Default is Cloud (OpenAI), but user may prefer Local (whisper-rs). If they choose Local, they don't need an OpenAI key at all.
3. **No settings access from widget** — After onboarding, users must open the tray menu to change settings. No visible gear icon.
4. **Provider name mismatch** — Onboarding saves as `"claude"`, Settings panel uses `"anthropic"`, prompt generator looks up `"claude"`. This causes silent key lookup failures.
5. **Idle state is blank** — When not recording, the widget shows nothing. No hint about the hotkey or app status.

## Design

### 5-Step Onboarding

**Step 1: Welcome**
- App name, brief description
- "Get started" button

**Step 2: Whisper Mode**
- "How would you like to transcribe your voice?"
- Two cards:
  - **Cloud (OpenAI Whisper API)** — Fast, requires OpenAI API key, sends audio to OpenAI
  - **Local (whisper-rs)** — Private, runs on-device, requires ~1.5GB model download
- Selection saves to settings immediately

**Step 3: API Keys (Dynamic)**
- If Cloud mode: Ask for both OpenAI API key (for transcription) AND Claude API key (for prompt generation). Each field has a label explaining its purpose.
- If Local mode: Ask for Claude API key only (for prompt generation). Explain that no OpenAI key is needed since transcription is local.
- Keys stored in macOS Keychain

**Step 4: Model Download (Conditional)**
- Only shown if Local whisper mode was selected
- "Download Whisper medium model (~1.5GB)"
- Three options: "Download now", "Download later", "Skip"
- If "Download now": show progress bar
- If "Download later" or "Skip": proceed, model downloads on first use

**Step 5: Ready**
- Summary of setup
- "Press Cmd+Shift+P to start recording"
- "Open Prompter" button

### Idle State Home View

Instead of hiding the widget when idle, show a minimal home view:
- App name/logo
- "Press Cmd+Shift+P to start" hint
- Small gear icon in top-right corner for settings access

### Settings Access

- **Gear icon** in widget corner (visible in all states)
- **Tray menu** Settings option (kept as-is)
- Both open the same SettingsPanel

### Provider Name Fix

Standardize API key account names:
- `"claude"` for Claude/Anthropic API key (used everywhere)
- `"openai"` for OpenAI API key (used everywhere)
- Fix Settings panel: change `provider="anthropic"` to `provider="claude"`

## Trade-offs

- 5 steps is more than 3, but each step is focused and contextual
- Conditional steps add complexity but prevent confusion
- Idle state view uses screen space but provides discoverability
