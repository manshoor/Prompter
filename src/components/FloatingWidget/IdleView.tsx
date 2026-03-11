import { Settings } from "lucide-react";

interface Props {
  onOpenSettings: () => void;
}

export function IdleView({ onOpenSettings }: Props) {
  return (
    <div className="relative flex flex-col items-center justify-center gap-4 p-8">
      <button
        onClick={onOpenSettings}
        className="absolute top-3 right-3 p-1.5 text-gray-500 hover:text-gray-300 transition-colors rounded-lg hover:bg-gray-800"
        title="Settings"
      >
        <Settings size={16} />
      </button>
      <h1 className="text-lg font-semibold text-gray-200">Prompter</h1>
      <p className="text-sm text-gray-500">
        Press{" "}
        <kbd className="px-2 py-0.5 bg-gray-800 border border-gray-700 rounded text-xs text-gray-300">
          Cmd+Shift+P
        </kbd>{" "}
        to start
      </p>
    </div>
  );
}
