import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ArrowLeft } from "lucide-react";

interface PromptRecord {
  id: number;
  selected_text: string | null;
  spoken_intent: string | null;
  generated_prompt: string;
  source_app: string | null;
  created_at: string;
}

interface Props {
  onBack: () => void;
}

function formatRelativeTime(dateStr: string): string {
  const now = Date.now();
  const then = new Date(dateStr).getTime();
  const diffMs = now - then;
  const diffSec = Math.floor(diffMs / 1000);

  if (diffSec < 60) return "just now";

  const diffMin = Math.floor(diffSec / 60);
  if (diffMin < 60) return `${diffMin} min ago`;

  const diffHr = Math.floor(diffMin / 60);
  if (diffHr < 24) return `${diffHr}h ago`;

  const diffDays = Math.floor(diffHr / 24);
  if (diffDays < 30) return `${diffDays}d ago`;

  const diffMonths = Math.floor(diffDays / 30);
  return `${diffMonths}mo ago`;
}

export function HistoryView({ onBack }: Props) {
  const [records, setRecords] = useState<PromptRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [copiedId, setCopiedId] = useState<number | null>(null);

  useEffect(() => {
    invoke<string>("get_prompt_history", { limit: 20 })
      .then((json) => {
        const parsed = JSON.parse(json) as PromptRecord[];
        setRecords(parsed);
      })
      .catch(() => {
        setRecords([]);
      })
      .finally(() => setLoading(false));
  }, []);

  const handleCopy = async (entry: PromptRecord) => {
    try {
      await invoke("copy_to_clipboard", { text: entry.generated_prompt });
      setCopiedId(entry.id);
      setTimeout(() => setCopiedId(null), 1500);
    } catch {
      // silently fail
    }
  };

  const truncate = (text: string, max: number) =>
    text.length > max ? text.slice(0, max) + "..." : text;

  return (
    <div className="flex flex-col h-full max-h-[480px]">
      {/* Header */}
      <div className="flex items-center gap-2 px-4 py-3 border-b border-gray-200 dark:border-gray-800 shrink-0">
        <button
          onClick={onBack}
          className="p-1 text-gray-500 dark:text-gray-400 hover:text-gray-800 dark:hover:text-gray-200 transition-colors rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800"
          title="Back"
        >
          <ArrowLeft size={16} />
        </button>
        <span className="text-sm font-medium text-gray-800 dark:text-gray-200">History</span>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto">
        {loading && (
          <div className="flex items-center justify-center p-8 text-sm text-gray-500">
            Loading...
          </div>
        )}

        {!loading && records.length === 0 && (
          <div className="flex items-center justify-center p-8 text-sm text-gray-500">
            No prompts yet
          </div>
        )}

        {!loading &&
          records.map((entry) => (
            <button
              key={entry.id}
              onClick={() => handleCopy(entry)}
              className="w-full text-left px-4 py-3 border-b border-gray-200/50 dark:border-gray-800/50 hover:bg-gray-100/50 dark:hover:bg-gray-800/50 transition-colors cursor-pointer"
            >
              <p className="text-sm text-gray-800 dark:text-gray-200 leading-relaxed">
                {copiedId === entry.id
                  ? "Copied!"
                  : truncate(entry.generated_prompt, 80)}
              </p>
              <p className="text-xs text-gray-500 mt-1">
                {[entry.source_app, formatRelativeTime(entry.created_at)]
                  .filter(Boolean)
                  .join(" · ")}
              </p>
            </button>
          ))}
      </div>
    </div>
  );
}
