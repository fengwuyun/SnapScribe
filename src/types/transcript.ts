/** Unified transcript structures mirrored one-to-one from src-tauri/src/model.rs. */
export interface TranscriptSegment {
  id: string;
  /** Seconds from the beginning of the source media. */
  start: number;
  end: number;
  text: string;
  speakerId?: string;
}

export interface TranscriptSpeaker {
  id: string;
  name: string;
  colorIndex: number;
}

export type DiarizationStatus = "disabled" | "provisional" | "processing" | "completed" | "failed";

export interface DiarizationState {
  enabled: boolean;
  status: DiarizationStatus;
  error?: string;
}

export interface TranscriptResult {
  fileName: string;
  duration: number;
  language?: string;
  segments: TranscriptSegment[];
}

export interface MediaInfo {
  fileName: string;
  sizeBytes: number;
  durationSecs: number;
  container: string;
}

export interface HistoryEntry {
  fileName: string;
  sizeBytes: number;
  modifiedMs: number;
}

export type TranscribeStage = "preparing" | "splitting" | "transcribing" | "diarizing" | "reconcilingSpeakers" | "paused";

export interface ProgressEvent {
  projectId: string;
  jobId: string;
  stage: TranscribeStage;
  percent: number;
  processedSeconds: number;
  totalSeconds: number;
  segmentIndex: number;
  segmentCount: number;
}

export interface SegmentsEvent {
  projectId: string;
  jobId: string;
  segments: TranscriptSegment[];
  speakers: TranscriptSpeaker[];
}

export interface RecordingLevelEvent {
  recordingId: string;
  projectId: string;
  level: number;
}
