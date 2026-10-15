# Prompter

Turn selected code and a spoken request into a ready-to-paste AI prompt.

Select text in any app, press a hotkey, say what you want ("write tests for this", "make this async"), and Prompter writes a clear prompt for Claude Code, Cursor, or any AI assistant. It then pastes the prompt back into the app you came from.

A macOS menu bar app built with Tauri 2, Rust, React, and TypeScript.

> **Status: pre-release.** Prompter is under active development and has no signed release yet. Build it from source. See [Known issues](#known-issues).

## How it works

1. Select text in any app: an editor, terminal, or browser.
2. Press **Cmd+Shift+P**. Prompter copies the selection and opens a small floating window.
3. Pick **Record Voice** to describe what you want, or **Text Only** to skip voice.
4. Prompter sends the text and your request to Claude or GPT and shows the generated prompt.
5. Edit it if needed, then **Paste to App** or **Copy**.

## Features

- **Voice is optional.** Use text only when you don't need to speak.
- **Cloud or local speech-to-text.** Use the OpenAI Whisper API, or run Whisper on your Mac so audio never leaves it.
- **Choice of model.** Claude or OpenAI GPT for prompt generation.
- **Preview or auto-paste.** Review and edit the prompt first, or paste it straight away.
- **Prompt history.** Search, copy, and delete past prompts.
- **Secret redaction.** Common API key and token formats are removed from your text before it is sent to an AI provider.
- **Light, dark, or system theme.**
- **First-run setup wizard** for API keys and speech settings.

## Requirements

- macOS (Apple Silicon or Intel)
- [Rust](https://rustup.rs/) 1.75 or later
- [Node.js](https://nodejs.org/) 18 or later
- cmake: `brew install cmake` (needed to build local Whisper)
- An API key:
  - **Claude or OpenAI** for prompt generation
  - **OpenAI** as well, if you use cloud speech-to-text

## Getting started

```bash
git clone https://github.com/manshoor/Prompter.git
cd Prompter
make setup
```

`make setup` checks for and installs Homebrew, Rust, Node.js, cmake, and the project dependencies.

Run in development mode:

```bash
make dev
```

Build a `.dmg`:

```bash
make build
```

The `.dmg` is written to `src-tauri/target/release/bundle/dmg/`.

## macOS permissions

Prompter needs these permissions to work:

| Permission | Why |
|---|---|
| **Microphone** | Record your spoken request |
| **Accessibility** | Copy your selection and paste the prompt by sending Cmd+C and Cmd+V |
| **Automation (System Events)** | Find and switch back to the app you came from |

macOS asks for some of these on first use. Grant the rest in **System Settings → Privacy & Security**.

## Settings

Open settings from the gear icon or the menu bar icon.

| Setting | Options | Default |
|---|---|---|
| Speech-to-text | Cloud (OpenAI) / Local | Cloud |
| Local model | tiny / base / small / medium | medium |
| Prompt model | Claude Sonnet / Opus / Haiku, GPT-4o / GPT-4o Mini | Claude Sonnet |
| Paste mode | Preview / Auto | Preview |
| Silence auto-stop | 1–10 seconds | 4 s |
| Global hotkey | A key combination with at least one modifier | Cmd+Shift+P |
| Theme | System / Light / Dark | System |

## Privacy

- **What leaves your Mac:** your selected text and spoken request go to the AI provider you choose (Anthropic or OpenAI). With cloud speech-to-text, your audio goes to OpenAI.
- **What stays local:** prompt history and settings are stored in a SQLite database at `~/Library/Application Support/com.prompter.app/`.
- **Nothing else.** Prompter has no telemetry and no server of its own.

## Known issues

These are being fixed before the first release:

- Claude prompt generation currently fails because the app requests a retired Claude model. Use GPT-4o until this is fixed.
- The model choice in Settings is not yet applied. Claude options all use one model, and GPT options all use GPT-4o.
- API keys are stored in the local database. The move to macOS Keychain is in progress.
- If nothing is selected, the existing clipboard content may be used as the selection.
- Pressing Escape while recording hides the window but does not stop the microphone.
- The **Push** and **Toggle** recording modes behave like the default mode.
- History times may be off by your UTC offset.
- The "Large" local model fails to download. Use **Medium** or smaller.

## Development

| Command | What it does |
|---|---|
| `make dev` | Start the app with hot reload |
| `make test` | Run Rust tests |
| `make test-frontend` | TypeScript check and frontend build |
| `make fmt` / `make lint` | Format and lint Rust code |
| `make check` | `cargo check` and frontend build |
| `make docker-ci` | Run the Linux CI checks in Docker |

Development and builds need native macOS. Docker is used only for CI checks.

See [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) for architecture and contributor details.

### Project layout

```
src/                  React frontend
  components/         Floating widget, settings, onboarding, history
  stores/             App state (Zustand)
src-tauri/src/        Rust backend
  audio/              Microphone capture and WAV encoding
  clipboard/          Clipboard read, write, save, and restore
  commands/           Commands the frontend calls
  focus/              Find and activate the source app
  prompt/             Claude and OpenAI calls, secret redaction
  storage/            SQLite settings and history
  whisper/            Cloud and local speech-to-text
```

## Tech stack

Tauri 2 · Rust · React 19 · TypeScript · Tailwind CSS 4 · Zustand · SQLite (rusqlite) · whisper-rs · cpal

## License

[MIT](LICENSE)
