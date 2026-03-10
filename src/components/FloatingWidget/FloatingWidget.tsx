import { useRecordingStore } from "../../stores/recordingStore";
import { RecordingView } from "./RecordingView";
import { PreviewView } from "./PreviewView";

export function FloatingWidget() {
  const { state, error } = useRecordingStore();

  if (state === "idle") return null;

  return (
    <div className="fixed inset-0 flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div className="w-[480px] bg-gray-900 border border-gray-700 rounded-xl shadow-2xl">
        {error && (
          <div className="px-4 py-2 bg-red-900/50 border-b border-red-800 text-red-300 text-xs">
            {error}
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
