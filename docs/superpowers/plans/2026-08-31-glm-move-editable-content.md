# SnapScribe GLM、文件移动与可编辑内容 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 SnapScribe 0.1.6 增加 GLM 4.7 Flash 默认预设、安全移动导入、数据目录打开、可编辑待办和可靠文本复制，并生成 Windows 安装包。

**Architecture:** 保留 React/Tauri 边界：React 负责交互状态，Rust 负责配置迁移、文件系统、原子保存和密钥。对现有 JSON 使用 serde 兼容读取和幂等迁移；移动导入采用复制校验后删除源文件，待办使用稳定 ID 对象。

**Tech Stack:** React 18、TypeScript、Vitest、Tauri 2、Rust、serde、DPAPI、NSIS。

**Spec:** `docs/superpowers/specs/2026-08-31-glm-move-editable-content-design.md`

## Global Constraints

- 不实现转录文本原地编辑；保留现有“编辑”按钮、保存流程和单击段落跳转。
- GLM Base URL 固定为 `https://open.bigmodel.cn/api/paas/v4`，模型 ID 固定为 `glm-4.7-flash`。
- API Key 仅存 DPAPI，不进入 JSON 或前端读取结果。
- 移动导入删除源文件必须是最后一步；失败不得丢失源文件。
- 旧模型配置、旧字符串待办和旧 `media/` 项目必须兼容。
- 设置仍通过“保存设置”提交；版本统一升级为 `0.1.6`。
- 每项生产代码必须先有失败测试。

---

### Task 1: GLM 预设数据模型与幂等迁移

**Files:**
- Modify: `src-tauri/src/model.rs`
- Modify: `src-tauri/src/ai_service_store.rs`
- Modify: `src-tauri/src/commands.rs`
- Modify: `src/types/project.ts`
- Test: `src-tauri/src/ai_service_store.rs`

**Interfaces:**
- Produces: `GLM_47_FLASH_PRESET_KEY`, `AIModelConfig.preset_key: Option<String>`，以及 `AIServiceStore::ensure_default_presets() -> Result<AIServiceConfig, String>`。
- Consumes: 现有 `AIServiceStore::load/save` 与模型排序逻辑。

- [ ] **Step 1: 写失败测试**

覆盖空配置插入一个 GLM、已有配置插入第一位、重复调用不重复、保留原模型相对顺序。

```rust
#[test]
fn ensures_glm_preset_once_and_keeps_existing_order() {
    let store = test_store();
    store.create_model(&draft("existing-a")).unwrap();
    store.create_model(&draft("existing-b")).unwrap();
    let first = store.ensure_default_presets().unwrap();
    let second = store.ensure_default_presets().unwrap();
    assert_eq!(second.models.iter().filter(|m| m.preset_key.as_deref() == Some("glm-4.7-flash")).count(), 1);
    assert_eq!(first.models[0].model_id, "glm-4.7-flash");
    assert_eq!(first.models[1].name, "existing-a");
    assert_eq!(first.models[2].name, "existing-b");
}
```

- [ ] **Step 2: 运行测试确认 RED**

Run: `cargo test ai_service_store::tests::ensures_glm_preset_once_and_keeps_existing_order`

Expected: FAIL，缺少 `preset_key` 或 `ensure_default_presets`。

- [ ] **Step 3: 最小实现**

给模型增加 `#[serde(default, skip_serializing_if = "Option::is_none")] preset_key`。GLM 记录使用 UUID 作为内部 ID，固定名称、URL、模型 ID、Bearer 和 auto endpoint，插入 `order=0` 后统一重排。`ai_store()` 初始化后执行幂等预设迁移；`hydrate_ai_config` 计算密钥状态。

- [ ] **Step 4: 验证 GREEN**

Run: `cargo test ai_service_store::tests`

Expected: PASS。

- [ ] **Step 5: 提交**

```bash
git add src-tauri/src/model.rs src-tauri/src/ai_service_store.rs src-tauri/src/commands.rs src/types/project.ts
git commit -m "feat: add default GLM model preset"
```

### Task 2: GLM 待配置状态、故障转移与简化弹窗

**Files:**
- Modify: `src-tauri/src/ai.rs`
- Modify: `src/pages/AIServicePage.tsx`
- Modify: `src/components/AIModelDialog.tsx`
- Modify: `src/lib/aiModelPresentation.ts`
- Test: `src-tauri/src/ai.rs`
- Test: `tests/ai-model-presentation.test.ts`

**Interfaces:**
- Consumes: `presetKey?: "glm-4.7-flash"` 与 `hasApiKey`。
- Produces: `getAIModelAvailability(model)`，状态 `needs-key` 仅用于展示；请求快照过滤缺少密钥的模型。

- [ ] **Step 1: 写失败测试**

```ts
it("marks the GLM preset without a key as needing configuration", () => {
  expect(getAIModelAvailability({ presetKey: "glm-4.7-flash", hasApiKey: false, enabled: true })).toBe("needs-key");
});
```

Rust 测试构造首个无 Key 的 GLM 和第二个有 Key 模型，断言故障转移跳过 GLM 并调用第二个。

- [ ] **Step 2: 运行测试确认 RED**

Run: `npx vitest run tests/ai-model-presentation.test.ts`

Run: `cargo test ai::tests`

Expected: FAIL，缺少展示判断或无 Key 过滤。

- [ ] **Step 3: 最小实现**

模型行对启用但缺 Key 的 Bearer/x-api-key/custom-header 模型显示“待配置”；不把缺 Key 变成红色失败。生成总结前过滤配置不完整模型。GLM 编辑弹窗根据 `presetKey` 把名称、URL、模型 ID设为只读并隐藏高级设置，只保留 Key、测试、取消、保存。

- [ ] **Step 4: 验证 GREEN**

Run: `npx vitest run tests/ai-model-presentation.test.ts`

Run: `cargo test ai::tests`

Expected: PASS。

- [ ] **Step 5: 提交**

```bash
git add src-tauri/src/ai.rs src/pages/AIServicePage.tsx src/components/AIModelDialog.tsx src/lib/aiModelPresentation.ts tests/ai-model-presentation.test.ts
git commit -m "feat: simplify GLM key configuration"
```

### Task 3: 安全移动导入与项目目录兼容

**Files:**
- Modify: `src-tauri/src/model.rs`
- Modify: `src-tauri/src/project_store.rs`
- Modify: `src-tauri/src/settings.rs`
- Modify: `src/types/project.ts`
- Test: `src-tauri/src/project_store.rs`
- Test: `src-tauri/src/settings.rs`

**Interfaces:**
- Produces: `ImportStrategy::Move`；`ProjectStore::create_imported_project` 支持移动；新管理媒体直接位于项目目录。
- Consumes: `MediaInfo`、现有临时项目目录、原子 JSON 写入。

- [ ] **Step 1: 写失败测试**

```rust
#[test]
fn move_import_commits_target_before_removing_source() {
    let source = write_media_fixture();
    let project = store.create_imported_project(&source, &media_info(), ImportStrategy::Move).unwrap();
    assert!(!source.exists());
    assert!(Path::new(project.media.path.as_ref().unwrap()).is_file());
    assert_eq!(Path::new(project.media.path.as_ref().unwrap()).parent(), Some(store.project_dir(&project.id).as_path()));
}
```

另写目标复制失败时源文件仍存在、旧 `media/` 路径仍可加载测试。

- [ ] **Step 2: 运行测试确认 RED**

Run: `cargo test project_store::tests`

Expected: FAIL，`ImportStrategy::Move` 不存在。

- [ ] **Step 3: 最小实现**

扩展枚举。对 copy/move 将媒体复制到项目暂存目录的临时文件，`sync_all` 后核对大小，重命名最终文件，写项目文档并提升暂存目录；Move 最后删除源文件。删除失败返回明确错误并保留目标项目。读取路径继续完全信任项目记录，因此旧 `media/` 项目无需迁移。

- [ ] **Step 4: 验证 GREEN**

Run: `cargo test project_store::tests settings::tests`

Expected: PASS。

- [ ] **Step 5: 提交**

```bash
git add src-tauri/src/model.rs src-tauri/src/project_store.rs src-tauri/src/settings.rs src/types/project.ts
git commit -m "feat: add safe move import strategy"
```

### Task 4: 设置页移动选项与打开数据目录

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/main.rs`
- Modify: `src/lib/tauri.ts`
- Modify: `src/pages/SettingsPage.tsx`
- Test: `tests/settings-header.test.tsx`

**Interfaces:**
- Produces: Tauri command `settings_open_data_root() -> Result<(), String>`；前端 `settingsOpenDataRoot(): Promise<void>`。
- Consumes: 已保存的 `SettingsStore::load().data_root`。

- [ ] **Step 1: 写失败测试**

扩展设置页面静态测试，断言包含 `value="move"`、警示说明和 `settingsOpenDataRoot` 调用入口。

```ts
expect(source).toContain('<option value="move">移动到 SnapScribe');
expect(source).toContain('打开当前文件夹');
expect(source).toContain('settingsOpenDataRoot');
```

- [ ] **Step 2: 运行测试确认 RED**

Run: `npx vitest run tests/settings-header.test.tsx`

Expected: FAIL，页面缺少移动选项和打开按钮。

- [ ] **Step 3: 最小实现**

设置页增加第三选项和删除源文件说明。数据路径行增加“打开当前文件夹”；Rust 读取已保存 dataRoot，创建目录后使用现有 Windows 打开路径方式打开，不使用前端 Shell。按钮不读取未保存的 draft 路径。

- [ ] **Step 4: 验证 GREEN**

Run: `npx vitest run tests/settings-header.test.tsx`

Run: `cargo test settings::tests`

Expected: PASS。

- [ ] **Step 5: 提交**

```bash
git add src-tauri/src/commands.rs src-tauri/src/main.rs src/lib/tauri.ts src/pages/SettingsPage.tsx tests/settings-header.test.tsx
git commit -m "feat: expose managed move storage settings"
```

### Task 5: 结构化待办与原子保存

**Files:**
- Modify: `src-tauri/src/model.rs`
- Modify: `src-tauri/src/ai.rs`
- Modify: `src-tauri/src/project_store.rs`
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/main.rs`
- Modify: `src/types/project.ts`
- Modify: `src/lib/tauri.ts`
- Test: `src-tauri/src/project_store.rs`

**Interfaces:**
- Produces: `ActionItem { id, text, completed }`；`project_action_items_save(project_id, items)`。
- Consumes: 旧 `actionItems: string[]` JSON 与现有 `atomic_write_json`。

- [ ] **Step 1: 写失败测试**

测试旧字符串数组读取为对象且 `completed=false`，新对象 round-trip，保存待办不会改变摘要、关键要点和来源 revision。

```rust
assert_eq!(detail.summary.unwrap().action_items[0].text, "旧待办");
assert!(!detail.summary.unwrap().action_items[0].completed);
```

- [ ] **Step 2: 运行测试确认 RED**

Run: `cargo test project_store::tests`

Expected: FAIL，`ActionItem` 不存在。

- [ ] **Step 3: 最小实现**

实现自定义兼容反序列化：字符串生成 UUID 对象，对象保持 ID。AI 返回字符串数组后在 Rust 转为对象。新增项目命令校验 ID 唯一、文字非空并原子保存完整总结，返回最新总结。

- [ ] **Step 4: 验证 GREEN**

Run: `cargo test project_store::tests ai::tests`

Expected: PASS。

- [ ] **Step 5: 提交**

```bash
git add src-tauri/src/model.rs src-tauri/src/ai.rs src-tauri/src/project_store.rs src-tauri/src/commands.rs src-tauri/src/main.rs src/types/project.ts src/lib/tauri.ts
git commit -m "feat: persist editable action items"
```

### Task 6: 待办原地编辑 UI

**Files:**
- Create: `src/components/EditableActionItems.tsx`
- Create: `src/lib/actionItems.ts`
- Modify: `src/pages/ProjectDetailPage.tsx`
- Test: `tests/action-items.test.ts`

**Interfaces:**
- Consumes: `ActionItem[]` 与 `projectActionItemsSave`。
- Produces: `EditableActionItems`，以及纯函数 `updateActionItem/addActionItem/removeActionItem/toggleActionItem`。

- [ ] **Step 1: 写失败测试**

```ts
it("updates, completes and removes action items without losing ids", () => {
  const item = { id: "a", text: "处理合同", completed: false };
  expect(updateActionItem([item], "a", "确认合同")[0]).toEqual({ ...item, text: "确认合同" });
  expect(toggleActionItem([item], "a")[0].completed).toBe(true);
  expect(removeActionItem([item], "a")).toEqual([]);
});
```

- [ ] **Step 2: 运行测试确认 RED**

Run: `npx vitest run tests/action-items.test.ts`

Expected: FAIL，模块不存在。

- [ ] **Step 3: 最小实现**

创建始终可编辑的无边框单行 input。focus 保存原值；blur/Enter 调用保存；Esc 恢复；新空行 blur 丢弃；复选框立即切换并保存；删除按钮悬停出现。保存失败保留输入并显示错误，父页面用返回的总结更新 detail。

- [ ] **Step 4: 验证 GREEN**

Run: `npx vitest run tests/action-items.test.ts`

Run: `npm run typecheck`

Expected: PASS。

- [ ] **Step 5: 提交**

```bash
git add src/components/EditableActionItems.tsx src/lib/actionItems.ts src/pages/ProjectDetailPage.tsx tests/action-items.test.ts
git commit -m "feat: edit AI action items inline"
```

### Task 7: 转录与总结文本复制

**Files:**
- Create: `src/lib/textSelection.ts`
- Modify: `src/components/TranscriptList.tsx`
- Modify: `src/pages/ProjectDetailPage.tsx`
- Test: `tests/text-selection.test.ts`

**Interfaces:**
- Produces: `selectedText(selection?: Selection | null): string`、`hasTextSelection` 和段落复制处理。
- Consumes: `navigator.clipboard.writeText`、现有 `onSeek` 和消息提示。

- [ ] **Step 1: 写失败测试**

```ts
it("treats only non-empty trimmed selections as active", () => {
  expect(hasTextSelection({ toString: () => " 选中文字 " } as Selection)).toBe(true);
  expect(hasTextSelection({ toString: () => "   " } as Selection)).toBe(false);
});
```

- [ ] **Step 2: 运行测试确认 RED**

Run: `npx vitest run tests/text-selection.test.ts`

Expected: FAIL，模块不存在。

- [ ] **Step 3: 最小实现**

把 TranscriptList 的整行 button 改为时间戳按钮加可选择正文区域。正文单击无选区时仍 seek；拖选后不 seek；双击复制整段文字并回调提示。文本区域加 `select-text`。对总结卡片和待办允许选择；保留复制全文。右键不阻止 WebView 原生菜单，若无选区且在转录段落上则提供“复制整段”的轻量菜单。

- [ ] **Step 4: 验证 GREEN**

Run: `npx vitest run tests/text-selection.test.ts tests/useTranscription.test.ts`

Run: `npm run typecheck`

Expected: PASS。

- [ ] **Step 5: 提交**

```bash
git add src/lib/textSelection.ts src/components/TranscriptList.tsx src/pages/ProjectDetailPage.tsx tests/text-selection.test.ts
git commit -m "feat: support reliable transcript text copying"
```

### Task 8: 版本 0.1.6、全量验证与 Windows 安装包

**Files:**
- Modify: `package.json`
- Modify: `package-lock.json`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/Cargo.lock`
- Modify: `src-tauri/tauri.conf.json`
- Output: `outputs/SnapScribe_0.1.6_x64-setup.exe`

**Interfaces:**
- Consumes: Tasks 1-7 全部功能。
- Produces: 版本一致的 NSIS 安装包和 SHA-256。

- [ ] **Step 1: 更新版本清单**

五处正式版本全部改为 `0.1.6`，Cargo.lock 只修改 SnapScribe 自身 package 记录。

- [ ] **Step 2: 全量测试**

Run: `npm test -- --run`

Run: `npm run typecheck`

Run: `npm run build`

Run: `cargo test`

Run: `cargo check`

Expected: 全部退出码 0，无失败测试。

- [ ] **Step 3: 构建安装包**

Run: `npm run tauri build -- --verbose`

Expected: 生成 `src-tauri/target/release/bundle/nsis/SnapScribe_0.1.6_x64-setup.exe`。

- [ ] **Step 4: 复制并校验**

复制到 `outputs/SnapScribe_0.1.6_x64-setup.exe`，比较源和目标大小及 SHA-256，必须一致。

- [ ] **Step 5: 最终提交**

```bash
git add package.json package-lock.json src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/tauri.conf.json docs/superpowers/plans/2026-08-31-glm-move-editable-content.md
git commit -m "chore: release SnapScribe 0.1.6"
```

不自动 push 或创建 GitHub Release，除非用户另行要求。
