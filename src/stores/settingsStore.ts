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
