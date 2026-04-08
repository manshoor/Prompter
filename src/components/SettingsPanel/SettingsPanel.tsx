import { useEffect, useCallback, useState, type ReactNode } from "react";
import { Key, Mic, Brain, MousePointer, Keyboard, Palette } from "lucide-react";
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

type SectionId = "keys" | "speech" | "prompt" | "behavior" | "hotkey" | "appearance";

const sections: { id: SectionId; label: string; icon: ReactNode }[] = [
  { id: "keys", label: "API Keys", icon: <Key size={14} /> },
  { id: "speech", label: "Speech", icon: <Mic size={14} /> },
  { id: "prompt", label: "Prompt", icon: <Brain size={14} /> },
  { id: "behavior", label: "Behavior", icon: <MousePointer size={14} /> },
  { id: "hotkey", label: "Hotkey", icon: <Keyboard size={14} /> },
  { id: "appearance", label: "Appearance", icon: <Palette size={14} /> },
];

function SegmentedControl<T extends string>({
  value,
  options,
  onChange,
}: {
  value: T;
  options: { value: T; label: string }[];
  onChange: (value: T) => void;
}) {
  return (
    <div className="segmented-control">
      {options.map((opt) => (
        <button
          key={opt.value}
          className={opt.value === value ? "active" : ""}
          onClick={() => onChange(opt.value)}
        >
          {opt.label}
        </button>
      ))}
    </div>
  );
}

function Row({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div className="flex items-center justify-between py-2">
      <span className="text-xs font-medium" style={{ color: "var(--text-secondary)" }}>
        {label}
      </span>
      <div>{children}</div>
    </div>
  );
}

function SelectField<T extends string>({
  value,
  options,
  onChange,
}: {
  value: T;
  options: { value: T; label: string }[];
  onChange: (value: T) => void;
}) {
  return (
    <select
      value={value}
      onChange={(e) => onChange(e.target.value as T)}
      className="rounded-md px-2 py-1 text-xs"
      style={{
        background: "var(--input-bg)",
        color: "var(--text-primary)",
        border: "1px solid var(--input-border)",
      }}
    >
      {options.map((opt) => (
        <option key={opt.value} value={opt.value}>
          {opt.label}
        </option>
      ))}
    </select>
  );
}

export function SettingsPanel() {
  const { settings, loading, load, save } = useSettingsStore();
  const [activeSection, setActiveSection] = useState<SectionId>("keys");

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
      <div className="flex h-full items-center justify-center">
        <p className="text-sm" style={{ color: "var(--text-muted)" }}>Loading...</p>
      </div>
    );
  }

  const renderContent = () => {
    switch (activeSection) {
      case "keys":
        return (
          <div className="space-y-3">
            <ApiKeyInput provider="openai" label="OpenAI API Key" />
            <ApiKeyInput provider="claude" label="Claude API Key" />
          </div>
        );

      case "speech":
        return (
          <div className="space-y-2">
            <Row label="Whisper Mode">
              <SegmentedControl<WhisperMode>
                value={settings.whisper_mode}
                options={[
                  { value: "Cloud", label: "Cloud" },
                  { value: "Local", label: "Local" },
                ]}
                onChange={(v) => update({ whisper_mode: v })}
              />
            </Row>
            {settings.whisper_mode === "Local" && (
              <Row label="Local Model">
                <SelectField
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
              </Row>
            )}
          </div>
        );

      case "prompt":
        return (
          <Row label="Model">
            <SelectField<PromptModel>
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
          </Row>
        );

      case "behavior":
        return (
          <div className="space-y-2">
            <Row label="Paste Mode">
              <SegmentedControl<PasteMode>
                value={settings.paste_mode}
                options={[
                  { value: "Auto", label: "Auto" },
                  { value: "Preview", label: "Preview" },
                ]}
                onChange={(v) => update({ paste_mode: v })}
              />
            </Row>
            <Row label="Recording Mode">
              <SegmentedControl<RecordingMode>
                value={settings.recording_mode}
                options={[
                  { value: "PushToTalk", label: "Push" },
                  { value: "Toggle", label: "Toggle" },
                  { value: "AutoDetect", label: "Auto" },
                ]}
                onChange={(v) => update({ recording_mode: v })}
              />
            </Row>
            {settings.recording_mode === "AutoDetect" && (
              <div className="pt-1">
                <div className="flex items-center justify-between mb-1">
                  <span className="text-xs" style={{ color: "var(--text-secondary)" }}>
                    Silence Threshold
                  </span>
                  <span className="text-xs font-mono tabular-nums" style={{ color: "var(--text-muted)" }}>
                    {settings.silence_threshold_secs.toFixed(1)}s
                  </span>
                </div>
                <input
                  type="range"
                  min="1"
                  max="10"
                  step="0.5"
                  value={settings.silence_threshold_secs}
                  onChange={(e) =>
                    update({ silence_threshold_secs: parseFloat(e.target.value) })
                  }
                  className="w-full"
                  style={{ accentColor: "var(--accent)" }}
                />
              </div>
            )}
          </div>
        );

      case "hotkey":
        return (
          <div>
            <input
              type="text"
              value={settings.global_hotkey}
              onChange={(e) => update({ global_hotkey: e.target.value })}
              placeholder="e.g. CmdOrCtrl+Shift+P"
              className="w-full rounded-md px-2.5 py-1.5 text-xs"
              style={{
                background: "var(--input-bg)",
                color: "var(--text-primary)",
                border: "1px solid var(--input-border)",
              }}
            />
            <p className="mt-1.5 text-[10px]" style={{ color: "var(--text-muted)" }}>
              Use CmdOrCtrl, Shift, Alt + a key. Changes apply immediately.
            </p>
          </div>
        );

      case "appearance":
        return (
          <Row label="Theme">
            <SegmentedControl<Theme>
              value={settings.theme}
              options={[
                { value: "System", label: "System" },
                { value: "Light", label: "Light" },
                { value: "Dark", label: "Dark" },
              ]}
              onChange={(v) => update({ theme: v })}
            />
          </Row>
        );
    }
  };

  return (
    <div className="flex h-full">
      {/* Sidebar */}
      <nav
        className="w-36 shrink-0 py-2 px-2 space-y-0.5 border-r"
        style={{ borderColor: "var(--border-color)" }}
      >
        {sections.map((s) => (
          <button
            key={s.id}
            onClick={() => setActiveSection(s.id)}
            className={`w-full flex items-center gap-2 px-2.5 py-1.5 rounded-md text-xs font-medium transition-colors text-left ${
              activeSection === s.id ? "active-section" : ""
            }`}
            style={{
              background: activeSection === s.id ? "var(--accent)" : "transparent",
              color: activeSection === s.id ? "white" : "var(--text-secondary)",
            }}
          >
            {s.icon}
            {s.label}
          </button>
        ))}
      </nav>

      {/* Content */}
      <div className="flex-1 p-4 overflow-y-auto">
        <h2 className="text-sm font-semibold mb-3" style={{ color: "var(--text-primary)" }}>
          {sections.find((s) => s.id === activeSection)?.label}
        </h2>
        {renderContent()}
      </div>
    </div>
  );
}
