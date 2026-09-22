# Speaker Diarization and Batch Rename Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add optional local anonymous speaker diarization for imported media and live recordings, plus project-wide speaker renaming that updates transcript display, copy, export, and future AI summaries.

**Architecture:** Keep the existing SenseVoice transcription pipeline and add a cancellable native Sherpa-ONNX diarization sidecar. Persist stable speaker IDs separately from editable display names; align diarization turns to ASR cues by maximum time overlap. Live recording uses 15-second provisional diarization and a final full-recording pass after stop.

**Tech Stack:** Tauri 2, Rust 2021, React 18, TypeScript, Vitest, Rust unit tests, FFmpeg, Sherpa-ONNX v1.13.8, pyannote segmentation 3.0 ONNX, 3D-Speaker Chinese embedding ONNX.

**Spec:** `docs/superpowers/specs/2026-09-22-speaker-diarization-design.md`

## Global Constraints

- Speaker diarization is opt-in and defaults off for every new import or recording operation.
- File import still starts automatically; no confirmation step is added.
- Speaker count is automatic; the UI never asks users to enter a count.
- Persist anonymous stable IDs (`speaker-1`) separately from mutable names (`说话人 1`).
- Diarization failure never discards or blocks saved transcript text.
- Old schema-version-1 project files must remain readable without migration dialogs.
- Live recording remains on the existing 15-second chunk cadence.
- No real-name voice identification or cross-project voiceprint database is introduced.
- Do not commit, push, tag, or publish unless the user separately requests it.

## Review Focus

- A version-1 transcript with no speaker fields must load and export exactly as before; Task 1 adds a compatibility test.
- A transcript cue that overlaps several speaker turns must deterministically select the largest overlap and preserve its text; Task 3 adds boundary tests.
- Diarization runtime/model failure must finish the project with text and a non-blocking speaker error; Task 4 adds a pipeline failure test.
- Live provisional labels may be corrected after stop without appending duplicate text; Task 5 adds a full-document replacement test.
- Batch rename must not alter timestamps, text, IDs, or unrelated speakers; Task 6 adds store and frontend formatter tests.

---

## File Structure

### New files

- `src-tauri/src/diarization.rs` — sidecar invocation, JSON parsing, cancellation, overlap alignment, and live speaker-registry matching.
- `src-tauri/diarization-sidecar/CMakeLists.txt` — pinned Windows x64 native sidecar build definition.
- `src-tauri/diarization-sidecar/main.cc` — Sherpa-ONNX C++ wrapper producing newline-delimited JSON progress/results.
- `src-tauri/resources/DIARIZATION_LICENSES.md` — pinned sources, computed SHA-256 values, and redistributed licenses.
- `src-tauri/test-data/0-four-speakers-zh.wav` — official Sherpa-ONNX four-speaker Chinese smoke-test fixture.
- `src/components/SpeakerManagerDialog.tsx` — edit and atomically save all project speaker display names.
- `src/lib/speakers.ts` — speaker label lookup, display formatting, colors, and rename validation.
- `src/lib/speakers.test.ts` — frontend speaker helper tests.
- `src/lib/format.test.ts` — clipboard/export text formatting tests.

### Modified files

- `src-tauri/src/model.rs` — schema-2 transcript and diarization state types.
- `src-tauri/src/project_store.rs` — backward-compatible reads, speaker persistence, and atomic rename.
- `src-tauri/src/runtime.rs` — optional diarization runtime/model resolution.
- `src-tauri/src/pipeline.rs` — imported-file diarization and progress stages.
- `src-tauri/src/commands.rs` — opt-in flags, speaker update command, recording final correction.
- `src-tauri/src/export.rs` — speaker-aware TXT/SRT rendering.
- `src-tauri/src/ai.rs` — speaker-aware AI transcript input.
- `src-tauri/src/main.rs` — module and command registration.
- `src-tauri/tauri.conf.json` — include diarization binary, DLLs, and models.
- `src/types/transcript.ts` and `src/types/project.ts` — frontend mirrored types.
- `src/lib/tauri.ts` — updated command arguments and speaker update binding.
- `src/lib/importProject.ts` — pass the opt-in choice without changing navigation.
- `src/lib/format.ts` — speaker-aware full-text copy formatting.
- `src/pages/HomePage.tsx` — compact opt-in control for import and recording.
- `src/pages/ProjectDetailPage.tsx` — status text, manager entry, edit-view speaker labels.
- `src/components/TranscriptList.tsx` — colored speaker badges and speaker-aware segment copy.
- `src/components/RecordingPanel.tsx` — provisional label rendering and correction status.
- `src/components/TranscribeProgress.tsx` — diarization/reconciliation stage labels.

---

### Task 1: Add backward-compatible speaker data and atomic renaming

**Files:**
- Modify: `src-tauri/src/model.rs`
- Modify: `src-tauri/src/project_store.rs`
- Modify: `src/types/transcript.ts`
- Modify: `src/types/project.ts`

**Interfaces:**
- Produces: `TranscriptSpeaker { id, name, color_index }`, optional `TranscriptSegment.speaker_id`, `DiarizationState`, `ProjectStore::update_speakers(project_id, speakers)`.
- Consumes: existing `TranscriptDocument`, `TranscriptionProject`, and atomic JSON write helpers.

- [ ] **Step 1: Add failing Rust compatibility and rename tests**

Add tests that deserialize this version-1 payload and assert empty speakers plus absent speaker IDs:

```rust
let old = r#"{"schemaVersion":1,"projectId":"p1","revision":1,"segments":[{"id":"s1","start":0.0,"end":1.0,"text":"你好"}]}"#;
let doc: TranscriptDocument = serde_json::from_str(old).unwrap();
assert!(doc.speakers.is_empty());
assert_eq!(doc.segments[0].speaker_id, None);
```

Add a store test with two segments using `speaker-1`; rename it to `刘德华` and assert segment text, timestamps, IDs, and `speakerId` are unchanged while only `speakers[0].name` changes.

- [ ] **Step 2: Run the focused tests and confirm failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml transcript_v1_defaults_speaker_fields
cargo test --manifest-path src-tauri/Cargo.toml batch_rename_only_updates_display_name
```

Expected: compile failure because the speaker fields and update method do not exist.

- [ ] **Step 3: Add schema-2 types with serde defaults**

Implement:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptSpeaker {
    pub id: String,
    pub name: String,
    pub color_index: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DiarizationState {
    pub enabled: bool,
    pub status: DiarizationStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
```

Add `#[serde(default)] pub speakers: Vec<TranscriptSpeaker>` to `TranscriptDocument`, `#[serde(default, skip_serializing_if = "Option::is_none")] pub speaker_id: Option<String>` to `TranscriptSegment`, and defaulted diarization state to `TranscriptionProject`. New transcript writes use schema version 2; reads accept both versions.

- [ ] **Step 4: Implement validated atomic speaker updates**

Add:

```rust
pub fn update_speakers(
    &self,
    project_id: &str,
    speakers: Vec<TranscriptSpeaker>,
) -> Result<TranscriptDocument, String>
```

Trim names, reject empty names, reject unknown/duplicate IDs, preserve `segments`, increment transcript revision once, and write through `atomic_write_json`.

- [ ] **Step 5: Mirror the exact types in TypeScript and rerun tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
npm run typecheck
```

Expected: both commands pass.

- [ ] **Step 6: Record a logical checkpoint without committing**

Run `git diff --check` and keep changes uncommitted per the global constraint.

---

### Task 2: Build and resolve the local Sherpa-ONNX diarization runtime

**Files:**
- Create: `src-tauri/diarization-sidecar/CMakeLists.txt`
- Create: `src-tauri/diarization-sidecar/main.cc`
- Create: `src-tauri/src/diarization.rs`
- Modify: `src-tauri/src/runtime.rs`
- Modify: `src-tauri/src/main.rs`
- Modify: `src-tauri/tauri.conf.json`

**Interfaces:**
- Produces: `DiarizationRuntime`, `DiarizationTurn`, `DiarizationOutput`, `diarization::spawn(...)`, `diarization::collect_output_shared(...)`.
- Consumes: 16 kHz mono PCM WAV and the same `Arc<Mutex<Option<Child>>>` cancellation pattern used by ASR.

- [ ] **Step 1: Add failing runtime and JSON parser tests**

Test installed layout resolution for:

```text
resources/bin/snapscribe-diarization.exe
resources/bin/sherpa-onnx-c-api.dll
resources/models/speaker-segmentation/model.int8.onnx
resources/models/speaker-embedding/3dspeaker.onnx
```

Test parsing one final sidecar line:

```json
{"type":"result","speakers":2,"turns":[{"start":0.2,"end":2.1,"speaker":0},{"start":2.2,"end":4.0,"speaker":1}]}
```

- [ ] **Step 2: Run focused tests and confirm failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml diarization
```

Expected: compile failure because the runtime and parser do not exist.

- [ ] **Step 3: Implement the sidecar contract**

The executable accepts this concrete argument shape:

```text
snapscribe-diarization.exe --segmentation src-tauri/resources/models/speaker-segmentation/model.int8.onnx --embedding src-tauri/resources/models/speaker-embedding/3dspeaker.onnx --audio src-tauri/test-data/0-four-speakers-zh.wav --clusters -1 --threshold 0.5
```

`--clusters -1 --threshold 0.5` enables automatic speaker-count estimation using Sherpa-ONNX's documented clustering threshold. It writes progress JSON to stdout and one final result JSON; diagnostics go to stderr. It exits nonzero on invalid audio/model/runtime errors. Build against pinned Sherpa-ONNX v1.13.8 Windows x64 shared libraries.

- [ ] **Step 4: Implement Rust spawning and parsing**

Add typed structures:

```rust
pub struct DiarizationTurn { pub start: f64, pub end: f64, pub speaker: usize }
pub struct DiarizationOutput { pub speakers: usize, pub turns: Vec<DiarizationTurn> }
```

Reject non-finite/negative times, reversed intervals, speaker indexes outside the returned count, malformed JSON, and successful processes with no final result.

- [ ] **Step 5: Add runtime resources and package entries**

Download the segmentation model from `https://github.com/k2-fsa/sherpa-onnx/releases/download/speaker-segmentation-models/sherpa-onnx-pyannote-segmentation-3-0.tar.bz2`, the embedding model from `https://github.com/k2-fsa/sherpa-onnx/releases/download/speaker-recongition-models/3dspeaker_speech_eres2net_base_sv_zh-cn_3dspeaker_16k.onnx`, and the smoke fixture from `https://github.com/k2-fsa/sherpa-onnx/releases/download/speaker-segmentation-models/0-four-speakers-zh.wav`. Place the extracted/renamed assets in the paths above, compute SHA-256 values with `Get-FileHash -Algorithm SHA256`, record the actual hashes and redistributed licenses in `src-tauri/resources/DIARIZATION_LICENSES.md`, and keep `tauri.conf.json` resource globs covering them.

- [ ] **Step 6: Verify runtime resolution and sidecar smoke test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml diarization
src-tauri\resources\bin\snapscribe-diarization.exe --segmentation src-tauri\resources\models\speaker-segmentation\model.int8.onnx --embedding src-tauri\resources\models\speaker-embedding\3dspeaker.onnx --audio src-tauri\test-data\0-four-speakers-zh.wav --clusters -1 --threshold 0.5
```

Expected: tests pass; the smoke command returns JSON containing at least one turn and exits 0.

- [ ] **Step 7: Record a logical checkpoint without committing**

Run `git diff --check` and keep changes uncommitted.

---

### Task 3: Align diarization turns with transcript cues

**Files:**
- Modify: `src-tauri/src/diarization.rs`
- Modify: `src-tauri/src/export.rs`

**Interfaces:**
- Produces: `assign_speakers(segments, turns) -> SpeakerAssignment`, `speaker_name(document, speaker_id)` and speaker-aware export functions.
- Consumes: Task 1 transcript types and Task 2 `DiarizationTurn` values.

- [ ] **Step 1: Add failing overlap tests**

Cover all cases:

```rust
// 0..10 overlaps speaker 0 for 6 seconds and speaker 1 for 4 seconds.
assert_eq!(assigned[0].speaker_id.as_deref(), Some("speaker-1"));

// Equal overlap resolves by earliest turn, then lowest speaker index.
assert_eq!(tie[0].speaker_id.as_deref(), Some("speaker-1"));

// No overlap keeps None and preserves the source segment byte-for-byte.
assert_eq!(silent[0], original);
```

Also test stable `TranscriptSpeaker` order and color indexes after receiving non-contiguous raw indexes.

- [ ] **Step 2: Run focused tests and confirm failure**

Run `cargo test --manifest-path src-tauri/Cargo.toml assign_speakers`.

- [ ] **Step 3: Implement deterministic overlap assignment**

For each text segment, calculate `max(0, min(end) - max(start))` against every turn, sum overlap by raw speaker, then choose the greatest sum. Ties resolve by earliest overlapping turn and then raw index. Map used raw indexes in first-appearance order to stable IDs `speaker-1`, `speaker-2`, … and names `说话人 1`, `说话人 2`, ….

- [ ] **Step 4: Make exports speaker-aware without changing unlabeled output**

Change signatures to:

```rust
pub fn build_txt(document: &TranscriptDocument) -> String
pub fn build_srt(document: &TranscriptDocument) -> String
```

Labeled TXT lines use `[MM:SS] 名称：文本`; labeled SRT body uses `名称：文本`. If the document has no labeled segment, output must remain identical to version 0.1.6.

- [ ] **Step 5: Run focused and full Rust tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml assign_speakers
cargo test --manifest-path src-tauri/Cargo.toml export
```

- [ ] **Step 6: Record a logical checkpoint without committing**

Run `git diff --check` and keep changes uncommitted.

---

### Task 4: Integrate diarization into imported-file transcription

**Files:**
- Modify: `src-tauri/src/pipeline.rs`
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/project_store.rs`
- Modify: `src-tauri/src/model.rs`
- Modify: `src/lib/tauri.ts`
- Modify: `src/lib/importProject.ts`

**Interfaces:**
- Produces: `project_create_from_media(path, diarization_enabled)`, progress stages `diarizing` and `reconcilingSpeakers`, completed/failed diarization state persistence.
- Consumes: Tasks 1–3 types, sidecar runner, and alignment function.

- [ ] **Step 1: Add failing pipeline policy tests**

Extract a pure completion policy and test:

```rust
let result = finish_with_diarization(transcript, Err("model missing".into()));
assert_eq!(result.document.segments[0].text, "保留文本");
assert_eq!(result.state.status, DiarizationStatus::Failed);
assert!(result.state.error.unwrap().contains("model missing"));
```

Add a disabled-path test asserting that no diarization runtime resolution is attempted and existing transcription output is unchanged.

- [ ] **Step 2: Run focused tests and confirm failure**

Run `cargo test --manifest-path src-tauri/Cargo.toml finish_with_diarization`.

- [ ] **Step 3: Thread the opt-in flag through frontend and commands**

Use exact signatures:

```ts
createProjectFromMedia(path: string, diarizationEnabled: boolean): Promise<StartProjectResult>
importAndOpenProject(path, from, diarizationEnabled, createProject, navigate)
```

```rust
pub fn project_create_from_media(..., path: String, diarization_enabled: bool)
```

Persist the chosen flag before spawning the job.

- [ ] **Step 4: Add post-ASR diarization with cancellable progress**

After normal ASR completes, run diarization only when enabled. Emit `diarizing` progress from sidecar callbacks, then `reconcilingSpeakers` while assigning IDs. Pause prevents starting the diarization process; cancellation kills the current child. On error, save the complete text, persist diarization `failed`, and emit a non-fatal document refresh rather than `transcript://failed` for the whole job.

- [ ] **Step 5: Persist the final schema-2 document atomically**

Update `complete_transcription` to accept the complete `TranscriptDocument` so speakers and segments are written together and revision increments once.

- [ ] **Step 6: Run Rust and TypeScript checks**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
npm run typecheck
```

- [ ] **Step 7: Record a logical checkpoint without committing**

Run `git diff --check` and keep changes uncommitted.

---

### Task 5: Add provisional live labels and final recording correction

**Files:**
- Modify: `src-tauri/src/diarization.rs`
- Modify: `src-tauri/src/pipeline.rs`
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/model.rs`
- Modify: `src/lib/tauri.ts`
- Modify: `src/pages/HomePage.tsx`

**Interfaces:**
- Produces: `recording_start(diarization_enabled)`, `LiveSpeakerRegistry::match_chunk`, final `transcript://replaced` event.
- Consumes: 15-second `RecordedChunk`, diarization output, Task 3 alignment.

- [ ] **Step 1: Add failing live-registry and replacement tests**

Test that embeddings above the cosine threshold reuse a stable ID, embeddings below it create the next ID, and final correction replaces the same segment IDs without appending duplicate text. Test final-pass failure keeps provisional labels.

- [ ] **Step 2: Run focused tests and confirm failure**

Run `cargo test --manifest-path src-tauri/Cargo.toml live_speaker`.

- [ ] **Step 3: Extend sidecar output for live speaker centroids**

For chunk mode, output one normalized centroid embedding per raw speaker in the final JSON. Keep embeddings in memory only; never persist them or expose them to the frontend.

- [ ] **Step 4: Implement the session speaker registry**

Use cosine similarity with a single named constant `LIVE_SPEAKER_MATCH_THRESHOLD: f32 = 0.65`. Match each chunk centroid to the best existing centroid above the threshold; otherwise allocate the next stable ID. Update a matched centroid with a duration-weighted average. Unit tests pin threshold behavior at 0.649 and 0.650.

- [ ] **Step 5: Diarize each completed 15-second chunk when enabled**

Run ASR first so subtitles remain responsive, then diarize the same chunk and emit speaker-tagged segment replacements. The UI must not append replacements as new segments.

- [ ] **Step 6: Run full-recording final correction after stop**

After the live worker finishes and the WAV header is finalized, run full-audio diarization, align against all text, atomically replace the document, and emit `transcript://replaced`. On failure, keep provisional IDs, store a non-blocking error, and still complete the recording project.

- [ ] **Step 7: Run targeted recording tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml live_speaker
cargo test --manifest-path src-tauri/Cargo.toml recording
npm run typecheck
```

- [ ] **Step 8: Record a logical checkpoint without committing**

Run `git diff --check` and keep changes uncommitted.

---

### Task 6: Add speaker controls, badges, and batch rename UI

**Files:**
- Create: `src/components/SpeakerManagerDialog.tsx`
- Create: `src/lib/speakers.ts`
- Modify: `src/pages/HomePage.tsx`
- Modify: `src/pages/ProjectDetailPage.tsx`
- Modify: `src/components/TranscriptList.tsx`
- Modify: `src/components/RecordingPanel.tsx`
- Modify: `src/components/TranscribeProgress.tsx`
- Modify: `src/lib/tauri.ts`
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/main.rs`

**Interfaces:**
- Produces: `speakerDisplayName`, `speakerBadgeClass`, `validateSpeakerDrafts`, `projectUpdateSpeakers` and `SpeakerManagerDialog`.
- Consumes: schema-2 speaker types and diarization progress/replacement events.

- [ ] **Step 1: Add failing frontend helper tests**

Test exact behaviors:

```ts
expect(speakerDisplayName("speaker-1", speakers)).toBe("刘德华");
expect(speakerDisplayName(undefined, speakers)).toBe("");
expect(validateSpeakerDrafts([{ id: "speaker-1", name: "   ", colorIndex: 0 }])).toEqual({ ok: false, message: "说话人名称不能为空" });
```

Test that duplicate display names return a warning but remain savable.

- [ ] **Step 2: Run Vitest and confirm failure**

Run `npm test -- src/lib/speakers.test.ts`.

- [ ] **Step 3: Add the opt-in home control**

Add one compact switch next to the upload helper and recording entry. Keep `diarizationEnabled` local to `HomePage`, initialize it to `false`, and pass it directly to import/record commands. Do not insert an intermediate dialog.

- [ ] **Step 4: Render stable speaker badges without breaking selection/seek**

Add the badge between timestamp and text. Timestamp and single-click text still seek; double-click copies `[MM:SS] 名称：文本`; drag selection and context-menu copy remain native. Hide the badge entirely for unlabeled segments.

- [ ] **Step 5: Implement the manager dialog and command**

Register:

```rust
project_update_speakers(project_id: String, speakers: Vec<TranscriptSpeaker>)
```

The dialog copies server data into drafts on open, trims on save, blocks empty names, shows but does not block duplicate-name warnings, disables save while submitting, and refreshes project detail only after the atomic command succeeds.

- [ ] **Step 6: Add progress and live-correction presentation**

Map stages to “正在识别说话人” and “正在校正说话人”. Keep existing transcript rows mounted; on `transcript://replaced`, replace the local segment array by ID/document instead of clearing and appending.

- [ ] **Step 7: Run frontend verification**

Run:

```powershell
npm test
npm run typecheck
npm run build
```

- [ ] **Step 8: Record a logical checkpoint without committing**

Run `git diff --check` and keep changes uncommitted.

---

### Task 7: Connect copy, export, AI summary, and end-to-end packaging

**Files:**
- Modify: `src/lib/format.ts`
- Modify: `src/pages/ProjectDetailPage.tsx`
- Modify: `src-tauri/src/export.rs`
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/ai.rs`
- Modify: `src-tauri/src/pipeline.rs`
- Modify: `src-tauri/tauri.conf.json`

**Interfaces:**
- Produces: consistent speaker-aware text across clipboard, TXT, SRT, auto-saved history, and AI input.
- Consumes: final transcript document and current speaker-name mapping.

- [ ] **Step 1: Add failing formatter and AI-input tests**

Assert labeled clipboard/TXT text exactly equals:

```text
[00:17] 刘德华：这里是第一段。
[00:25] 说话人 2：这里是第二段。
```

Assert unlabeled documents keep the existing plain-line output. Assert AI prompt input includes current renamed labels and does not include raw `speaker-1` IDs.

- [ ] **Step 2: Run focused tests and confirm failure**

Run:

```powershell
npm test -- src/lib/format.test.ts
cargo test --manifest-path src-tauri/Cargo.toml speaker_aware
```

- [ ] **Step 3: Centralize frontend and backend formatting**

Change frontend `fullText` to accept a transcript document or explicit speakers, and update every call site. Change backend export, history auto-save, and AI summary assembly to use the same `name：text` rule. Avoid speaker prefixes for documents with no labeled segments.

- [ ] **Step 4: Run complete automated checks**

Run:

```powershell
npm test
npm run typecheck
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
git diff --check
```

- [ ] **Step 5: Perform focused manual acceptance**

Verify one old project, one imported two-speaker file, one 45-second two-speaker recording, rename `说话人 1` to `刘德华`, seek by clicking its text, copy one segment, copy full text, export TXT/SRT, generate a new AI summary, cancel one diarization run, and simulate a missing diarization model. Confirm text survives every failure case.

- [ ] **Step 6: Build and inspect the NSIS installer only when requested**

Run `npm run tauri build`, record the generated EXE path and size, install on a clean Windows environment without Python/network, and confirm the bundled sidecar and models resolve. Do not change the application version unless the user asks.

- [ ] **Step 7: Report final status without committing**

List changed files, automated-check results, manual acceptance results, remaining diarization accuracy limitations, installer path if built, and `git status --short`.
