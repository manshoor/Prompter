import { useState, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { motion, AnimatePresence } from "framer-motion";
import { ApiKeyInput } from "../SettingsPanel/ApiKeyInput";
import { useSettingsStore, type WhisperMode } from "../../stores/settingsStore";

interface Props {
  onComplete: () => void;
}

const slideVariants = {
  enter: { opacity: 0, x: 20 },
  center: { opacity: 1, x: 0 },
  exit: { opacity: 0, x: -20 },
};

export function Onboarding({ onComplete }: Props) {
  const [step, setStep] = useState(0);
  const [whisperMode, setWhisperMode] = useState<WhisperMode>("Cloud");
  const [downloadStatus, setDownloadStatus] = useState<"idle" | "downloading" | "done" | "error">("idle");
  const [downloadError, setDownloadError] = useState("");
  const { settings, updateWhisperMode } = useSettingsStore();

  const hotkey = useMemo(() => {
    const raw = settings?.global_hotkey ?? "CmdOrCtrl+Shift+P";
    return raw.replace("CmdOrCtrl", "\u2318");
  }, [settings?.global_hotkey]);

  const handleWhisperSelect = async (mode: WhisperMode) => {
    setWhisperMode(mode);
    await updateWhisperMode(mode);
  };

  const handleDownloadModel = async () => {
    setDownloadStatus("downloading");
    try {
      await invoke("download_whisper_model", { model: "medium" });
      setDownloadStatus("done");
    } catch (e) {
      setDownloadStatus("error");
      setDownloadError(String(e));
    }
  };

  const handleNext = async () => {
    if (step === 2 && whisperMode === "Cloud") {
      setStep(4);
    } else if (step < 4) {
      setStep(step + 1);
    } else {
      await invoke("complete_onboarding");
      onComplete();
    }
  };

  const handleBack = () => {
    if (step === 4 && whisperMode === "Cloud") {
      setStep(2);
    } else {
      setStep(step - 1);
    }
  };

  const stepContent = [
    // Step 0: Welcome
    {
      title: "Welcome to Prompter",
      content: (
        <div className="space-y-3">
          <p className="text-sm" style={{ color: "var(--text-secondary)" }}>
            Turn your voice + selected code into polished AI prompts.
          </p>
          <p className="text-xs" style={{ color: "var(--text-muted)" }}>
            Select text in any app, press{" "}
            <kbd
              className="px-1.5 py-0.5 text-[10px] rounded"
              style={{
                background: "var(--bg-tertiary)",
                border: "1px solid var(--border-color)",
                color: "var(--text-secondary)",
              }}
            >
              {hotkey}
            </kbd>
            , speak your intent, and get a ready-to-use prompt.
          </p>
        </div>
      ),
    },
    // Step 1: Whisper Mode
    {
      title: "How should voice be transcribed?",
      content: (
        <div className="space-y-2.5">
          {(["Cloud", "Local"] as WhisperMode[]).map((mode) => (
            <button
              key={mode}
              onClick={() => handleWhisperSelect(mode)}
              className="w-full text-left p-3.5 rounded-lg transition-all"
              style={{
                background: whisperMode === mode ? "rgba(0,122,255,0.08)" : "var(--bg-secondary)",
                border: `1px solid ${whisperMode === mode ? "var(--accent)" : "var(--border-color)"}`,
              }}
            >
              <div className="flex items-center justify-between mb-0.5">
                <span className="text-sm font-medium" style={{ color: "var(--text-primary)" }}>
                  {mode === "Cloud" ? "Cloud (OpenAI Whisper API)" : "Local (on-device)"}
                </span>
                {whisperMode === mode && (
                  <span className="text-[10px] font-medium" style={{ color: "var(--accent)" }}>
                    Selected
                  </span>
                )}
              </div>
              <p className="text-xs" style={{ color: "var(--text-muted)" }}>
                {mode === "Cloud"
                  ? "Fast and accurate. Sends audio to OpenAI. Requires an API key."
                  : "Private \u2014 nothing leaves your Mac. Requires a ~1.5 GB model download."}
              </p>
            </button>
          ))}
        </div>
      ),
    },
    // Step 2: API Keys
    {
      title: "Add Your API Keys",
      content: (
        <div className="space-y-3">
          {whisperMode === "Cloud" && (
            <div>
              <p className="text-[10px] mb-1.5" style={{ color: "var(--text-muted)" }}>
                Used for voice transcription (Whisper API)
              </p>
              <ApiKeyInput provider="openai" label="OpenAI API Key" />
            </div>
          )}
          <div>
            <p className="text-[10px] mb-1.5" style={{ color: "var(--text-muted)" }}>
              Used for generating polished prompts
            </p>
            <ApiKeyInput provider="claude" label="Claude API Key" />
          </div>
          {whisperMode === "Local" && (
            <p className="text-[10px]" style={{ color: "var(--text-muted)" }}>
              No OpenAI key needed \u2014 transcription runs locally on your Mac.
            </p>
          )}
        </div>
      ),
    },
    // Step 3: Model Download (Local only)
    {
      title: "Download Whisper Model",
      content: (
        <div className="space-y-3">
          <p className="text-sm" style={{ color: "var(--text-secondary)" }}>
            Local transcription needs the Whisper medium model (~1.5 GB).
          </p>
          {downloadStatus === "idle" && (
            <div className="space-y-2">
              <button
                onClick={handleDownloadModel}
                className="w-full px-4 py-2.5 text-white rounded-lg text-sm font-medium transition-all active:scale-[0.98]"
                style={{ background: "var(--accent)" }}
              >
                Download Now
              </button>
              <button
                onClick={handleNext}
                className="w-full px-4 py-2 text-sm transition-colors"
                style={{ color: "var(--text-muted)" }}
              >
                Download Later
              </button>
            </div>
          )}
          {downloadStatus === "downloading" && (
            <div className="flex items-center gap-2.5 p-3 rounded-lg" style={{ background: "var(--bg-secondary)" }}>
              <div
                className="w-3.5 h-3.5 border-2 rounded-full animate-spin"
                style={{ borderColor: "var(--accent)", borderTopColor: "transparent" }}
              />
              <span className="text-xs" style={{ color: "var(--text-secondary)" }}>Downloading model...</span>
            </div>
          )}
          {downloadStatus === "done" && (
            <div className="p-3 rounded-lg" style={{ background: "rgba(52,199,89,0.1)", border: "1px solid rgba(52,199,89,0.3)" }}>
              <span className="text-xs" style={{ color: "var(--success)" }}>Model downloaded successfully!</span>
            </div>
          )}
          {downloadStatus === "error" && (
            <div className="space-y-2">
              <div className="p-2.5 rounded-lg" style={{ background: "rgba(255,59,48,0.08)", border: "1px solid rgba(255,59,48,0.2)" }}>
                <span className="text-xs" style={{ color: "var(--danger)" }}>{downloadError}</span>
              </div>
              <button
                onClick={handleDownloadModel}
                className="px-3 py-1.5 text-sm rounded-lg"
                style={{ background: "var(--bg-secondary)", color: "var(--text-secondary)", border: "1px solid var(--border-color)" }}
              >
                Retry
              </button>
            </div>
          )}
        </div>
      ),
    },
    // Step 4: Ready
    {
      title: "You\u2019re All Set!",
      content: (
        <div className="space-y-3">
          <p className="text-sm" style={{ color: "var(--text-secondary)" }}>
            Press{" "}
            <kbd
              className="px-1.5 py-0.5 text-[10px] rounded"
              style={{
                background: "var(--bg-tertiary)",
                border: "1px solid var(--border-color)",
                color: "var(--text-secondary)",
              }}
            >
              {hotkey}
            </kbd>{" "}
            anywhere to start.
          </p>
          <div className="p-3 rounded-lg text-xs space-y-1" style={{ background: "var(--bg-secondary)", border: "1px solid var(--border-color)" }}>
            <p>
              <span style={{ color: "var(--text-primary)" }}>Transcription:</span>{" "}
              <span style={{ color: "var(--text-muted)" }}>
                {whisperMode === "Cloud" ? "Cloud (OpenAI)" : "Local (on-device)"}
              </span>
            </p>
            <p>
              <span style={{ color: "var(--text-primary)" }}>Prompt generation:</span>{" "}
              <span style={{ color: "var(--text-muted)" }}>Claude Sonnet</span>
            </p>
          </div>
          <p className="text-[10px]" style={{ color: "var(--text-muted)" }}>
            You can change these anytime via the gear icon or tray menu.
          </p>
        </div>
      ),
    },
  ];

  const current = stepContent[step];
  const isLastStep = step === 4;
  const showNextButton = step !== 3 || downloadStatus === "done";

  return (
    <div className="h-screen flex items-center justify-center rounded-xl glass shadow-2xl overflow-hidden">
      <div className="w-full max-w-sm px-8 py-6">
        {/* Step indicator */}
        <div className="flex gap-1.5 mb-5">
          {[0, 1, 2, 3, 4]
            .filter((i) => !(i === 3 && whisperMode === "Cloud"))
            .map((i) => (
              <div
                key={i}
                className="h-1 flex-1 rounded-full transition-colors"
                style={{
                  background: i <= step ? "var(--accent)" : "var(--bg-tertiary)",
                }}
              />
            ))}
        </div>

        <AnimatePresence mode="wait">
          <motion.div
            key={step}
            variants={slideVariants}
            initial="enter"
            animate="center"
            exit="exit"
            transition={{ duration: 0.2 }}
          >
            <h1 className="text-lg font-semibold mb-4" style={{ color: "var(--text-primary)" }}>
              {current.title}
            </h1>
            {current.content}
          </motion.div>
        </AnimatePresence>

        <div className="flex justify-between mt-6">
          {step > 0 ? (
            <button
              onClick={handleBack}
              className="px-3 py-1.5 text-sm transition-colors"
              style={{ color: "var(--text-muted)" }}
            >
              Back
            </button>
          ) : (
            <div />
          )}
          {showNextButton && (
            <button
              onClick={handleNext}
              className="px-5 py-1.5 text-white rounded-lg text-sm font-medium transition-all active:scale-[0.98]"
              style={{ background: "var(--accent)" }}
            >
              {isLastStep ? "Get Started" : "Next"}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
