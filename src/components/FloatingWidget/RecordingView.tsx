import { useRecordingStore } from "../../stores/recordingStore";

export function RecordingView() {
  const { state, downloadProgress, stopRecording } = useRecordingStore();

  return (
    <div className="flex flex-col items-center gap-4 p-6">
      {state === "downloading" ? (
        <>
          <span className="text-sm text-gray-300">Downloading Whisper model...</span>
          <div className="w-full bg-gray-800 rounded-full h-2">
            <div
              className="bg-blue-500 h-2 rounded-full transition-all duration-300"
              style={{ width: `${downloadProgress}%` }}
            />
          </div>
          <span className="text-xs text-gray-500">{downloadProgress}%</span>
        </>
      ) : (
        <>
          <div className="flex items-center gap-3">
            <div className="w-3 h-3 bg-red-500 rounded-full animate-recording" />
            <span className="text-sm text-gray-300">
              {state === "recording" && "Recording... Speak your intent"}
              {state === "transcribing" && "Transcribing..."}
              {state === "generating" && "Generating prompt..."}
            </span>
          </div>
          {state === "recording" && (
            <button
              onClick={stopRecording}
              className="px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg text-sm transition-colors"
            >
              Stop Recording
            </button>
          )}
        </>
      )}
    </div>
  );
}
