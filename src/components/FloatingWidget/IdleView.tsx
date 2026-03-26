import { useEffect } from "react";
import { Settings, Clock } from "lucide-react";
import { useSettingsStore } from "../../stores/settingsStore";

interface Props {
  onOpenSettings: () => void;
  onOpenHistory: () => void;
}

export function IdleView({ onOpenSettings, onOpenHistory }: Props) {
  const { settings, load } = useSettingsStore();

  useEffect(() => {
    if (!settings) {
      load();
    }
  }, [settings, load]);

  const hotkey = settings?.global_hotkey
    ? settings.global_hotkey.replace("CmdOrCtrl", "Cmd")
    : "Cmd+Shift+P";

  return (
    <div className="relative flex flex-col items-center justify-center gap-4 p-8">
      <div className="absolute top-3 right-3 flex items-center gap-1">
        <button
          onClick={onOpenHistory}
          className="p-1.5 text-gray-500 hover:text-gray-300 transition-colors rounded-lg hover:bg-gray-800"
          title="History"
        >
          <Clock size={16} />
        </button>
        <button
          onClick={onOpenSettings}
          className="p-1.5 text-gray-500 hover:text-gray-300 transition-colors rounded-lg hover:bg-gray-800"
          title="Settings"
        >
          <Settings size={16} />
        </button>
      </div>
      <h1 className="text-lg font-semibold text-gray-200">Prompter</h1>
      <p className="text-sm text-gray-500">
        Press{" "}
        <kbd className="px-2 py-0.5 bg-gray-800 border border-gray-700 rounded text-xs text-gray-300">
          {hotkey}
        </kbd>{" "}
        to start
      </p>
    </div>
  );
}
