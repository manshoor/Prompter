import { useEffect, useCallback, type ReactNode } from "react";
import { Settings, Key, Mic, Brain, MousePointer, Keyboard, Palette } from "lucide-react";
import {
  useSettingsStore,
  type AppSettings,
  type WhisperMode,
  type PromptModel,
  type PasteMode,
  type RecordingMode,
  type Theme,
} from "../../stores/settingsStore";
import { ApiKeyInput } from "./ApiKeyInput";

function Section({
  icon,
  title,
  children,
}: {
  icon: ReactNode;
  title: string;
  children: ReactNode;
}) {
  return (
    <section className="mb-6 rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 p-5">
      <h2 className="mb-4 flex items-center gap-2 text-base font-semibold text-gray-900 dark:text-gray-100">
        {icon}
        {title}
      </h2>
      {children}
    </section>
  );
}

function SelectField<T extends string>({
  label,
  value,
  options,
  onChange,
}: {
  label: string;
  value: T;
  options: { value: T; label: string }[];
  onChange: (value: T) => void;
}) {
  return (
    <div className="mb-3">
      <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
        {label}
      </label>
      <select
        value={value}
        onChange={(e) => onChange(e.target.value as T)}
        className="w-full rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 px-3 py-2 text-sm text-gray-900 dark:text-gray-100 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
      >
        {options.map((opt) => (
          <option key={opt.value} value={opt.value}>
            {opt.label}
          </option>
        ))}
      </select>
    </div>
  );
}

export function SettingsPanel() {
  const { settings, loading, load, save } = useSettingsStore();

  useEffect(() => {
    load();
  }, [load]);

  const update = useCallback(
    (patch: Partial<AppSettings>) => {
      if (!settings) return;
      const updated = { ...settings, ...patch };
      save(updated);
    },
    [settings, save],
  );

  if (loading || !settings) {
    return (
      <div className="flex h-screen items-center justify-center bg-gray-50 dark:bg-gray-900">
        <p className="text-gray-500 dark:text-gray-400">Loading settings...</p>
      </div>
    );
  }

  return (
    <div className="settings-panel min-h-screen bg-gray-50 dark:bg-gray-900 p-6">
      <div className="mx-auto max-w-2xl">
        <h1 className="mb-6 flex items-center gap-2 text-xl font-bold text-gray-900 dark:text-gray-100">
          <Settings size={24} />
          Settings
        </h1>

        {/* API Keys */}
        <Section icon={<Key size={18} />} title="API Keys">
          <ApiKeyInput provider="openai" label="OpenAI API Key" />
          <ApiKeyInput provider="claude" label="Claude API Key" />
        </Section>

        {/* Speech-to-Text */}
        <Section icon={<Mic size={18} />} title="Speech-to-Text">
          <SelectField<WhisperMode>
            label="Whisper Mode"
            value={settings.whisper_mode}
            options={[
              { value: "Cloud", label: "Cloud (OpenAI)" },
              { value: "Local", label: "Local (whisper.cpp)" },
            ]}
            onChange={(v) => update({ whisper_mode: v })}
          />
          {settings.whisper_mode === "Local" && (
            <SelectField
              label="Local Model"
              value={settings.local_whisper_model}
              options={[
                { value: "tiny", label: "Tiny" },
                { value: "base", label: "Base" },
                { value: "small", label: "Small" },
                { value: "medium", label: "Medium" },
                { value: "large", label: "Large" },
              ]}
              onChange={(v) => update({ local_whisper_model: v })}
            />
          )}
        </Section>

        {/* Prompt Generation */}
        <Section icon={<Brain size={18} />} title="Prompt Generation">
          <SelectField<PromptModel>
            label="Model"
            value={settings.prompt_model}
            options={[
              { value: "ClaudeSonnet", label: "Claude Sonnet" },
              { value: "ClaudeOpus", label: "Claude Opus" },
              { value: "ClaudeHaiku", label: "Claude Haiku" },
              { value: "Gpt4o", label: "GPT-4o" },
              { value: "Gpt4oMini", label: "GPT-4o Mini" },
            ]}
            onChange={(v) => update({ prompt_model: v })}
          />
        </Section>

        {/* Behavior */}
        <Section icon={<MousePointer size={18} />} title="Behavior">
          <SelectField<PasteMode>
            label="Paste Mode"
            value={settings.paste_mode}
            options={[
              { value: "Auto", label: "Auto (paste immediately)" },
              { value: "Preview", label: "Preview (review before pasting)" },
            ]}
            onChange={(v) => update({ paste_mode: v })}
          />
          <SelectField<RecordingMode>
            label="Recording Mode"
            value={settings.recording_mode}
            options={[
              { value: "PushToTalk", label: "Push to Talk" },
              { value: "Toggle", label: "Toggle" },
              { value: "AutoDetect", label: "Auto Detect (voice activity)" },
            ]}
            onChange={(v) => update({ recording_mode: v })}
          />
          {settings.recording_mode === "AutoDetect" && (
            <div className="mb-3">
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                Silence Threshold: {settings.silence_threshold_secs.toFixed(1)}s
              </label>
              <input
                type="range"
                min="1"
                max="10"
                step="0.5"
                value={settings.silence_threshold_secs}
                onChange={(e) =>
                  update({ silence_threshold_secs: parseFloat(e.target.value) })
                }
                className="w-full accent-blue-600"
              />
              <div className="flex justify-between text-xs text-gray-400">
                <span>1s</span>
                <span>10s</span>
              </div>
            </div>
          )}
        </Section>

        {/* Global Hotkey */}
        <Section icon={<Keyboard size={18} />} title="Global Hotkey">
          <div className="mb-3">
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              Hotkey
            </label>
            <input
              type="text"
              value={settings.global_hotkey}
              onChange={(e) => update({ global_hotkey: e.target.value })}
              placeholder="e.g. CmdOrCtrl+Shift+P"
              className="w-full rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 px-3 py-2 text-sm text-gray-900 dark:text-gray-100 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
            />
            <p className="mt-1 text-xs text-gray-400">
              Use modifier keys like CmdOrCtrl, Shift, Alt, plus a key.
            </p>
          </div>
        </Section>

        {/* Appearance */}
        <Section icon={<Palette size={18} />} title="Appearance">
          <SelectField<Theme>
            label="Theme"
            value={settings.theme}
            options={[
              { value: "System", label: "System" },
              { value: "Light", label: "Light" },
              { value: "Dark", label: "Dark" },
            ]}
            onChange={(v) => update({ theme: v })}
          />
        </Section>
      </div>
    </div>
  );
}
