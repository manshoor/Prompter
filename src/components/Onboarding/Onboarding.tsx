import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ApiKeyInput } from "../SettingsPanel/ApiKeyInput";
import { useSettingsStore, type WhisperMode } from "../../stores/settingsStore";

interface Props {
  onComplete: () => void;
}

export function Onboarding({ onComplete }: Props) {
  const [step, setStep] = useState(0);
  const [whisperMode, setWhisperMode] = useState<WhisperMode>("Cloud");
  const [downloadStatus, setDownloadStatus] = useState<"idle" | "downloading" | "done" | "error">("idle");
  const [downloadError, setDownloadError] = useState("");
  const { updateWhisperMode } = useSettingsStore();

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
    // Skip model download step if Cloud mode
    if (step === 2 && whisperMode === "Cloud") {
      setStep(4); // Jump to Ready
    } else if (step < 4) {
      setStep(step + 1);
    } else {
      await invoke("complete_onboarding");
      onComplete();
    }
  };

  const handleBack = () => {
    // If on Ready and Cloud mode, go back to API Keys (skip download)
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
        <div className="space-y-3 text-gray-300">
          <p>Turn your voice + selected code into polished AI prompts.</p>
          <p className="text-sm text-gray-400">
            Select text in any app, press{" "}
            <kbd className="px-2 py-0.5 bg-gray-800 border border-gray-700 rounded text-xs">
              Cmd+Shift+P
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
        <div className="space-y-3">
          <button
            onClick={() => handleWhisperSelect("Cloud")}
            className={`w-full text-left p-4 rounded-lg border transition-colors ${
              whisperMode === "Cloud"
                ? "border-blue-500 bg-blue-950/30"
                : "border-gray-700 bg-gray-800 hover:border-gray-600"
            }`}
          >
            <div className="flex items-center justify-between mb-1">
              <span className="font-medium text-gray-100">Cloud (OpenAI Whisper API)</span>
              {whisperMode === "Cloud" && (
                <span className="text-xs text-blue-400">Selected</span>
              )}
            </div>
            <p className="text-sm text-gray-400">
              Fast and accurate. Sends audio to OpenAI for transcription. Requires an OpenAI API key.
            </p>
          </button>
          <button
            onClick={() => handleWhisperSelect("Local")}
            className={`w-full text-left p-4 rounded-lg border transition-colors ${
              whisperMode === "Local"
                ? "border-blue-500 bg-blue-950/30"
                : "border-gray-700 bg-gray-800 hover:border-gray-600"
            }`}
          >
            <div className="flex items-center justify-between mb-1">
              <span className="font-medium text-gray-100">Local (on-device)</span>
              {whisperMode === "Local" && (
                <span className="text-xs text-blue-400">Selected</span>
              )}
            </div>
            <p className="text-sm text-gray-400">
              Private — nothing leaves your Mac. Requires a ~1.5 GB model download.
            </p>
          </button>
        </div>
      ),
    },
    // Step 2: API Keys (dynamic)
    {
      title: "Add Your API Keys",
      content: (
        <div className="space-y-4">
          {whisperMode === "Cloud" && (
            <div>
              <p className="text-xs text-gray-400 mb-2">
                Used for voice transcription (Whisper API)
              </p>
              <ApiKeyInput provider="openai" label="OpenAI API Key" />
            </div>
          )}
          <div>
            <p className="text-xs text-gray-400 mb-2">
              Used for generating polished prompts
            </p>
            <ApiKeyInput provider="claude" label="Claude API Key" />
          </div>
          {whisperMode === "Local" && (
            <p className="text-xs text-gray-500">
              No OpenAI key needed — transcription runs locally on your Mac.
            </p>
          )}
        </div>
      ),
    },
    // Step 3: Model Download (only shown for Local mode)
    {
      title: "Download Whisper Model",
      content: (
        <div className="space-y-4">
          <p className="text-sm text-gray-300">
            Local transcription needs the Whisper medium model (~1.5 GB).
          </p>
          {downloadStatus === "idle" && (
            <div className="space-y-3">
              <button
                onClick={handleDownloadModel}
                className="w-full px-4 py-3 bg-blue-600 hover:bg-blue-700 text-white rounded-lg text-sm transition-colors"
              >
                Download Now
              </button>
              <button
                onClick={handleNext}
                className="w-full px-4 py-2 text-sm text-gray-400 hover:text-gray-200 transition-colors"
              >
                Download Later (will download on first use)
              </button>
            </div>
          )}
          {downloadStatus === "downloading" && (
            <div className="flex items-center gap-3 p-4 bg-gray-800 rounded-lg">
              <div className="w-4 h-4 border-2 border-blue-500 border-t-transparent rounded-full animate-spin" />
              <span className="text-sm text-gray-300">Downloading model...</span>
            </div>
          )}
          {downloadStatus === "done" && (
            <div className="p-4 bg-green-950/30 border border-green-800 rounded-lg">
              <span className="text-sm text-green-300">Model downloaded successfully!</span>
            </div>
          )}
          {downloadStatus === "error" && (
            <div className="space-y-2">
              <div className="p-3 bg-red-950/30 border border-red-800 rounded-lg">
                <span className="text-sm text-red-300">{downloadError}</span>
              </div>
              <button
                onClick={handleDownloadModel}
                className="px-4 py-2 bg-gray-700 hover:bg-gray-600 text-gray-200 rounded-lg text-sm transition-colors"
              >
                Retry
              </button>
              <button
                onClick={handleNext}
                className="px-4 py-2 text-sm text-gray-400 hover:text-gray-200 transition-colors"
              >
                Skip for now
              </button>
            </div>
          )}
        </div>
      ),
    },
    // Step 4: Ready
    {
      title: "You're All Set!",
      content: (
        <div className="space-y-3 text-gray-300">
          <p>
            Press{" "}
            <kbd className="px-2 py-0.5 bg-gray-800 border border-gray-700 rounded text-xs">
              Cmd+Shift+P
            </kbd>{" "}
            anywhere to start recording.
          </p>
          <p className="text-sm text-gray-400">
            Select text, speak your intent, and Prompter will generate a polished prompt for you.
          </p>
          <div className="mt-4 p-3 bg-gray-800 rounded-lg text-sm text-gray-400 space-y-1">
            <p>
              <span className="text-gray-300">Transcription:</span>{" "}
              {whisperMode === "Cloud" ? "Cloud (OpenAI)" : "Local (on-device)"}
            </p>
            <p>
              <span className="text-gray-300">Prompt generation:</span> Claude Sonnet
            </p>
          </div>
          <p className="text-xs text-gray-500">
            You can change these anytime via the gear icon or tray menu.
          </p>
        </div>
      ),
    },
  ];

  const current = stepContent[step];
  const isLastStep = step === 4;
  // Hide next button on model download step (it has its own navigation)
  const showNextButton = step !== 3 || downloadStatus === "done";

  return (
    <div className="min-h-screen bg-gray-950 flex items-center justify-center">
      <div className="w-[440px] bg-gray-900 border border-gray-800 rounded-xl p-8 shadow-2xl">
        {/* Step indicator */}
        <div className="flex gap-1.5 mb-6">
          {[0, 1, 2, 3, 4]
            .filter((i) => !(i === 3 && whisperMode === "Cloud"))
            .map((i) => (
              <div
                key={i}
                className={`h-1 flex-1 rounded-full transition-colors ${
                  i <= step ? "bg-blue-500" : "bg-gray-700"
                }`}
              />
            ))}
        </div>

        <h1 className="text-xl font-semibold text-white mb-6">{current.title}</h1>
        {current.content}

        <div className="flex justify-between mt-8">
          {step > 0 ? (
            <button
              onClick={handleBack}
              className="px-4 py-2 text-sm text-gray-400 hover:text-gray-200 transition-colors"
            >
              Back
            </button>
          ) : (
            <div />
          )}
          {showNextButton && (
            <button
              onClick={handleNext}
              className="px-6 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg text-sm transition-colors"
            >
              {isLastStep ? "Get Started" : "Next"}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
