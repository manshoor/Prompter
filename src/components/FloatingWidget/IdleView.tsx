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
    ? settings.global_hotkey.replace("CmdOrCtrl", "\u2318")
    : "\u2318\u21E7P";

  return (
    <div className="flex items-center justify-between px-4 py-3">
      <div className="flex items-center gap-2.5">
        <span className="text-sm font-semibold" style={{ color: "var(--text-primary)" }}>
          Prompter
        </span>
        <kbd
          className="px-1.5 py-0.5 text-[10px] font-medium rounded"
          style={{
            background: "var(--bg-tertiary)",
            color: "var(--text-muted)",
            border: "1px solid var(--border-color)",
          }}
        >
          {hotkey}
        </kbd>
      </div>
      <div className="flex items-center gap-0.5">
        <button
          onClick={onOpenHistory}
          className="p-1.5 rounded-md transition-colors hover:bg-black/5 dark:hover:bg-white/5"
          style={{ color: "var(--text-muted)" }}
          title="History"
        >
          <Clock size={14} />
        </button>
        <button
          onClick={onOpenSettings}
          className="p-1.5 rounded-md transition-colors hover:bg-black/5 dark:hover:bg-white/5"
          style={{ color: "var(--text-muted)" }}
          title="Settings"
        >
          <Settings size={14} />
        </button>
      </div>
    </div>
  );
}
