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

<!-- gitnexus:start -->
# GitNexus — Code Intelligence

This project is indexed by GitNexus as **Prompter** (675 symbols, 1193 relationships, 38 execution flows).

> Index stale? Run `node .gitnexus/run.cjs analyze --index-only` from the project root — it auto-selects an available runner. No `.gitnexus/run.cjs` yet? Bootstrap with `npx`, `bunx`, or `pnpm dlx` — e.g. `bunx gitnexus@latest analyze` (npm 11 npx crash; #1939).

## Always Do

- **MUST run impact before editing.** Use `impact({target: "symbolName", direction: "upstream"})` or `node .gitnexus/run.cjs impact "symbolName" --direction upstream --repo .`; report callers, processes, and risk. Never substitute grep for graph analysis.
- **MUST analyze graph changes before committing.** Use `detect_changes({scope: "all"})` (MCP) or `node .gitnexus/run.cjs detect-changes --scope all --repo .` (CLI fallback). `partial: true` or `truncated: true` is not a clean check — a zero means unseen, not unaffected; re-run it. For regression review: `detect_changes({scope: "compare", base_ref: "main"})` or `node .gitnexus/run.cjs detect-changes --scope compare --base-ref "main" --repo .`.
- MUST warn on HIGH/CRITICAL `risk` pre-edit; never use `riskSharedAxes` to waive a HIGH/CRITICAL `risk` warning. Compare File/symbol: MCP File omits axes; Graph-RAG expands File.
- **MUST treat `risk: UNKNOWN` as unresolved, not as low.** An empty caller set is not evidence the symbol is unused — it can also mean the callers are not resolvable by the index (plain-object property access, dynamic dispatch, cross-language calls). `impact` pairs `UNKNOWN` with a `riskNote` saying so. Confirm with a text search before treating the symbol as safe to change or delete; do not proceed on the strength of a zero.
- **MUST use `query({search_query: "concept"})` for concepts/flows, `context({name: "symbolName"})` for a named symbol, or `impact` for blast radius, on read-only callers, dependencies, imports, or execution flow.** Graph first; text search only for empty/`UNKNOWN`/literals.
- For security review, `explain({target: "fileOrSymbol"})` lists taint findings (source→sink flows; needs `analyze --pdg`).

## Never Do

- NEVER edit a function, class, or method before MCP/CLI impact analysis.
- NEVER ignore HIGH or CRITICAL risk warnings from impact analysis, and never read `UNKNOWN` as an all-clear — it means the walk could not answer, which is the one verdict that requires confirming by other means.
- NEVER rename symbols with find-and-replace — use `rename` which understands the call graph.
- NEVER commit before MCP/CLI graph change analysis.

## Resources

| Resource | Use for |
| --- | --- |
| `gitnexus://repo/Prompter/context` | Codebase overview, check index freshness |
| `gitnexus://repo/Prompter/clusters` | All functional areas |
| `gitnexus://repo/Prompter/processes` | All execution flows |
| `gitnexus://repo/Prompter/process/{name}` | Step-by-step execution trace |

## CLI

| Task | Read this skill file |
| --- | --- |
| Understand architecture / "How does X work?" | `.claude/skills/gitnexus-exploring/SKILL.md` |
| Blast radius / "What breaks if I change X?" | `.claude/skills/gitnexus-impact-analysis/SKILL.md` |
| Trace bugs / "Why is X failing?" | `.claude/skills/gitnexus-debugging/SKILL.md` |
| Rename / extract / split / refactor | `.claude/skills/gitnexus-refactoring/SKILL.md` |
| Tools, resources, schema reference | `.claude/skills/gitnexus-guide/SKILL.md` |
| Index, status, clean, wiki CLI commands | `.claude/skills/gitnexus-cli/SKILL.md` |

<!-- gitnexus:end -->
