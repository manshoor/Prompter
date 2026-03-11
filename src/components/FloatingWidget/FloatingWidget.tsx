import { useRecordingStore } from "../../stores/recordingStore";
import { RecordingView } from "./RecordingView";
import { PreviewView } from "./PreviewView";

export function FloatingWidget() {
  const { state, error, reset } = useRecordingStore();

  if (state === "idle") return null;

  return (
    <div className="fixed inset-0 flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div className="w-[480px] bg-gray-900 border border-gray-700 rounded-xl shadow-2xl">
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
      </div>
    </div>
  );
}
