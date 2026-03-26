import { useRecordingStore } from "../../stores/recordingStore";
import { RecordingView } from "./RecordingView";
import { PreviewView } from "./PreviewView";
import { IdleView } from "./IdleView";
import { Settings } from "lucide-react";

interface Props {
  onOpenSettings: () => void;
  onOpenHistory: () => void;
}

export function FloatingWidget({ onOpenSettings, onOpenHistory }: Props) {
  const { state, error, reset } = useRecordingStore();

  return (
    <div className="fixed inset-0 flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div className="relative w-[480px] bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-700 rounded-xl shadow-2xl">
        {state === "idle" && <IdleView onOpenSettings={onOpenSettings} onOpenHistory={onOpenHistory} />}
        {state === "error" && (
          <div className="flex flex-col gap-3 p-4">
            <div className="flex items-center gap-2">
              <div className="w-2 h-2 bg-red-500 rounded-full" />
              <span className="text-sm font-medium text-red-600 dark:text-red-400">Error</span>
            </div>
            <p className="text-sm text-gray-700 dark:text-gray-300 bg-red-50 dark:bg-red-950/50 border border-red-200 dark:border-red-900 rounded-lg p-3 break-words">
              {error}
            </p>
            <button
              onClick={reset}
              className="px-4 py-2 bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 text-gray-700 dark:text-gray-200 rounded-lg text-sm transition-colors"
            >
              Dismiss
            </button>
          </div>
        )}
        {(state === "recording" ||
          state === "transcribing" ||
          state === "downloading" ||
          state === "generating") && <RecordingView />}
        {state === "preview" && <PreviewView />}
        {state === "pasting" && (
          <div className="flex items-center justify-center p-6 text-sm text-gray-500 dark:text-gray-400">
            Pasting...
          </div>
        )}
        {/* Gear icon visible in non-idle states */}
        {state !== "idle" && (
          <button
            onClick={onOpenSettings}
            className="absolute top-3 right-3 p-1.5 text-gray-400 dark:text-gray-600 hover:text-gray-700 dark:hover:text-gray-300 transition-colors rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800/50"
            title="Settings"
          >
            <Settings size={14} />
          </button>
        )}
      </div>
    </div>
  );
}
