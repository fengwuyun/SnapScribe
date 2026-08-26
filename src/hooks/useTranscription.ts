import { useCallback, useEffect, useReducer, useRef } from "react";
import type { MediaInfo, ProgressEvent, TranscriptResult, TranscriptSegment } from "@/types/transcript";
import {
  cancelTranscription,
  getMediaInfo,
  listenCompleted,
  listenFailed,
  listenProgress,
  listenSegments,
  startTranscription,
} from "@/lib/tauri";

/**
 * Application state machine (DEVELOPMENT_SPEC §6). Exactly five phases; the
 * split/transcribe stages live inside the progress payload, never here.
 */
export type AppPhase = "empty" | "ready" | "transcribing" | "completed" | "error";

export interface TranscriptionState {
  phase: AppPhase;
  file: MediaInfo | null;
  filePath: string | null;
  jobId: string | null;
  /** Segments received so far — appended incrementally as the backend emits them. */
  segments: TranscriptSegment[];
  progress: ProgressEvent | null;
  result: TranscriptResult | null;
  error: string | null;
}

export type TranscriptionAction =
  | { type: "fileSelected"; path: string; info: MediaInfo }
  | { type: "fileCleared" }
  | { type: "started"; jobId: string }
  | { type: "progress"; jobId: string; event: ProgressEvent }
  | { type: "segments"; jobId: string; segments: TranscriptSegment[] }
  | { type: "completed"; result: TranscriptResult }
  | { type: "failed"; message: string }
  | { type: "canceled" };

export const initialTranscriptionState: TranscriptionState = {
  phase: "empty",
  file: null,
  filePath: null,
  jobId: null,
  segments: [],
  progress: null,
  result: null,
  error: null,
};

/** Events from a superseded job are dropped instead of corrupting the current one.
 *  A null jobId during `transcribing` adopts the arriving event's id: backend
 *  events can race ahead of the `started` dispatch. */
function matchJob(state: TranscriptionState, jobId: string): boolean {
  return state.jobId === null || state.jobId === jobId;
}

export function transcriptionReducer(
  state: TranscriptionState,
  action: TranscriptionAction,
): TranscriptionState {
  switch (action.type) {
    case "fileSelected":
      return {
        ...initialTranscriptionState,
        phase: "ready",
        filePath: action.path,
        file: action.info,
      };
    case "fileCleared":
      return { ...initialTranscriptionState };
    case "started":
      return {
        ...state,
        phase: "transcribing",
        jobId: action.jobId,
        // Cancel keeps the selected file; a fresh start always begins empty.
        segments: [],
        progress: null,
        result: null,
        error: null,
      };
    case "progress":
      if (state.phase !== "transcribing" || !matchJob(state, action.jobId)) return state;
      return { ...state, jobId: state.jobId ?? action.jobId, progress: action.event };
    case "segments":
      if (state.phase !== "transcribing" || !matchJob(state, action.jobId)) return state;
      return {
        ...state,
        jobId: state.jobId ?? action.jobId,
        segments: [...state.segments, ...action.segments],
      };
    case "completed":
      return {
        ...state,
        phase: "completed",
        // The final result is authoritative over incrementally received segments.
        segments: action.result.segments,
        result: action.result,
      };
    case "failed":
      if (state.phase === "transcribing" || state.phase === "ready") {
        return { ...state, phase: "error", error: action.message };
      }
      return { ...state, error: action.message };
    case "canceled":
      if (state.phase !== "transcribing") return state;
      return {
        ...state,
        phase: "ready",
        jobId: null,
        segments: [],
        progress: null,
      };
    default:
      return assertNeverAction(action);
  }
}

function assertNeverAction(value: never): never {
  throw new Error(`未知的转写状态动作：${JSON.stringify(value)}`);
}

/**
 * React binding over the transcription state machine: subscribes to the four
 * backend event channels once and exposes the user-facing actions.
 */
export function useTranscription() {
  const [state, dispatch] = useReducer(transcriptionReducer, initialTranscriptionState);
  const stateRef = useRef(state);
  stateRef.current = state;

  useEffect(() => {
    let disposed = false;
    const cleanups: (() => void)[] = [];
    void (async () => {
      const subscriptions = await Promise.all([
        listenProgress((e) =>
          dispatch({ type: "progress", jobId: e.payload.jobId, event: e.payload }),
        ),
        listenSegments((e) =>
          dispatch({ type: "segments", jobId: e.payload.jobId, segments: e.payload.segments }),
        ),
        listenCompleted<TranscriptResult>((e) => dispatch({ type: "completed", result: e.payload.result })),
        listenFailed((e) => dispatch({ type: "failed", message: e.payload.message })),
      ]);
      if (disposed) {
        for (const unlisten of subscriptions) unlisten();
      } else {
        cleanups.push(...subscriptions);
      }
    })();
    return () => {
      disposed = true;
      for (const unlisten of cleanups.splice(0)) unlisten();
    };
  }, []);

  const loadFile = useCallback(async (path: string) => {
    const info = await getMediaInfo(path);
    dispatch({ type: "fileSelected", path, info });
  }, []);

  const clearFile = useCallback(() => dispatch({ type: "fileCleared" }), []);

  const start = useCallback(async () => {
    const current = stateRef.current;
    if (current.phase !== "ready" || !current.filePath) return;
    const jobId = await startTranscription(current.filePath);
    // The reducer adopts events arriving before this dispatch via matchJob.
    dispatch({ type: "started", jobId });
  }, []);

  const cancel = useCallback(async () => {
    const current = stateRef.current;
    if (current.phase !== "transcribing") return;
    try {
      if (current.jobId) await cancelTranscription(current.jobId);
    } finally {
      dispatch({ type: "canceled" });
    }
  }, []);

  return { state, loadFile, clearFile, start, cancel };
}
