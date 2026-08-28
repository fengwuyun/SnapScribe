# AI Service Multi-Model Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build an AI Service top-level page with custom instructions, ordered model profiles, sequential failover, runtime status updates, and project-summary redirection.

**Architecture:** Rust owns persisted model profiles, per-model DPAPI credentials, migration, request classification, and sequential failover. React owns navigation, configuration UI, modal state, drag reordering, and redirect UX. The existing single-model configuration remains migration input only.

**Tech Stack:** Tauri 2, Rust, reqwest blocking client, serde, React 18, TypeScript, Tailwind CSS, Vitest.

**Spec:** `docs/superpowers/specs/2026-08-28-ai-service-multi-model-design.md`

## Global Constraints

- Preserve existing uncommitted work and do not create commits or push.
- Prioritize functional behavior; visual polish follows the working request chain.
- Keep the current OpenAI Chat Completions request/response format as the only protocol in v0.1.4.
- API keys must never be serialized into model JSON or returned to the frontend.
- The first enabled model by order is derived as default.
- Final formal version is exactly `0.1.4` in all manifests and installer filename.

---

### Task 1: AI model domain and persistence

**Files:**
- Create: `src-tauri/src/ai_service_store.rs`
- Modify: `src-tauri/src/model.rs`
- Modify: `src-tauri/src/main.rs`
- Test: Rust unit tests in `ai_service_store.rs`

**Interfaces:**
- Produces `AIServiceConfig`, `AIModelConfig`, `AIModelStatus`, `AIModelStatusKind`, `AIModelDraft`.
- Produces `AIServiceStore::{load, save_instruction, create_model, update_model, delete_model, reorder_models, set_enabled, update_status}`.

- [ ] Write failing tests for default empty config, normalized order, CRUD, reorder, and first-enabled default derivation.
- [ ] Run `cargo test --manifest-path src-tauri/Cargo.toml ai_service_store` and verify RED.
- [ ] Implement atomic JSON persistence and validation.
- [ ] Run focused Rust tests and verify GREEN.

### Task 2: Per-model encrypted credentials and legacy migration

**Files:**
- Modify: `src-tauri/src/secret.rs`
- Modify: `src-tauri/src/ai_service_store.rs`
- Modify: `src-tauri/src/settings.rs`
- Test: Rust tests in `secret.rs` and `ai_service_store.rs`

**Interfaces:**
- Produces `ModelSecretStore::{save, load, clear, has_key}` keyed by model UUID.
- Produces `AIServiceStore::migrate_legacy(&AppSettings, Option<String>)`.

- [ ] Write failing tests for per-model secret isolation and idempotent migration metadata.
- [ ] Run focused tests and verify RED.
- [ ] Implement per-model DPAPI files and migration from the old single key.
- [ ] Run focused tests and verify GREEN.

### Task 3: Request execution, status classification, and failover

**Files:**
- Rewrite: `src-tauri/src/ai.rs`
- Modify: `src-tauri/src/model.rs`
- Test: Rust tests in `ai.rs`

**Interfaces:**
- Produces `build_endpoint`, `classify_failure`, `generate_summary_with_failover`.
- Consumes ordered `AIModelConfig` snapshots and per-model secrets.
- Returns `AISummary` with actual `model_config_id`, `model_name`, and `model`.

- [ ] Write failing pure tests for endpoint modes, authentication modes, status classification, prompt composition, and sequential-stop behavior using an injectable request executor.
- [ ] Run focused tests and verify RED.
- [ ] Implement one-model request execution and sequential failover.
- [ ] Persist each attempt status and aggregate failures.
- [ ] Run focused tests and verify GREEN.

### Task 4: Tauri commands and frontend API types

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/main.rs`
- Modify: `src/lib/tauri.ts`
- Modify: `src/types/project.ts`

**Interfaces:**
- Produces commands `ai_service_get`, `ai_instruction_save`, `ai_model_create`, `ai_model_update`, `ai_model_delete`, `ai_model_reorder`, `ai_model_set_enabled`, `ai_model_test`.
- Changes `ai_generate_summary` to use store-driven failover.

- [ ] Add TypeScript domain types matching Rust camelCase serialization.
- [ ] Add command wrappers.
- [ ] Implement commands with secret redaction and structured failure messages.
- [ ] Run `npm run typecheck` and `cargo test`.

### Task 5: Top-level navigation and AI Service page

**Files:**
- Modify: `src/lib/navigation.ts`
- Modify: `src/components/AppSidebar.tsx`
- Modify: `src/App.tsx`
- Create: `src/pages/AIServicePage.tsx`
- Create: `src/components/AIModelModal.tsx`
- Modify: `src/pages/SettingsPage.tsx`
- Test: `tests/navigation.test.ts`, create `tests/aiServiceUi.test.tsx`

**Interfaces:**
- Adds route `{ page: "ai-service"; focus?: "models"; returnToProjectId?: string }`.
- AI page consumes all command wrappers from Task 4.

- [ ] Write failing navigation and component-markup tests.
- [ ] Run focused Vitest tests and verify RED.
- [ ] Add sidebar route and remove AI card/tab from Settings.
- [ ] Implement instruction view/edit state and explicit save.
- [ ] Implement model rows, derived default label, status text, enable/test/edit/delete actions.
- [ ] Implement add/edit modal and validation.
- [ ] Run focused tests and verify GREEN.

### Task 6: Drag sorting and project redirect

**Files:**
- Modify: `src/pages/AIServicePage.tsx`
- Modify: `src/pages/ProjectDetailPage.tsx`
- Modify: `src/App.tsx`
- Test: `tests/navigation.test.ts`, `tests/aiServiceUi.test.tsx`

**Interfaces:**
- Reorder passes complete model ID order to `aiModelReorder`.
- Project detail checks enabled-model availability and navigates with return context.

- [ ] Write failing tests for default derivation after reorder and redirect route construction.
- [ ] Implement drag/drop with drop indicator and immediate persistence.
- [ ] Implement no-model redirect and post-save return CTA.
- [ ] Run focused and full frontend tests.

### Task 7: Version, verification, and installer

**Files:**
- Modify: `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/tauri.conf.json`
- Output: `outputs/SnapScribe_0.1.4_x64-setup.exe`

- [ ] Set all formal versions to `0.1.4`.
- [ ] Run `npm test` and require zero failures.
- [ ] Run `npm run typecheck` and `npm run build`.
- [ ] Run `cargo test --manifest-path src-tauri/Cargo.toml` and `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`.
- [ ] Run `npm run tauri build -- --verbose` and wait for final exit code 0.
- [ ] Copy installer to outputs and verify source/destination size, SHA-256, and four version sources.
- [ ] Confirm git status remains uncommitted and report deliverables.
