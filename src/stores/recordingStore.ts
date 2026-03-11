import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

type RecordingState =
  | "idle"
  | "recording"
  | "transcribing"
  | "downloading"
  | "generating"
  | "preview"
  | "pasting"
  | "error";

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
  downloadProgress: number;

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
  downloadProgress: 0,

  startRecording: async () => {
    try {
      set({ state: "recording", error: null });
      const result = await invoke<string>("start_recording");
      const data = JSON.parse(result);
      set({ selectedText: data.selected_text, sourceApp: data.source_app });
    } catch (e) {
      set({ state: "error", error: String(e) });
    }
  },

  stopRecording: async () => {
    try {
      set({ state: "transcribing" });

      // Listen for model download events (auto-download on first local use)
      let unlistenStarted: UnlistenFn | null = null;
      let unlistenProgress: UnlistenFn | null = null;

      unlistenStarted = await listen("model-download-started", () => {
        set({ state: "downloading", downloadProgress: 0 });
      });
      unlistenProgress = await listen<{ percent: number }>(
        "model-download-progress",
        (event) => {
          set({ downloadProgress: event.payload.percent });
        },
      );

      const transcription = await invoke<string>("stop_recording");

      unlistenStarted?.();
      unlistenProgress?.();

      set({ transcription, state: "generating" });
      await get().generatePrompt();
    } catch (e) {
      set({ state: "error", error: String(e) });
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
      set({ state: "error", error: String(e) });
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
      set({ state: "error", error: String(e) });
    }
  },

  reset: () =>
    set({
      state: "idle",
      selectedText: "",
      transcription: "",
      generatedPrompt: "",
      sourceApp: null,
      error: null,
      downloadProgress: 0,
    }),
}));
