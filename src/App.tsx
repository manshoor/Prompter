import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { SettingsPanel } from "./components/SettingsPanel";
import { FloatingWidget } from "./components/FloatingWidget";
import { HistoryView } from "./components/FloatingWidget/HistoryView";
import { Onboarding } from "./components/Onboarding";
import { useRecordingStore } from "./stores/recordingStore";
import { useSettingsStore } from "./stores/settingsStore";

function App() {
  const [view, setView] = useState<"loading" | "onboarding" | "widget" | "settings" | "history">("loading");
  const { prepareRecording } = useRecordingStore();
  const { settings, load: loadSettings } = useSettingsStore();

  // Load settings on mount
  useEffect(() => {
    loadSettings();
  }, [loadSettings]);

  // Apply theme to DOM
  useEffect(() => {
    const theme = settings?.theme ?? "System";

    function applyDark(isDark: boolean) {
      if (isDark) {
        document.documentElement.classList.add("dark");
        document.documentElement.style.colorScheme = "dark";
      } else {
        document.documentElement.classList.remove("dark");
        document.documentElement.style.colorScheme = "light";
      }
    }

    if (theme === "Dark") {
      applyDark(true);
      return;
    }
    if (theme === "Light") {
      applyDark(false);
      return;
    }

    // System: detect and listen for OS preference changes
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    applyDark(mq.matches);

    const handler = (e: MediaQueryListEvent) => applyDark(e.matches);
    mq.addEventListener("change", handler);
    return () => mq.removeEventListener("change", handler);
  }, [settings?.theme]);

  const resizeWindow = useCallback((width: number, height: number) => {
    invoke("resize_window", { width, height }).catch(() => {});
  }, []);

  const hideWindow = useCallback(() => {
    invoke("hide_window").catch(() => {});
  }, []);

  useEffect(() => {
    invoke<boolean>("is_first_run").then((isFirst) => {
      if (isFirst) {
        resizeWindow(440, 520);
        setView("onboarding");
      } else {
        setView("widget");
      }
    }).catch(() => setView("widget"));
  }, [resizeWindow]);

  // Resize window based on view
  useEffect(() => {
    const sizes: Record<string, [number, number]> = {
      widget: [420, 180],
      settings: [600, 520],
      history: [420, 420],
      onboarding: [440, 520],
    };
    const size = sizes[view];
    if (size) resizeWindow(size[0], size[1]);
  }, [view, resizeWindow]);

  // Resize window based on recording state (widget view only)
  const recordingState = useRecordingStore((s) => s.state);
  useEffect(() => {
    if (view !== "widget") return;
    const heights: Record<string, number> = {
      idle: 180,
      ready: 220,
      recording: 180,
      transcribing: 180,
      downloading: 180,
      generating: 180,
      preview: 380,
      pasting: 140,
      error: 220,
    };
    const h = heights[recordingState] ?? 180;
    resizeWindow(420, h);
  }, [recordingState, view, resizeWindow]);

  // Escape key hides the window
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        if (view === "settings" || view === "history") {
          setView("widget");
        } else {
          hideWindow();
        }
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [view, hideWindow]);

  useEffect(() => {
    const unlisten = listen("hotkey-triggered", () => {
      prepareRecording();
      setView("widget");
    });
    const unlistenSettings = listen("open-settings", () => {
      setView("settings");
    });
    const unlistenHistory = listen("open-history", () => {
      setView("history");
    });
    return () => {
      unlisten.then((fn) => fn());
      unlistenSettings.then((fn) => fn());
      unlistenHistory.then((fn) => fn());
    };
  }, [prepareRecording]);

  if (view === "loading") {
    return <div className="min-h-screen flex items-center justify-center text-gray-500 dark:text-gray-400">Loading...</div>;
  }
  if (view === "onboarding") {
    return <Onboarding onComplete={() => setView("widget")} />;
  }
  if (view === "settings") {
    return (
      <div className="h-screen flex flex-col rounded-xl bg-white/90 dark:bg-gray-900/90 backdrop-blur-2xl border border-white/20 dark:border-white/8 shadow-2xl overflow-hidden text-gray-900 dark:text-gray-100">
        <div data-tauri-drag-region className="flex items-center gap-2 px-4 py-3 border-b border-gray-200/50 dark:border-gray-800/50 shrink-0 cursor-grab">
          <button
            onClick={() => setView("widget")}
            className="px-3 py-1 text-sm bg-gray-100/80 dark:bg-gray-800/80 hover:bg-gray-200 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-200 rounded-lg transition-colors"
          >
            Back
          </button>
          <span className="text-sm text-gray-500 dark:text-gray-400">Settings</span>
        </div>
        <div className="flex-1 overflow-y-auto">
          <SettingsPanel />
        </div>
      </div>
    );
  }

  if (view === "history") {
    return (
      <div className="h-screen flex flex-col rounded-xl bg-white/90 dark:bg-gray-900/90 backdrop-blur-2xl border border-white/20 dark:border-white/8 shadow-2xl overflow-hidden text-gray-900 dark:text-gray-100">
        <div data-tauri-drag-region className="h-2 w-full cursor-grab" />
        <HistoryView onBack={() => setView("widget")} />
      </div>
    );
  }

  return <FloatingWidget onOpenSettings={() => setView("settings")} onOpenHistory={() => setView("history")} />;
}

export default App;
