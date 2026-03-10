import { useEffect, useState } from "react";
import { SettingsPanel } from "./components/SettingsPanel";
import { FloatingWidget } from "./components/FloatingWidget";
import { listen } from "@tauri-apps/api/event";
import { useRecordingStore } from "./stores/recordingStore";

function App() {
  const [view, setView] = useState<"widget" | "settings">("widget");
  const { startRecording } = useRecordingStore();

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
      <FloatingWidget />
    </div>
  );
}

export default App;
