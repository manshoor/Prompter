import { useEffect, useState, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Square } from "lucide-react";
import { motion } from "framer-motion";
import { useRecordingStore } from "../../stores/recordingStore";

function AudioVisualizer() {
  const [levels, setLevels] = useState([0, 0, 0, 0, 0]);
  const animRef = useRef<number>(0);
  const state = useRecordingStore((s) => s.state);

  useEffect(() => {
    if (state !== "recording") return;

    let active = true;
    const poll = async () => {
      if (!active) return;
      try {
        const rms = await invoke<number>("get_audio_level");
        const normalized = Math.min(rms * 8, 1); // Scale up for visibility
        setLevels((prev) => {
          const next = [...prev.slice(1), normalized];
          return next;
        });
      } catch {
        // ignore
      }
      animRef.current = requestAnimationFrame(() => {
        setTimeout(poll, 80);
      });
    };
    poll();
    return () => {
      active = false;
      cancelAnimationFrame(animRef.current);
    };
  }, [state]);

  return (
    <div className="flex items-end gap-[3px] h-6">
      {levels.map((level, i) => (
        <motion.div
          key={i}
          className="w-[3px] rounded-full"
          style={{ background: "var(--danger)" }}
          animate={{ height: Math.max(4, level * 24) }}
          transition={{ type: "spring", stiffness: 300, damping: 20 }}
        />
      ))}
    </div>
  );
}

function formatTime(secs: number): string {
  const m = Math.floor(secs / 60);
  const s = Math.floor(secs % 60);
  return `${m}:${s.toString().padStart(2, "0")}`;
}

export function RecordingView() {
  const { state, downloadProgress, stopRecording } = useRecordingStore();
  const [elapsed, setElapsed] = useState(0);

  useEffect(() => {
    if (state !== "recording") {
      setElapsed(0);
      return;
    }
    const start = Date.now();
    const interval = setInterval(() => {
      setElapsed((Date.now() - start) / 1000);
    }, 200);
    return () => clearInterval(interval);
  }, [state]);

  if (state === "downloading") {
    return (
      <div className="flex flex-col items-center gap-3 px-4 pb-4 pt-2">
        <span className="text-xs font-medium" style={{ color: "var(--text-secondary)" }}>
          Downloading Whisper model...
        </span>
        <div className="w-full h-1.5 rounded-full overflow-hidden" style={{ background: "var(--bg-tertiary)" }}>
          <motion.div
            className="h-full rounded-full"
            style={{ background: "var(--accent)" }}
            animate={{ width: `${downloadProgress}%` }}
            transition={{ duration: 0.3 }}
          />
        </div>
        <span className="text-[10px]" style={{ color: "var(--text-muted)" }}>
          {downloadProgress}%
        </span>
      </div>
    );
  }

  return (
    <div className="flex flex-col items-center gap-3 px-4 pb-4 pt-2">
      <div className="flex items-center gap-3">
        {state === "recording" ? (
          <>
            <div className="w-2.5 h-2.5 rounded-full animate-recording" style={{ background: "var(--danger)" }} />
            <AudioVisualizer />
            <span className="text-xs font-mono tabular-nums" style={{ color: "var(--text-muted)" }}>
              {formatTime(elapsed)}
            </span>
          </>
        ) : (
          <>
            <div
              className="w-3.5 h-3.5 border-2 rounded-full animate-spin"
              style={{ borderColor: "var(--accent)", borderTopColor: "transparent" }}
            />
            <span className="text-xs font-medium" style={{ color: "var(--text-secondary)" }}>
              {state === "transcribing" && "Transcribing..."}
              {state === "generating" && "Generating prompt..."}
            </span>
          </>
        )}
      </div>
      {state === "recording" && (
        <button
          onClick={stopRecording}
          className="w-10 h-10 flex items-center justify-center rounded-full text-white transition-all active:scale-90 hover:brightness-110"
          style={{ background: "var(--danger)" }}
          title="Stop Recording"
        >
          <Square size={14} fill="white" />
        </button>
      )}
    </div>
  );
}
