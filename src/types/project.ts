import type { TranscriptSegment } from "@/types/transcript";

export type ProjectStatus = "transcribing" | "completed" | "canceled" | "failed";
export type MediaOrigin = "imported" | "recorded";
export type MediaStorage = "reference" | "managedCopy" | "recording";
export type ProjectSort = "recentlyUpdated" | "createdAt" | "name" | "duration";

export interface MediaReference {
  origin: MediaOrigin;
  storage: MediaStorage;
  path: string | null;
  originalFileName: string;
  sizeBytes: number;
  container: string;
}

export interface TranscriptionProject {
  schemaVersion: number;
  id: string;
  name: string;
  createdAt: string;
  updatedAt: string;
  status: ProjectStatus;
  durationSecs: number;
  media: MediaReference;
  transcriptRevision: number;
  summaryRevision?: number;
  error?: string;
}

export interface TranscriptDocument {
  schemaVersion: number;
  projectId: string;
  revision: number;
  segments: TranscriptSegment[];
}

export interface ProjectDetail {
  project: TranscriptionProject;
  transcript: TranscriptDocument;
  summary: AISummary | null;
  mediaAvailable: boolean;
}

export interface AISummary {
  schemaVersion: number;
  projectId: string;
  sourceTranscriptRevision: number;
  generatedAt: string;
  summary: string;
  keyPoints: string[];
  actionItems: string[];
  model: string;
}

export interface ProjectListItem {
  id: string;
  name: string;
  createdAt: string;
  updatedAt: string;
  status: ProjectStatus;
  durationSecs: number;
  media: MediaReference;
  mediaAvailable: boolean;
}

export interface StartProjectResult {
  projectId: string;
  jobId: string;
}

export interface RecordingStartResult {
  recordingId: string;
  projectId: string;
  jobId: string;
}

export interface TranscriptionJobStatus {
  projectId: string;
  jobId: string;
  running: boolean;
  paused: boolean;
}

export interface AppSettings {
  schemaVersion: number;
  dataRoot: string;
  importStrategy: "reference" | "copy";
  ai: {
    serviceType: "openai-compatible";
    baseUrl: string;
    model: string;
    hasApiKey: boolean;
  };
  apiKey?: string;
  clearApiKey?: boolean;
}

export interface ConnectionResult {
  ok: boolean;
  message: string;
}

export interface AboutInfo {
  version: string;
  buildTime: string;
  repositoryUrl: string;
}
