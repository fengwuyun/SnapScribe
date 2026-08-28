# SnapScribe Desktop Product Redesign

**Status:** Approved and implemented; verification in progress  
**Date:** 2026-08-27  
**Baseline:** `main@58c8727`

Implementation note: development was performed directly on the pulled baseline without committing or pushing. At the user's request, functional implementation preceded the concentrated test pass.

## 1. Goal

Turn the current single-page local transcription MVP into a usable Windows desktop application for local audio/video and microphone recording transcription. The product must stay simple, fast, and local-first: imported media is referenced by default, transcripts and saved AI summaries remain available without the media, and the only top-level pages are Home, Library, and Settings.

This design supersedes the current product boundaries in `DEVELOPMENT_SPEC.md`, `UI_DESIGN_SPEC.md`, and `AGENTS.md` where they prohibit multi-page navigation, recording, project storage, and optional online AI summarization. It does not supersede the existing local ASR pipeline contract, Rust/frontend responsibility split, supported media formats, packaging model, or visual design tokens unless this document says so explicitly.

## 2. Confirmed Product Decisions

1. Local transcription always runs offline through the existing bundled FFmpeg and SenseVoice runtime.
2. Network access occurs only when the user explicitly tests an AI connection or generates an AI summary.
3. “Start recording” records the default microphone only. System audio and mixed recording are out of scope.
4. Library search covers project names and transcript text. Tags, folders, favorites, and batch management are out of scope.
5. Changing the data location migrates existing projects. Migration failure rolls back to the old location and settings.
6. Existing history TXT files are imported as projects without media while the original TXT files remain untouched.
7. Persistent storage uses one directory per project with JSON documents. SQLite is not introduced.
8. Settings are persisted only after the user presses “Save settings.” Testing an AI connection never saves the draft automatically.
9. Imported files start transcription automatically after validation. No confirmation screen or additional start action is inserted.
10. No automated Git push is performed during development.

## 3. Scope

### 3.1 Included

- Home page with a large drag-and-drop/click upload area, Start Recording action, and recent projects.
- Library page for transcription projects with search, sorting, open, rename, delete, export, and open-original-location actions.
- Project detail view with Transcript and AI Summary tabs.
- Timestamped transcript playback navigation, editing, search, and TXT/SRT export.
- Persistent optional AI summaries with summary, key points, and action items.
- Missing-media detection and relinking.
- Reference and managed-copy import strategies.
- Managed microphone recordings that can be deleted without deleting transcript or summary data.
- Explicitly saved storage and OpenAI Compatible settings.
- Migration of existing TXT history into media-less projects.

### 3.2 Excluded

- Authentication, cloud sync, collaboration, team workspaces, or user accounts.
- System-audio recording, mixed recording, live transcription while recording, and audio-device selection.
- Tags, folders, favorites, bulk operations, multiple library views, or dashboard metrics.
- Speaker diarization, translation, AI chat, custom AI templates, or automatic summary generation.
- Concurrent transcription jobs. The existing single-active-job rule remains.
- SQLite, background sync, model management, auto-update, and runtime model downloads.

## 4. Architecture

### 4.1 Responsibility Split

React owns presentation, navigation state, form drafts, local view filtering, player state, and Tauri event subscriptions. Rust owns filesystem access, project persistence, media copying and validation, storage migration, recording, FFmpeg/ASR subprocesses, cancellation, export, AI HTTP calls, and secret storage.

Components never import Tauri APIs directly. `src/lib/tauri.ts` remains the single frontend IPC boundary. ASR stdout continues to be parsed only in Rust, which emits normalized `TranscriptSegment` values.

### 4.2 Preserved Transcription Contract

The current pipeline remains:

```text
media path
  -> ffprobe validation
  -> FFmpeg converts to 16 kHz mono PCM and creates fixed 60-second segments
  -> Rust transcribes segments strictly sequentially with SenseVoice
  -> each completed segment is emitted immediately
  -> progress is monotonic
  -> cancel kills the active child process and cleans temporary files
```

The pipeline is extended with a `projectId`. Incremental segments and terminal status are written to that project and emitted with both `jobId` and `projectId`. It is not replaced with parallel, whole-file, or online transcription.

### 4.3 Navigation

The desktop application uses a small internal route state instead of adding a routing dependency:

```ts
type AppRoute =
  | { page: "home" }
  | { page: "library" }
  | { page: "settings" }
  | { page: "project"; projectId: string; from: "home" | "library" };
```

Home, Library, and Settings are the only sidebar entries. Project detail is a secondary view opened from Home or Library and returns to its originating page. Browser-style deep linking is not required for a single-window desktop app.

## 5. Persistent Storage

### 5.1 Layout

The pointer to the current data root stays in the fixed Tauri app config directory so the application can locate project data after restart:

```text
%APPDATA%/com.snapscribe.app/
├─ settings.json
└─ legacy-import.json

<dataRoot>/
├─ projects/
│  └─ <projectId>/
│     ├─ project.json
│     ├─ transcript.json
│     ├─ summary.json       optional
│     └─ media/             optional
│        └─ <safe-file-name>
└─ temp/
```

The default data root is `%APPDATA%/com.snapscribe.app/data`. `settings.json` stores non-secret settings and the data root. `legacy-import.json` records which old TXT paths have already been converted so the import is idempotent.

### 5.2 Write Safety

JSON documents are written to a unique temporary file in the destination directory, flushed, and atomically renamed over the destination. An interrupted write must leave either the previous valid document or the new valid document. Project deletion removes exactly `projects/<validated-project-id>` and never accepts arbitrary paths from the frontend.

Project IDs are UUIDs created in Rust. All project commands resolve paths from a validated ID under the configured data root; frontend-provided filesystem paths are used only by explicit file selection, relinking, export, and storage-location commands.

### 5.3 Data Models

```ts
type ProjectStatus = "transcribing" | "completed" | "failed";
type MediaOrigin = "imported" | "recorded";
type MediaStorage = "reference" | "managedCopy" | "recording";

interface TranscriptionProject {
  schemaVersion: 1;
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

interface MediaReference {
  origin: MediaOrigin;
  storage: MediaStorage;
  path: string | null;
  originalFileName: string;
  sizeBytes: number;
  container: string;
}

interface TranscriptDocument {
  schemaVersion: 1;
  projectId: string;
  revision: number;
  segments: TranscriptSegment[];
}

interface TranscriptSegment {
  id: string;
  start: number;
  end: number;
  text: string;
}

interface AISummary {
  schemaVersion: 1;
  projectId: string;
  sourceTranscriptRevision: number;
  generatedAt: string;
  summary: string;
  keyPoints: string[];
  actionItems: string[];
  model: string;
}

interface AppSettings {
  schemaVersion: 1;
  dataRoot: string;
  importStrategy: "reference" | "copy";
  ai: {
    serviceType: "openai-compatible";
    baseUrl: string;
    model: string;
    hasApiKey: boolean;
  };
}
```

Timestamps use UTC RFC 3339 strings. Project names are user-facing and may differ from original media filenames. Transcript revisions increment on every successful transcript edit. A saved summary is stale when `sourceTranscriptRevision !== transcriptRevision`.

### 5.4 Media Lifecycle

For `reference`, the selected source path is stored and the file is never copied. For `managedCopy`, the source is copied into the project’s `media` directory before transcription. For `recording`, microphone output is written directly to the project’s `media` directory.

Media availability is computed when a project is loaded or listed; it is not treated as durable truth in JSON. If the path is absent or no longer a readable file, playback and open-location actions are disabled while transcript and summary remain available.

Relinking validates that the new path is a supported readable media file, updates `media.path`, `originalFileName`, `sizeBytes`, `container`, and `durationSecs`, and preserves transcript and summary. It does not automatically retranscribe.

Deleting managed media is allowed only for `recording` or `managedCopy`. It deletes the file, sets `media.path` to `null`, and leaves project documents unchanged. A referenced external file is never deleted by SnapScribe.

## 6. Pages and Components

### 6.1 App Shell

`AppShell` contains the SnapScribe identity, three-item sidebar, main content region, and global toast/dialog host. The sidebar is compact and uses the existing light background, primary blue-purple selection state, rounded controls, and Lucide icons.

The current design tokens remain authoritative: white and light gray surfaces, `#4353FF` primary, existing radius and spacing scales, restrained shadows, no gradient, no glass, and no dashboard styling.

### 6.2 Home

Home contains:

- A top-right Start Recording action.
- A large `MediaDropZone` supporting drag-and-drop and click selection.
- A recent-project list with project name, duration, source, update time, status, and open action.

On import:

```text
select/drop path
  -> validate supported file and ffprobe metadata
  -> create project using saved import strategy
  -> start transcription immediately
  -> navigate to project detail
```

There is no ready/confirmation screen and no separate Start Transcription button. If another transcription is active, import fails with a clear message and does not create an abandoned project.

### 6.3 Library

The Library manages projects, not media files. The default order is recently updated. Sorting options are recently updated, creation time, name, and duration. Search is case-insensitive over project name and transcript text; it does not search tags because tags do not exist.

Each row supports open, rename, delete project, export TXT, export SRT, and open original location. Open original location is disabled for unavailable media. Project deletion requires one destructive confirmation because it removes transcript and summary data; ordinary opening, importing, exporting, and relinking add no confirmation step.

Initial search performs a Rust-side scan of project metadata and transcript JSON. This is acceptable for a personal local library and avoids a database. Search results return compact list items rather than full transcripts.

### 6.4 Project Detail

Project detail has a back action, editable project name, export action, overflow menu, and two tabs: Transcript and AI Summary.

The Transcript tab includes:

- Existing timestamped segment presentation and active playback highlighting.
- Click-to-seek when media is available.
- Transcript search with match highlighting and previous/next navigation.
- Segment text editing while preserving `id`, `start`, and `end`.
- Explicit transcript save after edits.
- Fixed bottom media player.
- Existing transcription progress and incremental segments while a job is active.

If media is unavailable, the player is replaced by a banner explaining that transcript and summary remain available, with a Relink Original File action. Recording and managed-copy projects additionally expose Delete Audio File while media exists.

### 6.5 AI Summary

The AI Summary tab has exactly three result sections: Summary, Key Points, and Action Items. The application sends current saved transcript text, never media bytes, to the configured OpenAI Compatible service only after the user presses Generate AI Summary.

Generation requires a completed non-empty transcript and saved AI settings. A successful structured result is persisted in `summary.json`. Existing summary data is readable without a network connection. If the transcript revision changes, the UI shows that the summary may be outdated and offers explicit regeneration; it does not regenerate automatically.

### 6.6 Settings

Settings contains only File Storage and AI Service.

File Storage fields:

- Transcript data location.
- Import media strategy: Reference original file (default/recommended) or Copy to SnapScribe.
- Static explanation that SnapScribe recordings are saved by default and may be deleted later without affecting transcript or summary.

AI Service fields:

- Service type fixed to OpenAI Compatible.
- API Base URL.
- API Key.
- Model name.
- Test Connection.
- Connection status.

The page loads persisted values into a draft. Editing the draft has no side effects. Test Connection uses the draft in memory and updates only transient connection status. Save Settings validates and persists the complete draft. Navigating away discards unsaved changes after a small inline unsaved-state warning; it does not auto-save.

Changing the data root during Save Settings invokes migration before writing the new root to fixed settings. Migration copies projects to a staging directory at the new location, validates document counts and JSON readability, atomically promotes the staging directory, and then updates settings. Failure removes staging data and keeps the old root active. The old root is not automatically deleted, providing a recoverable rollback copy.

## 7. Recording

Recording is implemented in Rust using the Windows default input device. The recording state machine is:

```ts
type RecordingState =
  | { phase: "idle" }
  | { phase: "recording"; recordingId: string; elapsedSecs: number }
  | { phase: "stopping"; recordingId: string }
  | { phase: "error"; message: string };
```

Start Recording creates a project and a managed WAV target, opens the default microphone, and streams PCM samples to disk. Stop finalizes the WAV header, probes the media, starts the existing transcription pipeline, and navigates to project detail. Cancel closes and deletes the incomplete recording and project. Recording duration and optional input level are emitted as UI events; audio samples are never sent to React.

If microphone permission is denied, the device is unavailable, or file writing fails, the application stops cleanly, removes incomplete managed data, and displays an actionable error. No recording device selector is added in this version.

## 8. AI Service and Secret Handling

Rust performs all AI HTTP requests. The OpenAI Compatible endpoint uses the configured base URL, model, API key, and a chat-completions-compatible request. The test action sends a minimal request that validates authentication, base URL, and model together; relying only on a `/models` endpoint is avoided because compatible services vary.

The API key is stored with Windows Credential Manager or DPAPI-backed local encryption and is never written to `settings.json`, project files, logs, exports, or error strings. `settings_get` returns only `hasApiKey`. Leaving the API Key draft blank preserves an existing key; an explicit Clear API Key action removes it.

The summary prompt requests one JSON object with `summary`, `keyPoints`, and `actionItems`. Rust validates the response shape, rejects empty or wrong-typed fields, and reports a readable error without overwriting an existing saved summary.

## 9. IPC and Events

The frontend IPC surface is organized around projects:

```text
project_create_from_media(path) -> { projectId, jobId }
project_list(query, sort) -> ProjectListItem[]
project_get(projectId) -> ProjectDetail
project_rename(projectId, name)
project_delete(projectId)
project_save_transcript(projectId, segments)
project_relink_media(projectId, path)
project_delete_managed_media(projectId)
project_open_media_location(projectId)
project_export(projectId, format, targetPath)

recording_start() -> { recordingId, projectId }
recording_stop(recordingId) -> { projectId, jobId }
recording_cancel(recordingId)

settings_get() -> AppSettings
settings_save(draft)
ai_test_connection(draft) -> ConnectionResult
ai_generate_summary(projectId) -> AISummary
```

Existing transcription event names remain, but every payload contains `projectId`; job-scoped events also contain `jobId`:

```ts
interface ProjectProgressEvent extends ProgressEvent { projectId: string }
interface ProjectSegmentsEvent extends SegmentsEvent { projectId: string }
interface ProjectCompletedEvent { projectId: string; result: TranscriptResult }
interface ProjectFailedEvent { projectId: string; jobId: string; message: string }
```

The frontend discards events that do not match the active project and job. The project store still persists terminal status, so reopening the Library does not depend on an in-memory event having been observed.

## 10. Error Handling

- Unsupported, unreadable, or missing imports fail before a project is created.
- A managed-copy failure removes the incomplete project and copied partial file.
- Transcription failure retains the project with status `failed`, any safely persisted segments, and a retry action when media remains available.
- Cancel returns the project to a retryable failed state with a user-readable cancellation message; it does not leave status `transcribing` after restart.
- On startup, projects left as `transcribing` without a live job are marked `failed` with an interrupted-session message.
- Corrupt project JSON is skipped from normal results and reported as a recoverable library error; other projects continue loading.
- Relinking failure leaves the old media reference unchanged.
- Summary generation failure preserves the last saved summary.
- Project deletion and managed-media deletion validate both project ID and resolved descendant path before removing files.
- No credentials or transcript bodies are written to logs.

## 11. Legacy TXT Import

On first startup after the project-store upgrade, Rust scans the current `app_data/transcripts` directory. Each `.txt` not recorded in `legacy-import.json` becomes one completed project with:

- Project name from the TXT filename without the timestamp suffix when recognizable.
- `media.path = null`.
- `media.origin = imported` and `media.storage = reference`.
- One transcript segment starting at `0` with the TXT content, because historic TXT does not retain structured timestamps.
- Duration `0` and media unavailable.

The original TXT is not moved, renamed, or deleted. Successful imports are recorded by canonical path plus size and modified timestamp so repeated startup is idempotent.

## 12. Testing and Verification

### 12.1 Rust Tests

- Project ID/path validation and traversal rejection.
- Atomic JSON write and recovery from interrupted temporary files.
- Reference, managed-copy, recording, relink, and managed-media deletion rules.
- Project list ordering and name/transcript search.
- Transcript revision and summary stale detection.
- Settings validation and data-root migration success/failure rollback.
- Legacy TXT import idempotency and no-source-file mutation.
- AI response validation and secret redaction.
- Recording state transitions with an injectable recorder abstraction; Windows default-device behavior is covered by manual smoke testing.

### 12.2 Frontend Tests

- Navigation among three top-level pages and project detail return behavior.
- Import automatically starts transcription and opens detail.
- Recent-project and Library empty/loading/error states.
- Library search, sort, and disabled media actions.
- Transcript edit draft, explicit save, search navigation, and stale-summary indicator.
- Settings draft, Test Connection without save, Save Settings, and unsaved-state behavior.
- Recording reducer transitions and cleanup on errors.
- Stale transcription events are ignored by both project ID and job ID.

### 12.3 End-to-End Windows Smoke Test

```text
install/open
  -> import by reference -> automatic local transcription -> edit -> export
  -> move source -> reopen text -> relink -> playback restored
  -> import by copy -> delete managed media -> text remains
  -> record microphone -> stop -> automatic transcription -> delete recording -> text remains
  -> configure and explicitly save AI service -> test -> generate summary
  -> edit transcript -> summary marked stale -> regenerate
  -> change data root -> projects migrate -> restart -> projects still visible
  -> migrate old TXT -> media-less project visible -> original TXT remains
```

Fresh verification before completion includes `npm test`, `npm run typecheck`, `npm run build`, `cargo test`, `cargo check`, and `npm run tauri build`, followed by inspection of the NSIS output and a clean-worktree diff review. Runtime binaries and models must be downloaded and verified before Rust/Tauri package checks because the repository intentionally does not track them.

## 13. Delivery Sequence

1. Update governing product, UI, architecture, and decision documents to match this approved design.
2. Build the project persistence kernel and pipeline integration.
3. Build the three-page shell, automatic import flow, and Library.
4. Complete project detail editing, media-loss behavior, relinking, and exports.
5. Add microphone recording.
6. Add explicitly saved settings, storage migration, secure AI configuration, and AI summaries.
7. Import legacy TXT history, run full regression, and package the Windows application.

Each stage must leave a testable application state. Existing local transcription behavior is regression-tested before and after each pipeline-affecting change. Development may create local commits following the repository’s Conventional Commit rule, but must not push automatically.
