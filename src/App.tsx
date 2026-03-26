import { useEffect, useState } from "react";
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
  const { startRecording } = useRecordingStore();
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

  useEffect(() => {
    invoke<boolean>("is_first_run").then((isFirst) => {
      setView(isFirst ? "onboarding" : "widget");
    }).catch(() => setView("widget"));
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
    return <div className="min-h-screen bg-white dark:bg-gray-950 flex items-center justify-center text-gray-500 dark:text-gray-400">Loading...</div>;
  }
  if (view === "onboarding") {
    return <Onboarding onComplete={() => setView("widget")} />;
  }
  if (view === "settings") {
    return (
      <div className="h-screen flex flex-col bg-white dark:bg-gray-950 text-gray-900 dark:text-gray-100">
        <div className="flex items-center gap-2 p-4 border-b border-gray-200 dark:border-gray-800 shrink-0">
          <button
            onClick={() => setView("widget")}
            className="px-3 py-1 text-sm bg-gray-100 dark:bg-gray-800 hover:bg-gray-200 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-200 rounded-lg transition-colors"
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
      <div className="h-screen flex flex-col bg-white dark:bg-gray-950 text-gray-900 dark:text-gray-100">
        <HistoryView onBack={() => setView("widget")} />
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-white dark:bg-gray-950">
      <FloatingWidget onOpenSettings={() => setView("settings")} onOpenHistory={() => setView("history")} />
    </div>
  );
}

export default App;
