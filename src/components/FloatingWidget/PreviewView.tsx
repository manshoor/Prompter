import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useRecordingStore } from "../../stores/recordingStore";

export function PreviewView() {
  const { generatedPrompt, pastePrompt, reset } = useRecordingStore();
  const [editedPrompt, setEditedPrompt] = useState(generatedPrompt);
  const [copyStatus, setCopyStatus] = useState<string | null>(null);

  const handleCopy = async () => {
    try {
      await invoke("copy_to_clipboard", { text: editedPrompt });
      setCopyStatus("Copied!");
    } catch {
      setCopyStatus("Failed");
    }
    setTimeout(() => setCopyStatus(null), 2000);
  };

  return (
    <div className="flex flex-col gap-3 p-4">
      <h3 className="text-sm font-medium text-gray-300">Generated Prompt</h3>
      <textarea
        value={editedPrompt}
        onChange={(e) => setEditedPrompt(e.target.value)}
        rows={8}
        className="w-full p-3 bg-gray-800 border border-gray-700 rounded-lg text-sm text-gray-200 resize-y focus:outline-none focus:ring-2 focus:ring-blue-500"
      />
      <div className="flex gap-2">
        <button
          onClick={() => pastePrompt(editedPrompt)}
          className="flex-1 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg text-sm transition-colors"
        >
          Paste to App
        </button>
        <button
          onClick={handleCopy}
          className="px-4 py-2 bg-gray-700 hover:bg-gray-600 text-gray-200 rounded-lg text-sm transition-colors"
        >
          {copyStatus ?? "Copy"}
        </button>
        <button
          onClick={reset}
          className="px-4 py-2 bg-gray-700 hover:bg-gray-600 text-gray-200 rounded-lg text-sm transition-colors"
        >
          Cancel
        </button>
      </div>
    </div>
  );
}
