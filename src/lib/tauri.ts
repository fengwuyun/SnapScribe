import { invoke } from "@tauri-apps/api/core";
import { listen, type EventCallback } from "@tauri-apps/api/event";
import type {
  HistoryEntry,
  MediaInfo,
  ProgressEvent,
  RecordingLevelEvent,
  SegmentsEvent,
  TranscriptSegment,
} from "@/types/transcript";
import type {
  AISummary,
  AIModelConfig,
  AIModelDraft,
  AIModelStatus,
  AIServiceConfig,
  AboutInfo,
  AppSettings,
  ConnectionResult,
  ProjectDetail,
  ProjectListItem,
  ProjectSort,
  StartProjectResult,
  RecordingStartResult,
  TranscriptDocument,
  TranscriptionProject,
  TranscriptionJobStatus,
} from "@/types/project";

/**
 * The single exit point for all Tauri IPC calls. Components never import
 * `@tauri-apps/*` directly (DEVELOPMENT_SPEC architecture rule).
 */
export function selectFile(): Promise<string | null> {
  return invoke("select_file");
}

export function selectDirectory(): Promise<string | null> {
  return invoke("select_directory");
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

export function transcriptionJobStatus(projectId: string): Promise<TranscriptionJobStatus | null> {
  return invoke("transcription_job_status", { projectId });
}

export function setTranscriptionPaused(projectId: string, paused: boolean): Promise<TranscriptionJobStatus> {
  return invoke("set_transcription_paused", { projectId, paused });
}

export function cancelProjectTranscription(projectId: string): Promise<void> {
  return invoke("cancel_project_transcription", { projectId });
}

export function createProjectFromMedia(path: string): Promise<StartProjectResult> {
  return invoke("project_create_from_media", { path });
}

export function projectList(query = "", sort: ProjectSort = "recentlyUpdated"): Promise<ProjectListItem[]> {
  return invoke("project_list", { query, sort });
}

export function projectGet(projectId: string): Promise<ProjectDetail> {
  return invoke("project_get", { projectId });
}

export function projectDelete(projectId: string): Promise<void> {
  return invoke("project_delete", { projectId });
}

export function projectRename(projectId: string, name: string): Promise<TranscriptionProject> {
  return invoke("project_rename", { projectId, name });
}

export function projectSaveTranscript(
  projectId: string,
  segments: TranscriptSegment[],
): Promise<TranscriptDocument> {
  return invoke("project_save_transcript", { projectId, segments });
}

export function projectRelinkMedia(projectId: string, path: string): Promise<TranscriptionProject> {
  return invoke("project_relink_media", { projectId, path });
}

export function projectDeleteManagedMedia(projectId: string): Promise<TranscriptionProject> {
  return invoke("project_delete_managed_media", { projectId });
}

export function projectOpenMediaLocation(projectId: string): Promise<void> {
  return invoke("project_open_media_location", { projectId });
}

export function projectExport(
  projectId: string,
  format: "txt" | "srt" | "summary",
  path: string,
): Promise<void> {
  return invoke("project_export", { projectId, format, path });
}

export function settingsGet(): Promise<AppSettings> {
  return invoke("settings_get");
}

export function aboutGet(): Promise<AboutInfo> {
  return invoke("about_get");
}

export function openRepository(): Promise<void> {
  return invoke("open_repository");
}

export function settingsSave(settings: AppSettings): Promise<AppSettings> {
  return invoke("settings_save", { settings });
}

export function aiTestConnection(settings: AppSettings): Promise<ConnectionResult> {
  return invoke("ai_test_connection", { settings });
}

export function aiGenerateSummary(projectId: string): Promise<AISummary> {
  return invoke("ai_generate_summary", { projectId });
}

export function aiServiceGet(): Promise<AIServiceConfig> {
  return invoke("ai_service_get");
}

export function aiInstructionSave(instruction: string): Promise<AIServiceConfig> {
  return invoke("ai_instruction_save", { instruction });
}

export function aiModelCreate(draft: AIModelDraft): Promise<AIModelConfig> {
  return invoke("ai_model_create", { draft });
}

export function aiModelUpdate(modelId: string, draft: AIModelDraft): Promise<AIModelConfig> {
  return invoke("ai_model_update", { modelId, draft });
}

export function aiModelDelete(modelId: string): Promise<void> {
  return invoke("ai_model_delete", { modelId });
}

export function aiModelReorder(orderedIds: string[]): Promise<AIServiceConfig> {
  return invoke("ai_model_reorder", { orderedIds });
}

export function aiModelSetEnabled(modelId: string, enabled: boolean): Promise<AIModelConfig> {
  return invoke("ai_model_set_enabled", { modelId, enabled });
}

export function aiModelTest(draft: AIModelDraft): Promise<AIModelStatus> {
  return invoke("ai_model_test", { draft });
}

export function recordingStart(): Promise<RecordingStartResult> {
  return invoke("recording_start");
}

export function recordingStop(recordingId: string): Promise<StartProjectResult> {
  return invoke("recording_stop", { recordingId });
}

export function recordingSetPaused(recordingId: string, paused: boolean): Promise<void> {
  return invoke("recording_set_paused", { recordingId, paused });
}

export function recordingCancel(recordingId: string): Promise<void> {
  return invoke("recording_cancel", { recordingId });
}

export function listenProgress(onEvent: EventCallback<ProgressEvent>) {
  return listen("transcript://progress", onEvent);
}

export function listenSegments(onEvent: EventCallback<SegmentsEvent>) {
  return listen("transcript://segments", onEvent);
}

export function listenCompleted<T>(
  onEvent: EventCallback<{ projectId: string; jobId: string; result: T }>,
) {
  return listen("transcript://completed", onEvent);
}

export function listenFailed(onEvent: EventCallback<{ projectId: string; jobId: string; message: string }>) {
  return listen("transcript://failed", onEvent);
}

export function listenCanceled(onEvent: EventCallback<{ projectId: string; jobId: string }>) {
  return listen("transcript://canceled", onEvent);
}

export function listenRecordingLevel(onEvent: EventCallback<RecordingLevelEvent>) {
  return listen("recording://level", onEvent);
}

/** JSON form expected by the export commands. */
export function segmentsToJson(segments: TranscriptSegment[]): string {
  return JSON.stringify(segments);
}
