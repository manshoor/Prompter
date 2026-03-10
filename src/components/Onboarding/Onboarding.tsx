import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ApiKeyInput } from "../SettingsPanel/ApiKeyInput";

interface Props {
  onComplete: () => void;
}

export function Onboarding({ onComplete }: Props) {
  const [step, setStep] = useState(0);

  const steps = [
    {
      title: "Welcome to Prompter",
      content: (
        <div className="space-y-3 text-gray-300">
          <p>Prompter turns your voice + selected code into polished AI prompts.</p>
          <p>Let's get you set up in 3 quick steps.</p>
        </div>
      ),
    },
    {
      title: "Add an API Key",
      content: (
        <div className="space-y-4">
          <p className="text-sm text-gray-400">You need at least one API key for prompt generation.</p>
          <ApiKeyInput provider="openai" label="OpenAI API Key" />
          <ApiKeyInput provider="claude" label="Claude API Key" />
        </div>
      ),
    },
    {
      title: "You're Ready!",
      content: (
        <div className="space-y-3 text-gray-300">
          <p>Press <kbd className="px-2 py-1 bg-gray-800 border border-gray-700 rounded text-sm">Cmd+Shift+P</kbd> anywhere to start.</p>
          <p>Select text, speak your intent, and get a polished prompt.</p>
        </div>
      ),
    },
  ];

  const handleNext = async () => {
    if (step < steps.length - 1) {
      setStep(step + 1);
    } else {
      await invoke("complete_onboarding");
      onComplete();
    }
  };

  return (
    <div className="min-h-screen bg-gray-950 flex items-center justify-center">
      <div className="w-[420px] bg-gray-900 border border-gray-800 rounded-xl p-8 shadow-2xl">
        <h1 className="text-xl font-semibold text-white mb-6">{steps[step].title}</h1>
        {steps[step].content}
        <div className="flex justify-between mt-8">
          {step > 0 ? (
            <button
              onClick={() => setStep(step - 1)}
              className="px-4 py-2 text-sm text-gray-400 hover:text-gray-200 transition-colors"
            >
              Back
            </button>
          ) : <div />}
          <button
            onClick={handleNext}
            className="px-6 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg text-sm transition-colors"
          >
            {step < steps.length - 1 ? "Next" : "Get Started"}
          </button>
        </div>
      </div>
    </div>
  );
}
