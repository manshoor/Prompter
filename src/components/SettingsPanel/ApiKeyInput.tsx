import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Props {
  provider: string;
  label: string;
}

export function ApiKeyInput({ provider, label }: Props) {
  const [key, setKey] = useState("");
  const [saved, setSaved] = useState(false);
  const [hasKey, setHasKey] = useState(false);

  useEffect(() => {
    invoke<string | null>("get_api_key", { provider }).then((existing) => {
      if (existing) {
        setHasKey(true);
        setKey("••••••••••••••••");
      }
    });
  }, [provider]);

  const handleSave = async () => {
    if (key && !key.startsWith("••")) {
      await invoke("store_api_key", { provider, key });
      setHasKey(true);
      setSaved(true);
      setKey("••••••••••••••••");
      setTimeout(() => setSaved(false), 2000);
    }
  };

  const handleDelete = async () => {
    await invoke("delete_api_key", { provider });
    setHasKey(false);
    setKey("");
  };

  return (
    <div className="api-key-input mb-4">
      <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
        {label}
      </label>
      <div className="input-row flex items-center gap-2">
        <input
          type="password"
          value={key}
          onChange={(e) => setKey(e.target.value)}
          placeholder={`Enter ${label}`}
          onFocus={() => {
            if (hasKey) setKey("");
          }}
          className="flex-1 rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 px-3 py-2 text-sm text-gray-900 dark:text-gray-100 placeholder-gray-400 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
        />
        <button
          onClick={handleSave}
          disabled={!key || key.startsWith("••")}
          className="rounded-md bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {saved ? "Saved" : "Save"}
        </button>
        {hasKey && (
          <button
            onClick={handleDelete}
            className="rounded-md bg-red-600 px-4 py-2 text-sm font-medium text-white hover:bg-red-700"
          >
            Remove
          </button>
        )}
      </div>
    </div>
  );
}
