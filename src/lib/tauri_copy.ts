import { invoke } from "@tauri-apps/api/core";
import type { EventCallback } from "@tauri-apps/api/event";
import type {
  HistoryEntry,
  MediaInfo,
  ProgressEvent,
  SegmentsEvent,
  TranscriptSegment,
} from "@/types/transcript";

/**
 * The single exit point for all Tauri IPC calls. Components never import
 * `@tauri-apps/*` directly (DEVELOPMENT_SPEC architecture rule).
 */
export const tauri = {
  selectFile: () => invoke<string | null>("select_file"),

  getMediaInfo: (path: string) => invoke<MediaInfo>("get_media_info", { path }),

  startTranscription: (path: string) => invoke<string>("start_transcription", { path }),

  cancelTranscription: (jobId: string) => invoke<void>("cancel_transcription", { jobId }),

  saveFileDialog: (defaultName: string, ext: "txt" | "srt") =>
    invoke<string | null>("save_file_dialog", { defaultName, ext }),

  exportTxt: (segmentsJson: string, path: string) =>
    invoke<void>("export_txt", { segmentsJson, path }),

  exportSrt: (segmentsJson: string, path: string) =>
    invoke<void>("export_srt", { segmentsJson, path }),

  historyList: () => invoke<HistoryEntry[]>("history_list"),

  historyRead: (fileName: string) => invoke<string>("history_read", { fileName }),

  historyRename: (oldName: string, newName: string) =>
    invoke<string>("history_rename", { oldName, newName }),

  historyDelete: (fileName: string) => invoke<void>("history_delete", { fileName }),

  historyDirPath: () => invoke<string>("history_dir_path"),
};

export function listenProgress(onEvent: EventCallback<ProgressEvent>) {
  return import("@tauri-apps/api/event").then((m) => m.listen("transcript://progress", onEvent));
}

export function listenSegments(onEvent: EventCallback<SegmentsEvent>) {
  return import("@tauri-apps/api/event").then((m) => m.listen("transcript://segments", onEvent));
}

export function listenCompleted<T>(onEvent: EventCallback<{ result: T }>) {
  return import("@tauri-apps/api/event").then((m) => m.listen("transcript://completed", onEvent));
}

export function listenFailed(onEvent: EventCallback<{ message: string }>) {
  return import("@tauri-apps/api/event").then((m) => m.listen("transcript://failed", onEvent));
}

/** JSON form expected by the export commands. */
export function segmentsToJson(segments: TranscriptSegment[]): string {
  return JSON.stringify(segments);
}
