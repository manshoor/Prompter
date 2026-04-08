import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Copy, Check } from "lucide-react";
import { useRecordingStore } from "../../stores/recordingStore";

export function PreviewView() {
  const { generatedPrompt, pastePrompt, reset } = useRecordingStore();
  const [editedPrompt, setEditedPrompt] = useState(generatedPrompt);
  const [isEditing, setIsEditing] = useState(false);
  const [copyStatus, setCopyStatus] = useState(false);

  const handleCopy = async () => {
    try {
      await invoke("copy_to_clipboard", { text: editedPrompt });
      setCopyStatus(true);
      setTimeout(() => setCopyStatus(false), 1500);
    } catch {
      // ignore
    }
  };

  const wordCount = editedPrompt.trim().split(/\s+/).filter(Boolean).length;

  return (
    <div className="flex flex-col gap-2.5 px-4 pb-4 pt-1">
      <div className="flex items-center justify-between">
        <span className="text-xs font-medium" style={{ color: "var(--text-muted)" }}>
          Generated Prompt
        </span>
        <span className="text-[10px] tabular-nums" style={{ color: "var(--text-muted)" }}>
          {wordCount} words
        </span>
      </div>

      {isEditing ? (
        <textarea
          value={editedPrompt}
          onChange={(e) => setEditedPrompt(e.target.value)}
          rows={8}
          autoFocus
          onBlur={() => setIsEditing(false)}
          className="w-full p-2.5 text-xs leading-relaxed rounded-lg resize-none focus:outline-none focus:ring-2"
          style={{
            background: "var(--input-bg)",
            color: "var(--text-primary)",
            border: "1px solid var(--input-border)",
          }}
        />
      ) : (
        <div
          onClick={() => setIsEditing(true)}
          className="w-full p-2.5 text-xs leading-relaxed rounded-lg cursor-text max-h-56 overflow-y-auto whitespace-pre-wrap"
          style={{
            background: "var(--bg-secondary)",
            color: "var(--text-primary)",
            border: "1px solid var(--border-color)",
          }}
        >
          {editedPrompt}
        </div>
      )}

      <div className="flex items-center gap-2">
        <button
          onClick={() => pastePrompt(editedPrompt)}
          className="flex-1 px-3 py-2 text-white rounded-lg text-sm font-medium transition-all active:scale-[0.98]"
          style={{ background: "var(--accent)" }}
        >
          Paste to App
        </button>
        <button
          onClick={handleCopy}
          className="p-2 rounded-lg transition-all active:scale-95"
          style={{
            background: "var(--bg-secondary)",
            color: copyStatus ? "var(--success)" : "var(--text-secondary)",
            border: "1px solid var(--border-color)",
          }}
          title="Copy"
        >
          {copyStatus ? <Check size={14} /> : <Copy size={14} />}
        </button>
        <button
          onClick={reset}
          className="px-3 py-2 text-sm rounded-lg transition-colors"
          style={{ color: "var(--text-muted)" }}
        >
          Cancel
        </button>
      </div>
    </div>
  );
}
