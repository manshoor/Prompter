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
