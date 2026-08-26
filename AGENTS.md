# Project Instructions

## Goal

Build SnapScribe — a local Windows desktop app that turns video/audio files into
text with timestamps. Fully local: no login, no API key, no model download at
runtime, no Python/FFmpeg install for end users.

Priorities:

1. Simplicity
2. Stability
3. Low maintenance
4. UI consistency

## Required Docs

Before implementing features, read:

- `DEVELOPMENT_SPEC.md` — product goals, boundaries, architecture, pipeline contract
- `UI_DESIGN_SPEC.md` — design tokens, components, states, visual constraints

These are the source of truth. This file holds only long-term rules.

## Tech Stack

- Tauri 2
- React 18 + TypeScript
- Tailwind CSS 3
- shadcn/ui conventions (Button / Dialog / Input)
- Lucide Icons (only icon library)
- Rust
- FFmpeg / ffprobe (bundled binaries)
- funasr-llamacpp runtime (`llama-funasr-sensevoice.exe`, SenseVoiceSmall GGUF q8 + FSMN-VAD GGUF)

Do not introduce alternative frameworks, state libraries, or icon libraries
without explicit approval. Package manager is npm (pnpm is unavailable).

## Architecture

React handles:

- UI
- interaction
- presentation state

Rust handles:

- filesystem access
- FFmpeg extraction and segmentation
- ASR subprocess lifecycle and output parsing
- transcript data normalization
- exports and history-file management
- task cancellation and temp cleanup

Do not move native processing into React. Do not parse ASR output in the
frontend; Rust emits normalized `TranscriptSegment` data only.

## Segmented Transcription Contract

The transcription pipeline is fixed:

1. FFmpeg converts input to 16 kHz mono PCM WAV and splits it into fixed-length
   segments in one pass (`-f segment`).
2. Rust transcribes segments strictly sequentially.
3. Every finished segment is emitted immediately as a Tauri event; the UI appends
   it incrementally. Never buffer all segments until the end.
4. Progress percent derives from transcribed seconds / total seconds and must be
   monotonically non-decreasing.
5. Cancel kills the current child process, stops the loop, and removes temp
   segment files.

Do not switch to parallel transcription, whole-file transcription, or VAD-only
segmentation without explicit approval.

## UI Constraints

Strictly follow `UI_DESIGN_SPEC.md`. Do not introduce:

- new brand colors, radius values, shadow systems, or fonts
- another icon library
- gradients, glassmorphism, glow, heavy shadows
- dashboard layouts, sidebars, multi-page navigation

Transcript reading efficiency always wins over decoration.

## Product Constraints

Do not add unrequested features. Do not introduce authentication, cloud sync,
online APIs, AI post-processing, speaker diarization, multi-model switching,
model management UI, databases, or auto-update.

History management means exactly: auto-saved TXT transcripts listed from the
history directory with preview / rename / delete. No metadata database, no tags,
no search beyond filename.

## Engineering Rules

- Prefer YAGNI. No abstraction without two real use cases.
- One task = one commit. Commit messages follow Conventional Commits
  (`feat:`, `fix:`, `test:`, `chore:`).
- Add dependencies only when the current stack cannot solve the need; state why.
- Do not refactor unrelated code. Keep files focused.
- An empty `catch` must name what it swallows and why.
- Never silently change product behavior.

## Verification

Before claiming completion:

1. Run relevant unit tests (`npm test`, `cargo test`).
2. Run `npx tsc --noEmit`.
3. Run frontend build (`npm run build`).
4. Run `cargo check` (and `tauri build` when packaging changes).
5. Inspect `git diff` for unrelated changes and leftover debug code.
6. Compare against the task's acceptance criteria.

Never claim success without fresh verification evidence. Report commands
executed, pass/fail, and remaining issues.
