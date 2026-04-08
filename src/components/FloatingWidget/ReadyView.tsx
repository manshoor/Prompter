import { Mic, ArrowRight } from "lucide-react";
import { useRecordingStore } from "../../stores/recordingStore";

export function ReadyView() {
  const { selectedText, sourceApp, startRecording, skipRecording } =
    useRecordingStore();

  const hasText = selectedText.length > 0;
  const truncated =
    selectedText.length > 160
      ? selectedText.slice(0, 160) + "\u2026"
      : selectedText;

  return (
    <div className="flex flex-col gap-3 px-4 pb-4 pt-1">
      {/* Context info */}
      <div className="space-y-1.5">
        {sourceApp && (
          <span
            className="inline-block text-[10px] font-medium px-2 py-0.5 rounded-full"
            style={{
              background: "var(--bg-tertiary)",
              color: "var(--text-muted)",
            }}
          >
            from {sourceApp.name}
          </span>
        )}
        {hasText ? (
          <p
            className="text-xs rounded-lg p-2 max-h-16 overflow-y-auto leading-relaxed font-mono"
            style={{
              background: "var(--bg-secondary)",
              color: "var(--text-secondary)",
              border: "1px solid var(--border-color)",
            }}
          >
            {truncated}
          </p>
        ) : (
          <p className="text-xs italic" style={{ color: "var(--text-muted)" }}>
            No text selected — voice recording recommended
          </p>
        )}
      </div>

      {/* Action buttons */}
      <div className="flex gap-2">
        <button
          onClick={startRecording}
          className="flex-1 flex items-center justify-center gap-1.5 px-3 py-2 text-white rounded-lg text-sm font-medium transition-all active:scale-[0.98]"
          style={{ background: "var(--danger)" }}
        >
          <Mic size={14} />
          Record Voice
        </button>
        {hasText && (
          <button
            onClick={skipRecording}
            className="flex-1 flex items-center justify-center gap-1.5 px-3 py-2 rounded-lg text-sm font-medium transition-all active:scale-[0.98]"
            style={{
              background: "var(--bg-secondary)",
              color: "var(--text-primary)",
              border: "1px solid var(--border-color)",
            }}
          >
            <ArrowRight size={14} />
            Text Only
          </button>
        )}
      </div>
    </div>
  );
}
