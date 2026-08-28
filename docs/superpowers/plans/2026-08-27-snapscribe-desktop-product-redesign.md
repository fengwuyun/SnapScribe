# SnapScribe Desktop Product Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Upgrade the existing SnapScribe local transcription MVP into the approved three-page Windows desktop product with persistent transcription projects, missing-media recovery, microphone recording, and optional OpenAI Compatible summaries.

**Architecture:** Preserve the existing Rust-owned FFmpeg/SenseVoice pipeline and React/Tauri boundary. Add a Rust project-store around the pipeline, a lightweight React route state for Home/Library/Settings/Project Detail, and file-backed JSON documents under a configurable data root. AI and microphone access remain Rust-owned.

**Tech Stack:** Tauri 2, React 18, TypeScript 5, Tailwind CSS 3, Vitest, Rust, serde/serde_json, FFmpeg, SenseVoice, Windows microphone capture, OpenAI Compatible HTTP.

**Spec:** `docs/superpowers/specs/2026-08-27-snapscribe-desktop-product-redesign.md`

## Global Constraints

- Work directly in the pulled existing repository as explicitly requested by the user.
- Do not commit and do not push.
- Keep local transcription offline and preserve sequential 60-second segmented ASR.
- Only Home, Library, and Settings are top-level pages; Project Detail is secondary.
- Imported media starts transcription automatically with no confirmation step.
- Default import mode references the original file and never deletes external files.
- AI is invoked only by explicit Test Connection or Generate Summary actions.
- Settings persist only through Save Settings.
- Use existing colors, radius values, spacing, fonts, and Lucide icons.
- Per the user's later instruction, complete the functional implementation first, then add and run the concentrated test suite.

---

### Task 1: Project persistence kernel

**Files:**
- Create: `src-tauri/src/project_store.rs`
- Create: `src-tauri/src/settings.rs`
- Modify: `src-tauri/src/model.rs`
- Modify: `src-tauri/src/main.rs`
- Test: inline `#[cfg(test)]` modules in the new Rust files

**Interfaces:**
- Produces: `ProjectStore::new(data_root: PathBuf)`, `create_project`, `read_project`, `write_project`, `write_transcript`, `read_transcript`, `list_projects`, `rename_project`, `delete_project`.
- Produces: `SettingsStore::load(app_config_dir)`, `SettingsStore::save`, and `AppSettings` with reference import as default.
- Consumes: existing `TranscriptSegment`.

- [ ] **Step 1: Write failing project-store tests**

Add tests that create a temporary data root and assert literal outcomes:

```rust
#[test]
fn creates_a_reference_project_without_copying_media() {
    let created = store.create_project(&source, ImportStrategy::Reference).unwrap();
    assert_eq!(created.media.storage, MediaStorage::Reference);
    assert_eq!(created.media.path.as_deref(), source.to_str());
    assert!(!root.join("projects").join(&created.id).join("media").exists());
}

#[test]
fn transcript_survives_when_referenced_media_is_removed() {
    store.write_transcript(&project.id, &document).unwrap();
    std::fs::remove_file(&source).unwrap();
    let detail = store.read_detail(&project.id).unwrap();
    assert!(!detail.media_available);
    assert_eq!(detail.transcript.segments[0].text, "保留的文本");
}

#[test]
fn rejects_project_id_path_traversal() {
    assert!(store.read_project("../outside").is_err());
}
```

- [ ] **Step 2: Run the focused Rust tests and verify failure because the module/types do not exist**

Run: `cargo test project_store --lib` from `src-tauri` after ensuring resource validation does not block test-only builds.

- [ ] **Step 3: Implement minimal versioned JSON models and atomic writes**

Implement `TranscriptionProject`, `MediaReference`, `TranscriptDocument`, `AISummary`, enums for status/origin/storage, UUID validation, temporary-file write + rename, and exact descendant path checks.

- [ ] **Step 4: Implement settings default/load/save tests and code**

The default must be:

```rust
AppSettings {
    schema_version: 1,
    data_root: app_config_dir.join("data"),
    import_strategy: ImportStrategy::Reference,
    ai: AISettings { service_type: OpenAICompatible, base_url: "".into(), model: "".into(), has_api_key: false },
}
```

- [ ] **Step 5: Run focused tests, full Rust tests, and `cargo fmt --check`**

Expected: all new project/settings tests pass; existing tests stay green once runtime resources are present.

---

### Task 2: Project commands and transcription integration

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/pipeline.rs`
- Modify: `src-tauri/src/model.rs`
- Modify: `src-tauri/src/main.rs`
- Modify: `src/lib/tauri.ts`
- Modify: `src/types/transcript.ts`
- Test: `tests/useTranscription.test.ts` and Rust module tests

**Interfaces:**
- Produces: `project_create_from_media`, `project_list`, `project_get`, `project_rename`, `project_delete`, `project_save_transcript`, `project_relink_media`, `project_delete_managed_media`, `project_open_media_location`, `project_export`.
- Produces events carrying both `projectId` and `jobId`.
- Consumes: Task 1 `ProjectStore` and existing ASR pipeline.

- [ ] **Step 1: Add failing reducer tests for project-scoped events**

```ts
it("ignores segments for another project even when the job id matches", () => {
  const next = transcriptionReducer(activeProjectState, {
    type: "segments",
    projectId: "project-other",
    jobId: "job-1",
    segments: [segment("wrong")],
  });
  expect(next.segments).toEqual([]);
});
```

- [ ] **Step 2: Verify the new test fails because project scope is absent**

Run: `npm test -- tests/useTranscription.test.ts`.

- [ ] **Step 3: Extend event types and pipeline persistence**

Add `project_id` to progress, segment, completed, and failed payloads. Persist incremental transcript state after each completed ASR window and terminal project status on complete/fail/cancel/interrupted startup.

- [ ] **Step 4: Add Rust command tests for reference/copy/relink/delete rules**

Tests must prove that relinking preserves transcript JSON and that deleting managed media sets the path to null while deleting a reference never removes the external file.

- [ ] **Step 5: Implement commands and the TypeScript IPC boundary**

Keep all filesystem behavior in Rust and expose typed wrappers only from `src/lib/tauri.ts`.

- [ ] **Step 6: Run frontend and Rust focused tests**

Expected: stale project/job events are ignored; project operations preserve transcript and summary data.

---

### Task 3: Three-page shell, Home, and Library

**Files:**
- Modify: `src/App.tsx`
- Create: `src/types/project.ts`
- Create: `src/hooks/useAppRoute.ts`
- Create: `src/hooks/useProjects.ts`
- Create: `src/components/AppShell.tsx`
- Create: `src/components/AppSidebar.tsx`
- Create: `src/pages/HomePage.tsx`
- Create: `src/pages/LibraryPage.tsx`
- Create: `src/components/RecentProjects.tsx`
- Create: `src/components/ProjectTable.tsx`
- Modify: `src/components/DropZone.tsx`
- Modify: `src/styles/globals.css`
- Test: `tests/appRoute.test.ts`, `tests/projectList.test.ts`, `tests/autoImport.test.ts`

**Interfaces:**
- Produces: `AppRoute` union and `navigate(route)`.
- Produces: Home import handler that returns `{ projectId, jobId }` then navigates immediately to detail.
- Consumes: Task 2 project IPC wrappers.

- [ ] **Step 1: Add failing route tests**

Assert that top-level navigation exposes exactly `home`, `library`, and `settings`, and that a project route records its origin for Back behavior.

- [ ] **Step 2: Add failing auto-import orchestration test**

Assert literal call order: `createProject(path)` resolves before `navigate({ page: "project", ... })`; no `ready` or manual `start` action exists in the returned flow.

- [ ] **Step 3: Implement route state and AppShell**

Use React state rather than adding a router dependency. Render one selected sidebar item and preserve existing light blue-purple visual tokens.

- [ ] **Step 4: Implement Home and Library**

Home renders Start Recording, large DropZone, and recent projects. Library renders name/transcript search, four sort options, and project row actions. Do not render Tips, Dashboard, tags, folders, favorites, batch tools, or “in progress” dashboard cards.

- [ ] **Step 5: Run frontend tests, typecheck, and build**

Expected: existing DropZone behaviors remain; new navigation/import/list behavior passes.

---

### Task 4: Project Detail transcript workflow

**Files:**
- Create: `src/pages/ProjectDetailPage.tsx`
- Create: `src/components/ProjectHeader.tsx`
- Create: `src/components/TranscriptEditor.tsx`
- Create: `src/components/TranscriptSearch.tsx`
- Create: `src/components/MissingMediaBanner.tsx`
- Modify: `src/components/TranscriptList.tsx`
- Modify: `src/components/AudioPlayer.tsx`
- Modify: `src/hooks/useTranscription.ts`
- Test: `tests/transcriptEditor.test.ts`, `tests/transcriptSearch.test.ts`, `tests/mediaAvailability.test.ts`

**Interfaces:**
- Consumes: `ProjectDetail`, project-scoped transcription events, and project save/relink/delete/export commands.
- Produces: edited segment array preserving IDs/timestamps and incrementing transcript revision only after explicit save.

- [ ] **Step 1: Write failing transcript-edit and search tests**

Tests must prove editing changes only `text`, search matches literal Chinese substrings case-insensitively for Latin text, and next/previous wrap across matches.

- [ ] **Step 2: Implement TranscriptEditor and explicit save**

Keep edits in a draft. Save submits the whole normalized segment array. Cancel restores persisted segments.

- [ ] **Step 3: Write failing missing-media action tests**

Assert the player is not rendered when `mediaAvailable` is false and Relink remains available. Assert Delete Audio appears only for `recording` and `managedCopy` with available media.

- [ ] **Step 4: Implement detail tabs and media lifecycle UI**

Render only Transcript and AI Summary tabs. Reuse timestamp seek and bottom player when media exists.

- [ ] **Step 5: Run focused and full frontend verification**

Expected: edit/search/media-loss behaviors pass without regressing existing player and transcript tests.

---

### Task 5: Microphone recording

**Files:**
- Create: `src-tauri/src/recording.rs`
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/main.rs`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src/lib/tauri.ts`
- Create: `src/hooks/useRecording.ts`
- Create: `src/components/RecordingDialog.tsx`
- Modify: `src/pages/HomePage.tsx`
- Test: Rust recording state tests and `tests/useRecording.test.ts`

**Interfaces:**
- Produces: `recording_start`, `recording_stop`, `recording_cancel`.
- Produces: duration/level events and a finalized recording project passed to the existing pipeline.

- [ ] **Step 1: Write failing Rust recording lifecycle tests against an injected sample source**

Prove start creates a managed project, stop finalizes a valid WAV and returns a project/job pair, cancel removes incomplete project media, and write failure leaves no active recorder.

- [ ] **Step 2: Add the minimal Windows default-input implementation**

Use a focused recorder abstraction around the default microphone and WAV writer. Do not add device selection, system audio, mixing, or live transcription.

- [ ] **Step 3: Write failing frontend reducer tests**

Cover `idle -> recording -> stopping -> idle`, cancel, permission/device error, and ignoring stale recording events.

- [ ] **Step 4: Implement Start Recording UI and automatic transcription on stop**

The Home action opens one compact recording state view. Stop navigates to project detail; cancel returns Home.

- [ ] **Step 5: Run Rust/frontend tests and perform Windows microphone smoke test**

Expected: recorded WAV is playable and can be deleted without removing transcript data.

---

### Task 6: Explicit settings, storage migration, and AI summaries

**Files:**
- Modify: `src-tauri/src/settings.rs`
- Create: `src-tauri/src/ai.rs`
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/main.rs`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src/lib/tauri.ts`
- Create: `src/pages/SettingsPage.tsx`
- Create: `src/components/StorageSettings.tsx`
- Create: `src/components/AIServiceSettings.tsx`
- Create: `src/components/SummaryPanel.tsx`
- Test: Rust settings/AI tests and `tests/settingsDraft.test.ts`, `tests/summaryState.test.ts`

**Interfaces:**
- Produces: `settings_get`, `settings_save`, `ai_test_connection`, `ai_generate_summary`.
- Consumes: Task 1 project store and Task 4 project detail.

- [ ] **Step 1: Write failing settings-draft tests**

Assert field edits do not call save, Test Connection uses the draft without persisting it, and Save Settings submits one complete validated payload.

- [ ] **Step 2: Implement Settings page with only two sections**

Render File Storage and AI Service, no startup, update, model-management, or unrelated options.

- [ ] **Step 3: Write failing migration success and rollback tests**

Use temporary roots. Success must leave readable projects at the new root and update settings. A forced copy/validation failure must keep the old root active and readable.

- [ ] **Step 4: Implement storage migration and API-key secret boundary**

Do not return a stored key to React or write it into JSON/logs. A blank key draft preserves the existing secret; explicit clear removes it.

- [ ] **Step 5: Write failing AI response tests**

Use a local test server fixture and literal JSON responses to verify summary/keyPoints/actionItems parsing, authentication failure, malformed response rejection, old-summary preservation, and transcript revision stale detection.

- [ ] **Step 6: Implement OpenAI Compatible test and summary generation**

Send only saved transcript text. Persist a valid summary atomically and never overwrite it on failure.

- [ ] **Step 7: Run focused tests, full tests, typecheck, and build**

Expected: settings save semantics and AI persistence/error behavior pass.

---

### Task 7: Legacy migration, governing docs, and full verification

**Files:**
- Create: `src-tauri/src/legacy.rs`
- Modify: `src-tauri/src/main.rs`
- Modify: `DEVELOPMENT_SPEC.md`
- Modify: `UI_DESIGN_SPEC.md`
- Modify: `docs/architecture.md`
- Create: `docs/decisions/004-project-json-storage.md`
- Create: `docs/decisions/005-optional-openai-compatible-summary.md`
- Create: `docs/decisions/006-default-microphone-recording.md`
- Test: Rust legacy import tests

**Interfaces:**
- Produces: idempotent `import_legacy_txt(history_dir, project_store, ledger)`.
- Consumes: all preceding tasks and the approved spec.

- [ ] **Step 1: Write failing legacy import tests**

Assert a TXT becomes one media-less project with preserved text, the original TXT remains byte-identical, and a second import creates no duplicate.

- [ ] **Step 2: Implement idempotent legacy import and startup repair**

Mark stale `transcribing` projects failed at startup and import old TXT through a path/size/modified ledger.

- [ ] **Step 3: Update governing documentation**

Remove obsolete prohibitions and document the exact new page, storage, recording, AI, and migration contracts without weakening the preserved local ASR contract.

- [ ] **Step 4: Restore bundled runtime resources and run full verification**

Run:

```powershell
npm test
npm run typecheck
npm run build
Push-Location src-tauri
cargo test
cargo check
Pop-Location
npm run tauri build
git diff --check
git status --short
```

- [ ] **Step 5: Perform the approved Windows end-to-end smoke matrix**

Cover reference import, copied import, microphone recording, auto transcription, transcript edit/search/export, AI test/generation, media deletion/relink, data-root migration, legacy TXT import, restart persistence, and NSIS installation.

- [ ] **Step 6: Report exact incomplete phase if any verification is blocked**

Do not claim the application complete unless the full verification and smoke matrix have fresh evidence. Do not commit or push.
