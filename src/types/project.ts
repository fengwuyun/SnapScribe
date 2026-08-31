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
  actionItems: ActionItem[];
  model: string;
  modelConfigId?: string;
  modelName?: string;
}

export interface ActionItem {
  id: string;
  text: string;
  completed: boolean;
}

export type AIProtocol = "openai-chat";
export type AIEndpointMode = "auto" | "full-url" | "custom-path";
export type AIAuthType = "bearer" | "x-api-key" | "custom-header" | "none";
export type AIModelStatusKind = "untested" | "available" | "timeout" | "auth-failed" | "rate-limited" | "service-error" | "config-error" | "invalid-response";

export interface AIModelStatus {
  status: AIModelStatusKind;
  message?: string;
  checkedAt?: string;
  responseTimeMs?: number;
}

export interface AIModelConfig {
  id: string;
  presetKey?: "glm-4.7-flash";
  name: string;
  protocol: AIProtocol;
  baseUrl: string;
  modelId: string;
  endpointMode: AIEndpointMode;
  customPath?: string;
  authType: AIAuthType;
  authHeader?: string;
  timeoutSecs: number;
  enabled: boolean;
  order: number;
  hasApiKey: boolean;
  lastStatus: AIModelStatus;
}

export interface AIModelDraft {
  id?: string;
  name: string;
  protocol: AIProtocol;
  baseUrl: string;
  modelId: string;
  endpointMode: AIEndpointMode;
  customPath?: string;
  authType: AIAuthType;
  authHeader?: string;
  timeoutSecs: number;
  enabled: boolean;
  apiKey?: string;
  clearApiKey?: boolean;
}

export interface AIServiceConfig {
  schemaVersion: number;
  customInstruction: string;
  models: AIModelConfig[];
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
  importStrategy: "reference" | "copy" | "move";
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
