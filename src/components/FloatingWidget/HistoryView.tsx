import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ArrowLeft, Search, Trash2, Copy, Check, ChevronDown, ChevronUp } from "lucide-react";

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
  if (diffMin < 60) return `${diffMin}m ago`;
  const diffHr = Math.floor(diffMin / 60);
  if (diffHr < 24) return `${diffHr}h ago`;
  const diffDays = Math.floor(diffHr / 24);
  if (diffDays < 7) return `${diffDays}d ago`;
  return new Date(dateStr).toLocaleDateString();
}

function getDateGroup(dateStr: string): string {
  const now = new Date();
  const then = new Date(dateStr);
  const diffDays = Math.floor((now.getTime() - then.getTime()) / 86400000);

  if (diffDays === 0) return "Today";
  if (diffDays === 1) return "Yesterday";
  if (diffDays < 7) return "This Week";
  return "Older";
}

export function HistoryView({ onBack }: Props) {
  const [records, setRecords] = useState<PromptRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchQuery, setSearchQuery] = useState("");
  const [expandedId, setExpandedId] = useState<number | null>(null);
  const [copiedId, setCopiedId] = useState<number | null>(null);

  const loadRecords = useCallback(async () => {
    try {
      let json: string;
      if (searchQuery.trim()) {
        json = await invoke<string>("search_prompts", { query: searchQuery, limit: 50 });
      } else {
        json = await invoke<string>("get_prompt_history", { limit: 50 });
      }
      setRecords(JSON.parse(json) as PromptRecord[]);
    } catch {
      setRecords([]);
    } finally {
      setLoading(false);
    }
  }, [searchQuery]);

  useEffect(() => {
    setLoading(true);
    const debounce = setTimeout(loadRecords, searchQuery ? 300 : 0);
    return () => clearTimeout(debounce);
  }, [loadRecords, searchQuery]);

  const handleCopy = async (entry: PromptRecord) => {
    try {
      await invoke("copy_to_clipboard", { text: entry.generated_prompt });
      setCopiedId(entry.id);
      setTimeout(() => setCopiedId(null), 1500);
    } catch {
      // ignore
    }
  };

  const handleDelete = async (id: number) => {
    try {
      await invoke("delete_prompt", { id });
      setRecords((prev) => prev.filter((r) => r.id !== id));
      if (expandedId === id) setExpandedId(null);
    } catch {
      // ignore
    }
  };

  const truncate = (text: string, max: number) =>
    text.length > max ? text.slice(0, max) + "\u2026" : text;

  // Group records by date
  const groups: { label: string; items: PromptRecord[] }[] = [];
  for (const record of records) {
    const label = getDateGroup(record.created_at);
    const last = groups[groups.length - 1];
    if (last && last.label === label) {
      last.items.push(record);
    } else {
      groups.push({ label, items: [record] });
    }
  }

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div
        className="flex items-center gap-2 px-3 py-2 border-b shrink-0"
        style={{ borderColor: "var(--border-color)" }}
      >
        <button
          onClick={onBack}
          className="p-1 rounded-md transition-colors hover:bg-black/5 dark:hover:bg-white/5"
          style={{ color: "var(--text-muted)" }}
          title="Back"
        >
          <ArrowLeft size={14} />
        </button>
        <div
          className="flex-1 flex items-center gap-1.5 rounded-md px-2 py-1"
          style={{
            background: "var(--bg-secondary)",
            border: "1px solid var(--border-color)",
          }}
        >
          <Search size={12} style={{ color: "var(--text-muted)" }} />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search prompts..."
            className="flex-1 bg-transparent text-xs outline-none"
            style={{ color: "var(--text-primary)" }}
          />
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto">
        {loading && (
          <div className="flex items-center justify-center p-6 text-xs" style={{ color: "var(--text-muted)" }}>
            Loading...
          </div>
        )}

        {!loading && records.length === 0 && (
          <div className="flex items-center justify-center p-6 text-xs" style={{ color: "var(--text-muted)" }}>
            {searchQuery ? "No matches" : "No prompts yet"}
          </div>
        )}

        {!loading &&
          groups.map((group) => (
            <div key={group.label}>
              <div
                className="px-3 py-1.5 text-[10px] font-semibold uppercase tracking-wider sticky top-0"
                style={{
                  color: "var(--text-muted)",
                  background: "var(--bg-primary)",
                }}
              >
                {group.label}
              </div>
              {group.items.map((entry) => {
                const isExpanded = expandedId === entry.id;
                return (
                  <div
                    key={entry.id}
                    className="border-b transition-colors"
                    style={{ borderColor: "var(--border-color)" }}
                  >
                    <button
                      onClick={() => setExpandedId(isExpanded ? null : entry.id)}
                      className="w-full text-left px-3 py-2 flex items-start gap-2 hover:bg-black/3 dark:hover:bg-white/3 transition-colors"
                    >
                      <div className="flex-1 min-w-0">
                        <p className="text-xs leading-relaxed" style={{ color: "var(--text-primary)" }}>
                          {isExpanded ? entry.generated_prompt : truncate(entry.generated_prompt, 80)}
                        </p>
                        <p className="text-[10px] mt-0.5" style={{ color: "var(--text-muted)" }}>
                          {[entry.source_app, formatRelativeTime(entry.created_at)]
                            .filter(Boolean)
                            .join(" \u00B7 ")}
                        </p>
                      </div>
                      {isExpanded ? (
                        <ChevronUp size={12} style={{ color: "var(--text-muted)" }} className="mt-0.5 shrink-0" />
                      ) : (
                        <ChevronDown size={12} style={{ color: "var(--text-muted)" }} className="mt-0.5 shrink-0" />
                      )}
                    </button>

                    {isExpanded && (
                      <div className="px-3 pb-2 flex items-center gap-1.5">
                        <button
                          onClick={() => handleCopy(entry)}
                          className="flex items-center gap-1 px-2 py-1 rounded text-[10px] font-medium transition-colors"
                          style={{
                            background: "var(--bg-secondary)",
                            color: copiedId === entry.id ? "var(--success)" : "var(--text-secondary)",
                            border: "1px solid var(--border-color)",
                          }}
                        >
                          {copiedId === entry.id ? <Check size={10} /> : <Copy size={10} />}
                          {copiedId === entry.id ? "Copied" : "Copy"}
                        </button>
                        <button
                          onClick={() => handleDelete(entry.id)}
                          className="flex items-center gap-1 px-2 py-1 rounded text-[10px] font-medium transition-colors hover:bg-red-50 dark:hover:bg-red-950/30"
                          style={{
                            color: "var(--danger)",
                            border: "1px solid var(--border-color)",
                          }}
                        >
                          <Trash2 size={10} />
                          Delete
                        </button>
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
          ))}
      </div>
    </div>
  );
}
