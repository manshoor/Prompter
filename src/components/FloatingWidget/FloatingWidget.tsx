import { useRecordingStore } from "../../stores/recordingStore";
import { RecordingView } from "./RecordingView";
import { PreviewView } from "./PreviewView";
import { ReadyView } from "./ReadyView";
import { IdleView } from "./IdleView";
import { Settings } from "lucide-react";
import { AnimatePresence, motion } from "framer-motion";

interface Props {
  onOpenSettings: () => void;
  onOpenHistory: () => void;
}

const fadeVariants = {
  initial: { opacity: 0, y: 4 },
  animate: { opacity: 1, y: 0 },
  exit: { opacity: 0, y: -4 },
};

export function FloatingWidget({ onOpenSettings, onOpenHistory }: Props) {
  const { state, error, reset } = useRecordingStore();

  return (
    <div className="w-full h-full rounded-xl glass shadow-2xl overflow-hidden">
      {/* Drag region */}
      <div data-tauri-drag-region className="h-2 w-full cursor-grab" />

      <div className="relative">
        <AnimatePresence mode="wait">
          {state === "idle" && (
            <motion.div key="idle" {...fadeVariants} transition={{ duration: 0.15 }}>
              <IdleView onOpenSettings={onOpenSettings} onOpenHistory={onOpenHistory} />
            </motion.div>
          )}

          {state === "ready" && (
            <motion.div key="ready" {...fadeVariants} transition={{ duration: 0.15 }}>
              <ReadyView />
            </motion.div>
          )}

          {state === "error" && (
            <motion.div key="error" {...fadeVariants} transition={{ duration: 0.15 }}>
              <div className="flex flex-col gap-3 p-4">
                <div className="flex items-center gap-2">
                  <div className="w-2 h-2 bg-red-500 rounded-full" />
                  <span className="text-sm font-medium" style={{ color: "var(--danger)" }}>Error</span>
                </div>
                <p className="text-xs bg-red-50 dark:bg-red-950/30 border border-red-200 dark:border-red-900/50 rounded-lg p-2.5 break-words" style={{ color: "var(--text-secondary)" }}>
                  {error}
                </p>
                <button
                  onClick={reset}
                  className="px-3 py-1.5 text-sm rounded-lg transition-colors"
                  style={{ background: "var(--bg-secondary)", color: "var(--text-primary)", border: "1px solid var(--border-color)" }}
                >
                  Dismiss
                </button>
              </div>
            </motion.div>
          )}

          {(state === "recording" ||
            state === "transcribing" ||
            state === "downloading" ||
            state === "generating") && (
            <motion.div key="recording" {...fadeVariants} transition={{ duration: 0.15 }}>
              <RecordingView />
            </motion.div>
          )}

          {state === "preview" && (
            <motion.div key="preview" {...fadeVariants} transition={{ duration: 0.15 }}>
              <PreviewView />
            </motion.div>
          )}

          {state === "pasting" && (
            <motion.div key="pasting" {...fadeVariants} transition={{ duration: 0.15 }}>
              <div className="flex items-center justify-center p-6 text-sm" style={{ color: "var(--text-muted)" }}>
                <div className="w-4 h-4 border-2 rounded-full animate-spin mr-2" style={{ borderColor: "var(--accent)", borderTopColor: "transparent" }} />
                Pasting...
              </div>
            </motion.div>
          )}
        </AnimatePresence>

        {/* Gear icon visible in active states */}
        {state !== "idle" && state !== "ready" && state !== "pasting" && (
          <button
            onClick={onOpenSettings}
            className="absolute top-1 right-1 p-1.5 rounded-lg transition-colors"
            style={{ color: "var(--text-muted)" }}
            title="Settings"
          >
            <Settings size={13} />
          </button>
        )}
      </div>
    </div>
  );
}
