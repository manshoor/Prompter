# Prompter Development Guide

## Prerequisites

- **macOS** (required — Tauri uses native macOS frameworks)
- **Docker** (optional — for CI testing in containers)

## Quick Start

```bash
# One-command setup (installs Homebrew, Rust, Node.js, cmake, dependencies)
make setup

# Start development server
make dev
```

## Manual Setup

If you prefer to install dependencies yourself:

1. Install [Homebrew](https://brew.sh)
2. `brew install cmake node`
3. Install [Rust](https://rustup.rs)
4. `npm install`
5. `cd src-tauri && cargo fetch`

## Project Structure

```
Prompter/
├── src/                    # React/TypeScript frontend
│   ├── components/         # UI components (FloatingWidget, Settings, Onboarding)
│   ├── stores/             # Zustand state management
│   └── App.tsx             # Root component
├── src-tauri/
│   └── src/
│       ├── lib.rs          # Tauri app setup, tray, hotkey
│       ├── main.rs         # Entry point
│       ├── audio/          # Mic capture (cpal)
│       ├── clipboard/      # Clipboard management (arboard)
│       ├── commands/       # Tauri IPC commands
│       ├── focus/          # App focus tracking (osascript)
│       ├── hotkey/         # Global hotkey parsing
│       ├── keychain/       # macOS Keychain integration
│       ├── prompt/         # AI prompt generation (Claude/GPT)
│       ├── settings/       # App settings management
│       ├── storage/        # SQLite database
│       └── whisper/        # Speech-to-text (local + cloud)
├── Dockerfile              # Multi-stage Docker for CI
├── docker-compose.yml      # Docker services
├── Makefile                # All dev commands
└── scripts/setup.sh        # One-command setup
```

## Development Workflow

```bash
make dev              # Start dev server (Tauri + Vite hot reload)
make test             # Run all Rust tests
make test-frontend    # TypeScript type check + Vite build
make fmt              # Format Rust code
make lint             # Run clippy linter
make check            # Full check (cargo check + frontend build)
```

## Docker Usage

Docker runs CI checks in Linux containers. It **cannot** run `tauri dev` or `tauri build` (those require macOS frameworks).

```bash
make docker-build     # Build Docker image
make docker-test      # Run Rust tests in Docker
make docker-ci        # Full CI: check + test + fmt + lint + frontend
make clean-docker     # Remove Docker images/volumes
```

## Building for Release

```bash
make build            # Builds .dmg in src-tauri/target/release/bundle/dmg/
```

## CI/CD Pipeline

GitHub Actions runs on every push/PR to `main`:

1. **Linux job (Docker):** cargo check, cargo test, cargo fmt --check, cargo clippy, npm run build
2. **macOS job:** Full `tauri build` producing a .dmg artifact

## Troubleshooting

**`cmake` not found:**
```bash
brew install cmake
```

**Whisper model download slow:** The local Whisper model (~1.5GB for medium) downloads on first use. Use cloud mode during development for faster iteration.

**Clipboard tests fail in parallel:** Clipboard tests require single-threaded execution. Use `make test` which passes `--test-threads=1`.

**Docker build fails on macOS-specific code:** Expected. Docker (Linux) can compile and test most Rust code, but macOS-only features (Keychain, osascript) will be stubbed or skipped in Docker.
