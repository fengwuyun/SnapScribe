import { invoke } from "@tauri-apps/api/core";
import { listen, type EventCallback } from "@tauri-apps/api/event";
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
export function selectFile(): Promise<string | null> {
  return invoke("select_file");
}

export function getMediaInfo(path: string): Promise<MediaInfo> {
  return invoke("get_media_info", { path });
}

export function startTranscription(path: string): Promise<string> {
  return invoke("start_transcription", { path });
}

export function cancelTranscription(jobId: string): Promise<void> {
  return invoke("cancel_transcription", { jobId });
}

export function saveFileDialog(defaultName: string, ext: "txt" | "srt"): Promise<string | null> {
  return invoke("save_file_dialog", { defaultName, ext });
}

export function exportTxt(segmentsJson: string, path: string): Promise<void> {
  return invoke("export_txt", { segmentsJson, path });
}

export function exportSrt(segmentsJson: string, path: string): Promise<void> {
  return invoke("export_srt", { segmentsJson, path });
}

export function historyList(): Promise<HistoryEntry[]> {
  return invoke("history_list");
}

export function historyRead(fileName: string): Promise<string> {
  return invoke("history_read", { fileName });
}

export function historyRename(oldName: string, newName: string): Promise<string> {
  return invoke("history_rename", { oldName, newName });
}

export function historyDelete(fileName: string): Promise<void> {
  return invoke("history_delete", { fileName });
}

export function historyDirPath(): Promise<string> {
  return invoke("history_dir_path");
}

export function listenProgress(onEvent: EventCallback<ProgressEvent>) {
  return listen("transcript://progress", onEvent);
}

export function listenSegments(onEvent: EventCallback<SegmentsEvent>) {
  return listen("transcript://segments", onEvent);
}

export function listenCompleted<T>(onEvent: EventCallback<{ result: T }>) {
  return listen("transcript://completed", onEvent);
}

export function listenFailed(onEvent: EventCallback<{ message: string }>) {
  return listen("transcript://failed", onEvent);
}

/** JSON form expected by the export commands. */
export function segmentsToJson(segments: TranscriptSegment[]): string {
  return JSON.stringify(segments);
}
