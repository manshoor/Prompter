# Onboarding Redesign Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Redesign onboarding to explain API key purposes, let user choose Whisper mode, add settings access from widget, fix provider name mismatch, and show an idle home view.

**Architecture:** Replace 3-step onboarding with 5-step flow (Welcome → Whisper Mode → Dynamic API Keys → Model Download → Ready). Add idle state home view to FloatingWidget. Add gear icon for settings access. Fix `"anthropic"` → `"claude"` provider name.

**Tech Stack:** React, TypeScript, Zustand, Tauri IPC, Tailwind CSS, lucide-react

---

### Task 1: Fix provider name mismatch in SettingsPanel

The Settings panel uses `provider="anthropic"` for the Claude API key, but the Rust backend looks up `"claude"`. This causes silent key lookup failures.

**Files:**
- Modify: `src/components/SettingsPanel/SettingsPanel.tsx:100`

**Step 1: Fix the provider name**

In `src/components/SettingsPanel/SettingsPanel.tsx`, line 100, change:
```tsx
<ApiKeyInput provider="anthropic" label="Anthropic API Key" />
```
to:
```tsx
<ApiKeyInput provider="claude" label="Claude API Key" />
```

**Step 2: Verify frontend builds**

Run: `npm run build`
Expected: Build succeeds with no errors.

**Step 3: Commit**

```bash
git add src/components/SettingsPanel/SettingsPanel.tsx
git commit -m "fix: standardize provider name to 'claude' in settings panel"
```

---

### Task 2: Add whisper mode to settings store for onboarding use

The onboarding needs to save whisper mode selection before settings are fully loaded. Add a helper to the settings store.

**Files:**
- Modify: `src/stores/settingsStore.ts`

**Step 1: Add updateWhisperMode action to store**

In `src/stores/settingsStore.ts`, add a `updateWhisperMode` action that loads current settings, patches whisper_mode, and saves:

```typescript
import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

export type WhisperMode = "Local" | "Cloud";
export type PasteMode = "Auto" | "Preview";
export type RecordingMode = "PushToTalk" | "Toggle" | "AutoDetect";
export type PromptModel =
  | "ClaudeSonnet"
  | "ClaudeOpus"
  | "ClaudeHaiku"
  | "Gpt4o"
  | "Gpt4oMini";
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
  updateWhisperMode: (mode: WhisperMode) => Promise<void>;
}

export const useSettingsStore = create<SettingsState>((set, get) => ({
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
  updateWhisperMode: async (mode: WhisperMode) => {
    let current = get().settings;
    if (!current) {
      current = await invoke<AppSettings>("get_settings");
    }
    const updated = { ...current, whisper_mode: mode };
    await invoke("save_settings", { settings: updated });
    set({ settings: updated, loading: false });
  },
}));
```

**Step 2: Verify frontend builds**

Run: `npm run build`
Expected: Build succeeds.

**Step 3: Commit**

```bash
git add src/stores/settingsStore.ts
git commit -m "feat: add updateWhisperMode action to settings store"
```

---

### Task 3: Rewrite Onboarding with 5-step flow

Replace the 3-step onboarding with 5 steps: Welcome → Whisper Mode → Dynamic API Keys → Model Download (conditional) → Ready.

**Files:**
- Modify: `src/components/Onboarding/Onboarding.tsx`

**Step 1: Rewrite Onboarding component**

Replace the entire content of `src/components/Onboarding/Onboarding.tsx` with:

```tsx
import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ApiKeyInput } from "../SettingsPanel/ApiKeyInput";
import { useSettingsStore, type WhisperMode } from "../../stores/settingsStore";

interface Props {
  onComplete: () => void;
}

export function Onboarding({ onComplete }: Props) {
  const [step, setStep] = useState(0);
  const [whisperMode, setWhisperMode] = useState<WhisperMode>("Cloud");
  const [downloadStatus, setDownloadStatus] = useState<"idle" | "downloading" | "done" | "error">("idle");
  const [downloadError, setDownloadError] = useState("");
  const { updateWhisperMode } = useSettingsStore();

  const handleWhisperSelect = async (mode: WhisperMode) => {
    setWhisperMode(mode);
    await updateWhisperMode(mode);
  };

  const handleDownloadModel = async () => {
    setDownloadStatus("downloading");
    try {
      await invoke("download_whisper_model", { model: "medium" });
      setDownloadStatus("done");
    } catch (e) {
      setDownloadStatus("error");
      setDownloadError(String(e));
    }
  };

  const handleNext = async () => {
    // Skip model download step if Cloud mode
    if (step === 2 && whisperMode === "Cloud") {
      setStep(4); // Jump to Ready
    } else if (step < 4) {
      setStep(step + 1);
    } else {
      await invoke("complete_onboarding");
      onComplete();
    }
  };

  const handleBack = () => {
    // If on Ready and Cloud mode, go back to API Keys (skip download)
    if (step === 4 && whisperMode === "Cloud") {
      setStep(2);
    } else {
      setStep(step - 1);
    }
  };

  const stepContent = [
    // Step 0: Welcome
    {
      title: "Welcome to Prompter",
      content: (
        <div className="space-y-3 text-gray-300">
          <p>Turn your voice + selected code into polished AI prompts.</p>
          <p className="text-sm text-gray-400">
            Select text in any app, press{" "}
            <kbd className="px-2 py-0.5 bg-gray-800 border border-gray-700 rounded text-xs">
              Cmd+Shift+P
            </kbd>
            , speak your intent, and get a ready-to-use prompt.
          </p>
        </div>
      ),
    },
    // Step 1: Whisper Mode
    {
      title: "How should voice be transcribed?",
      content: (
        <div className="space-y-3">
          <button
            onClick={() => handleWhisperSelect("Cloud")}
            className={`w-full text-left p-4 rounded-lg border transition-colors ${
              whisperMode === "Cloud"
                ? "border-blue-500 bg-blue-950/30"
                : "border-gray-700 bg-gray-800 hover:border-gray-600"
            }`}
          >
            <div className="flex items-center justify-between mb-1">
              <span className="font-medium text-gray-100">Cloud (OpenAI Whisper API)</span>
              {whisperMode === "Cloud" && (
                <span className="text-xs text-blue-400">Selected</span>
              )}
            </div>
            <p className="text-sm text-gray-400">
              Fast and accurate. Sends audio to OpenAI for transcription. Requires an OpenAI API key.
            </p>
          </button>
          <button
            onClick={() => handleWhisperSelect("Local")}
            className={`w-full text-left p-4 rounded-lg border transition-colors ${
              whisperMode === "Local"
                ? "border-blue-500 bg-blue-950/30"
                : "border-gray-700 bg-gray-800 hover:border-gray-600"
            }`}
          >
            <div className="flex items-center justify-between mb-1">
              <span className="font-medium text-gray-100">Local (on-device)</span>
              {whisperMode === "Local" && (
                <span className="text-xs text-blue-400">Selected</span>
              )}
            </div>
            <p className="text-sm text-gray-400">
              Private — nothing leaves your Mac. Requires a ~1.5 GB model download.
            </p>
          </button>
        </div>
      ),
    },
    // Step 2: API Keys (dynamic)
    {
      title: "Add Your API Keys",
      content: (
        <div className="space-y-4">
          {whisperMode === "Cloud" && (
            <div>
              <p className="text-xs text-gray-400 mb-2">
                Used for voice transcription (Whisper API)
              </p>
              <ApiKeyInput provider="openai" label="OpenAI API Key" />
            </div>
          )}
          <div>
            <p className="text-xs text-gray-400 mb-2">
              Used for generating polished prompts
            </p>
            <ApiKeyInput provider="claude" label="Claude API Key" />
          </div>
          {whisperMode === "Local" && (
            <p className="text-xs text-gray-500">
              No OpenAI key needed — transcription runs locally on your Mac.
            </p>
          )}
        </div>
      ),
    },
    // Step 3: Model Download (only shown for Local mode)
    {
      title: "Download Whisper Model",
      content: (
        <div className="space-y-4">
          <p className="text-sm text-gray-300">
            Local transcription needs the Whisper medium model (~1.5 GB).
          </p>
          {downloadStatus === "idle" && (
            <div className="space-y-3">
              <button
                onClick={handleDownloadModel}
                className="w-full px-4 py-3 bg-blue-600 hover:bg-blue-700 text-white rounded-lg text-sm transition-colors"
              >
                Download Now
              </button>
              <button
                onClick={handleNext}
                className="w-full px-4 py-2 text-sm text-gray-400 hover:text-gray-200 transition-colors"
              >
                Download Later (will download on first use)
              </button>
            </div>
          )}
          {downloadStatus === "downloading" && (
            <div className="flex items-center gap-3 p-4 bg-gray-800 rounded-lg">
              <div className="w-4 h-4 border-2 border-blue-500 border-t-transparent rounded-full animate-spin" />
              <span className="text-sm text-gray-300">Downloading model...</span>
            </div>
          )}
          {downloadStatus === "done" && (
            <div className="p-4 bg-green-950/30 border border-green-800 rounded-lg">
              <span className="text-sm text-green-300">Model downloaded successfully!</span>
            </div>
          )}
          {downloadStatus === "error" && (
            <div className="space-y-2">
              <div className="p-3 bg-red-950/30 border border-red-800 rounded-lg">
                <span className="text-sm text-red-300">{downloadError}</span>
              </div>
              <button
                onClick={handleDownloadModel}
                className="px-4 py-2 bg-gray-700 hover:bg-gray-600 text-gray-200 rounded-lg text-sm transition-colors"
              >
                Retry
              </button>
              <button
                onClick={handleNext}
                className="px-4 py-2 text-sm text-gray-400 hover:text-gray-200 transition-colors"
              >
                Skip for now
              </button>
            </div>
          )}
        </div>
      ),
    },
    // Step 4: Ready
    {
      title: "You're All Set!",
      content: (
        <div className="space-y-3 text-gray-300">
          <p>
            Press{" "}
            <kbd className="px-2 py-0.5 bg-gray-800 border border-gray-700 rounded text-xs">
              Cmd+Shift+P
            </kbd>{" "}
            anywhere to start recording.
          </p>
          <p className="text-sm text-gray-400">
            Select text, speak your intent, and Prompter will generate a polished prompt for you.
          </p>
          <div className="mt-4 p-3 bg-gray-800 rounded-lg text-sm text-gray-400 space-y-1">
            <p>
              <span className="text-gray-300">Transcription:</span>{" "}
              {whisperMode === "Cloud" ? "Cloud (OpenAI)" : "Local (on-device)"}
            </p>
            <p>
              <span className="text-gray-300">Prompt generation:</span> Claude Sonnet
            </p>
          </div>
          <p className="text-xs text-gray-500">
            You can change these anytime via the gear icon or tray menu.
          </p>
        </div>
      ),
    },
  ];

  const current = stepContent[step];
  const isLastStep = step === 4;
  // Hide next button on model download step (it has its own navigation)
  const showNextButton = step !== 3 || downloadStatus === "done";

  return (
    <div className="min-h-screen bg-gray-950 flex items-center justify-center">
      <div className="w-[440px] bg-gray-900 border border-gray-800 rounded-xl p-8 shadow-2xl">
        {/* Step indicator */}
        <div className="flex gap-1.5 mb-6">
          {[0, 1, 2, 3, 4]
            .filter((i) => !(i === 3 && whisperMode === "Cloud"))
            .map((i) => (
              <div
                key={i}
                className={`h-1 flex-1 rounded-full transition-colors ${
                  i <= step ? "bg-blue-500" : "bg-gray-700"
                }`}
              />
            ))}
        </div>

        <h1 className="text-xl font-semibold text-white mb-6">{current.title}</h1>
        {current.content}

        <div className="flex justify-between mt-8">
          {step > 0 ? (
            <button
              onClick={handleBack}
              className="px-4 py-2 text-sm text-gray-400 hover:text-gray-200 transition-colors"
            >
              Back
            </button>
          ) : (
            <div />
          )}
          {showNextButton && (
            <button
              onClick={handleNext}
              className="px-6 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg text-sm transition-colors"
            >
              {isLastStep ? "Get Started" : "Next"}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
```

**Step 2: Verify frontend builds**

Run: `npm run build`
Expected: Build succeeds.

**Step 3: Commit**

```bash
git add src/components/Onboarding/Onboarding.tsx
git commit -m "feat: redesign onboarding with whisper mode choice and dynamic API keys"
```

---

### Task 4: Add Rust command for model download

The onboarding calls `invoke("download_whisper_model")` but no such Tauri command exists yet. Add it.

**Files:**
- Modify: `src-tauri/src/commands/recording.rs`
- Modify: `src-tauri/src/lib.rs` (register the new command)

**Step 1: Check lib.rs to see how commands are registered**

Read `src-tauri/src/lib.rs` and find the `invoke_handler` call to see the pattern.

**Step 2: Add the download_whisper_model command**

In `src-tauri/src/commands/recording.rs`, add after the `stop_recording` function:

```rust
#[tauri::command]
pub async fn download_whisper_model(model: String) -> Result<(), String> {
    let whisper = crate::whisper::local::LocalWhisper::new(&model);
    whisper
        .download_model()
        .await
        .map_err(|e| e.to_string())
}
```

**Step 3: Register the command in lib.rs**

In `src-tauri/src/lib.rs`, add `commands::recording::download_whisper_model` to the `invoke_handler` list.

**Step 4: Verify Rust compiles**

Run: `cd src-tauri && cargo check`
Expected: Compiles with no errors.

**Step 5: Commit**

```bash
git add src-tauri/src/commands/recording.rs src-tauri/src/lib.rs
git commit -m "feat: add download_whisper_model Tauri command"
```

---

### Task 5: Add idle home view with gear icon

Instead of showing nothing when idle, show a minimal home view with the app name, hotkey hint, and a gear icon for settings.

**Files:**
- Create: `src/components/FloatingWidget/IdleView.tsx`
- Modify: `src/components/FloatingWidget/FloatingWidget.tsx`
- Modify: `src/App.tsx`

**Step 1: Create IdleView component**

Create `src/components/FloatingWidget/IdleView.tsx`:

```tsx
import { Settings } from "lucide-react";

interface Props {
  onOpenSettings: () => void;
}

export function IdleView({ onOpenSettings }: Props) {
  return (
    <div className="relative flex flex-col items-center justify-center gap-4 p-8">
      <button
        onClick={onOpenSettings}
        className="absolute top-3 right-3 p-1.5 text-gray-500 hover:text-gray-300 transition-colors rounded-lg hover:bg-gray-800"
        title="Settings"
      >
        <Settings size={16} />
      </button>
      <h1 className="text-lg font-semibold text-gray-200">Prompter</h1>
      <p className="text-sm text-gray-500">
        Press{" "}
        <kbd className="px-2 py-0.5 bg-gray-800 border border-gray-700 rounded text-xs text-gray-300">
          Cmd+Shift+P
        </kbd>{" "}
        to start
      </p>
    </div>
  );
}
```

**Step 2: Update FloatingWidget to show IdleView and accept onOpenSettings**

Replace the entire content of `src/components/FloatingWidget/FloatingWidget.tsx` with:

```tsx
import { useRecordingStore } from "../../stores/recordingStore";
import { RecordingView } from "./RecordingView";
import { PreviewView } from "./PreviewView";
import { IdleView } from "./IdleView";
import { Settings } from "lucide-react";

interface Props {
  onOpenSettings: () => void;
}

export function FloatingWidget({ onOpenSettings }: Props) {
  const { state, error, reset } = useRecordingStore();

  return (
    <div className="fixed inset-0 flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div className="w-[480px] bg-gray-900 border border-gray-700 rounded-xl shadow-2xl">
        {state === "idle" && <IdleView onOpenSettings={onOpenSettings} />}
        {state === "error" && (
          <div className="flex flex-col gap-3 p-4">
            <div className="flex items-center gap-2">
              <div className="w-2 h-2 bg-red-500 rounded-full" />
              <span className="text-sm font-medium text-red-400">Error</span>
            </div>
            <p className="text-sm text-gray-300 bg-red-950/50 border border-red-900 rounded-lg p-3 break-words">
              {error}
            </p>
            <button
              onClick={reset}
              className="px-4 py-2 bg-gray-700 hover:bg-gray-600 text-gray-200 rounded-lg text-sm transition-colors"
            >
              Dismiss
            </button>
          </div>
        )}
        {(state === "recording" ||
          state === "transcribing" ||
          state === "generating") && <RecordingView />}
        {state === "preview" && <PreviewView />}
        {state === "pasting" && (
          <div className="flex items-center justify-center p-6 text-sm text-gray-400">
            Pasting...
          </div>
        )}
        {/* Gear icon visible in non-idle states */}
        {state !== "idle" && (
          <button
            onClick={onOpenSettings}
            className="absolute top-3 right-3 p-1.5 text-gray-600 hover:text-gray-300 transition-colors rounded-lg hover:bg-gray-800/50"
            title="Settings"
          >
            <Settings size={14} />
          </button>
        )}
      </div>
    </div>
  );
}
```

**Step 3: Update App.tsx to pass onOpenSettings and remove null-on-idle guard**

Replace the entire content of `src/App.tsx` with:

```tsx
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

  if (view === "loading") {
    return <div className="min-h-screen bg-gray-950 flex items-center justify-center text-gray-400">Loading...</div>;
  }
  if (view === "onboarding") {
    return <Onboarding onComplete={() => setView("widget")} />;
  }
  if (view === "settings") {
    return (
      <div className="min-h-screen bg-gray-950 text-gray-100">
        <div className="flex items-center gap-2 p-4 border-b border-gray-800">
          <button
            onClick={() => setView("widget")}
            className="px-3 py-1 text-sm bg-gray-800 hover:bg-gray-700 rounded-lg transition-colors"
          >
            Back
          </button>
          <span className="text-sm text-gray-400">Settings</span>
        </div>
        <SettingsPanel />
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-950">
      <FloatingWidget onOpenSettings={() => setView("settings")} />
    </div>
  );
}

export default App;
```

**Step 4: Verify frontend builds**

Run: `npm run build`
Expected: Build succeeds.

**Step 5: Commit**

```bash
git add src/components/FloatingWidget/IdleView.tsx src/components/FloatingWidget/FloatingWidget.tsx src/App.tsx
git commit -m "feat: add idle home view with gear icon and settings access"
```

---

### Task 6: Verify everything works end-to-end

**Step 1: Run Rust tests**

Run: `cd src-tauri && cargo test`
Expected: All tests pass.

**Step 2: Run frontend build**

Run: `npm run build`
Expected: Build succeeds.

**Step 3: Format and lint**

Run:
```bash
cd src-tauri && cargo fmt
cd src-tauri && cargo clippy -- -D warnings
```
Expected: No warnings, no errors.

**Step 4: Commit any formatting changes**

If `cargo fmt` made changes:
```bash
git add -A && git commit -m "style: format code"
```
