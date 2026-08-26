# Codex 开发 PC 端软件推荐方案

> 适用场景：使用 Codex 主导开发 Windows 桌面应用，技术栈可包含 Tauri、React、Rust、FFmpeg、本地 AI 模型等。  
> 核心目标：减少返工、降低上下文漂移、保证 UI 与架构一致、让 Codex 的每次改动都可验证、可回退。

---

# 1. 总体原则

推荐采用：

```text
规格驱动
+
小任务执行
+
强约束
+
每步验证
+
小步提交
```

完整流程：

```text
需求
  ↓
Spec
  ↓
Plan
  ↓
Task
  ↓
实现
  ↓
测试
  ↓
运行验证
  ↓
检查 Diff
  ↓
Commit
  ↓
下一个 Task
```

不要直接让 Codex：

```text
“根据开发文档把整个软件做完”
```

原因：

- 容易中途偏离需求
- UI 风格逐步失控
- 架构容易过度设计
- Bug 难定位
- 出问题后难回退
- 后半程容易忘记前半程约束

---

# 2. 推荐项目结构

```text
project/
├── AGENTS.md
├── README.md
├── DEVELOPMENT_SPEC.md
├── UI_DESIGN_SPEC.md
│
├── docs/
│   ├── architecture.md
│   ├── decisions/
│   └── plans/
│
├── src/
├── src-tauri/
└── tests/
```

其中最重要的是三个文件：

| 文件 | 作用 |
|---|---|
| `AGENTS.md` | Codex 长期执行规则 |
| `DEVELOPMENT_SPEC.md` | 产品目标、功能边界、技术架构 |
| `UI_DESIGN_SPEC.md` | UI Tokens、组件、状态、视觉约束 |

原则：

> `AGENTS.md` 放“必须长期遵守的规则”，详细设计放独立文档，不要把所有内容都塞进去。

---

# 3. AGENTS.md 推荐结构

建议控制在 100～200 行以内。

示例：

```md
# Project Instructions

## Goal

Build a local Windows desktop application.

Priorities:

1. Simplicity
2. Stability
3. Low maintenance
4. UI consistency

## Required Docs

Before implementing features, read:

- DEVELOPMENT_SPEC.md
- UI_DESIGN_SPEC.md

These are the source of truth.

## Tech Stack

- Tauri 2
- React
- TypeScript
- Tailwind CSS
- shadcn/ui
- Lucide Icons
- Rust

Do not introduce alternative frameworks without explicit approval.

## Architecture

React handles:

- UI
- interaction
- presentation state

Rust handles:

- filesystem
- native capabilities
- media processing
- model inference
- exports

Do not move native processing logic into React.

## UI Constraints

Strictly follow UI_DESIGN_SPEC.md.

Do not introduce:

- new brand colors
- new radius values
- new shadow systems
- another icon library
- gradients
- glassmorphism
- dashboard-style UI

## Product Constraints

Do not add unrequested features.

Do not introduce:

- authentication
- cloud sync
- database
- multiple models
- plugin systems
- model managers

unless explicitly requested.

## Engineering Rules

- Prefer YAGNI.
- Prefer simple implementation over extensibility.
- Do not refactor unrelated code.
- Keep files focused.
- Add dependencies only when necessary.
- Never silently change product behavior.

## Verification

Before claiming completion:

1. Run relevant tests.
2. Run TypeScript typecheck.
3. Run Rust checks/tests when relevant.
4. Run frontend build.
5. Run Tauri build/check when relevant.
6. Inspect git diff.
7. Compare against acceptance criteria.

Never claim success without fresh verification evidence.
```

---

# 4. 模型分工

建议不要频繁切模型。

## 默认策略

| 场景 | 推荐模型 |
|---|---|
| 日常开发主力 | GPT-5.6 Terra |
| 架构设计 / 难 Bug / 大范围重构 | GPT-5.6 Sol |
| 小改动 / 文案 / CSS / 配置 | GPT-5.6 Luna |

推荐比例：

```text
Terra：约 80%
Sol：约 15%
Luna：约 5%
```

---

# 5. 什么时候用 Terra

默认使用 Terra。

适合：

- 普通功能开发
- React 页面
- Tauri Command
- Rust 业务逻辑
- 文件处理
- 状态管理
- 测试
- 组件开发
- 一般 Bug 修复

原则：

> 没有明确理由，就不要切模型。

---

# 6. 什么时候切 Sol

出现以下情况再切：

```text
连续修改 2～3 次仍没解决
复杂 Rust 生命周期
FFI / DLL
线程 / async
任务取消
复杂架构问题
跨模块重构
难以复现的 Bug
```

使用方式：

```text
先让 Sol 做诊断
↓
确定 root cause
↓
给出最小修改方案
↓
再执行
```

不要一上来就让 Sol 大面积重写。

---

# 7. Luna 使用场景

适合：

- 改按钮文案
- README
- aria-label
- rename
- 简单 Tailwind
- 调整 padding
- 小配置
- 很简单的测试补充

不要用 Luna 处理复杂架构问题。

---

# 8. 标准功能开发流程

每一个功能固定走：

```text
1. Read Spec
2. Inspect Code
3. Plan
4. Review Plan
5. Implement
6. Test
7. Run App
8. Review Diff
9. Commit
```

---

# 9. 第一步：先 Plan，不写代码

例如实现拖拽文件：

```text
Read:

- AGENTS.md
- DEVELOPMENT_SPEC.md
- UI_DESIGN_SPEC.md

We are implementing file import.

Requirements:

- Drag MP4 / MOV / MKV / MP3 / M4A / WAV into the window.
- Clicking DropZone opens native file picker.
- MVP supports only one file.
- Show FileItem after selection.
- Do not implement transcription yet.
- Follow existing UI tokens exactly.

First inspect the repository.

Do not modify files.

Produce a plan containing:

1. files to create/change
2. interfaces
3. state changes
4. tests
5. acceptance criteria
```

确认计划没问题后再执行。

---

# 10. Task 粒度

一个 Task 应该满足：

```text
可以独立实现
可以独立测试
可以独立 Review
可以独立 Commit
```

不推荐：

```text
实现完整转写功能
```

推荐：

```text
Task 1
搭建 Tauri + React

Task 2
建立 Design Tokens

Task 3
实现 DropZone

Task 4
实现 FileItem

Task 5
Rust 获取媒体信息

Task 6
FFmpeg 转换音频

Task 7
SenseVoice CLI POC

Task 8
sherpa-onnx 集成

Task 9
Tauri transcribe command

Task 10
转写进度

Task 11
Transcript UI

Task 12
播放器

Task 13
时间戳跳转

Task 14
TXT 导出

Task 15
SRT 导出

Task 16
错误处理

Task 17
Windows 打包
```

---

# 11. Prompt 推荐模板

不要：

```text
帮我增加播放器。
```

推荐：

```text
## Task

Implement transcript audio playback.

## Context

Read:

- AGENTS.md
- DEVELOPMENT_SPEC.md
- UI_DESIGN_SPEC.md

## Scope

Implement:

- play
- pause
- seek
- current time
- duration

Do not implement:

- playback speed
- volume persistence
- playlists

## UI

Follow Audio Player section in UI_DESIGN_SPEC.md.

Do not introduce new colors or components.

## Files

Prefer modifying:

- src/components/AudioPlayer.tsx
- src/hooks/useAudioPlayer.ts

Do not modify unrelated files.

## Acceptance Criteria

- Play starts from current position.
- Pause preserves current position.
- Seek updates audio position.
- UI updates current time.
- TypeScript check passes.
- Existing tests pass.

Inspect code first.

Explain proposed changes before editing.
```

Prompt 尽量像 GitHub Issue，而不是聊天指令。

---

# 12. 每个 Prompt 必须包含什么

推荐至少包含：

```text
Task
Context
Scope
Out of Scope
Files
Constraints
Acceptance Criteria
Verification
```

---

# 13. Scope 必须明确

每次任务明确：

```text
做什么
+
不做什么
```

示例：

```text
Implement:
- TXT export
- UTF-8 encoding
- default filename

Do not implement:
- PDF
- DOCX
- cloud export
- export history
```

这样能明显减少 Codex 顺手扩展需求。

---

# 14. UI 开发约束

UI 开发采用：

> specification-driven，而不是 exploratory。

固定提示：

```text
UI is specification-driven, not exploratory.

Do not redesign the page.

Do not introduce:

- new colors
- gradients
- shadows
- radius values
- icon libraries
- decorative UI

Strictly follow UI_DESIGN_SPEC.md.

Prefer existing components.

If a visual requirement is unclear:

1. infer from UI_DESIGN_SPEC.md
2. inspect existing components
3. preserve current visual language
```

---

# 15. UI 修改流程

推荐：

```text
确认页面结构
↓
确认使用哪些现有组件
↓
实现
↓
运行界面
↓
截图检查
↓
与 UI_DESIGN_SPEC.md 对比
↓
调整
```

Codex 不应自行重新设计页面。

---

# 16. 测试策略

不要走两个极端：

```text
完全不测试
```

或者：

```text
所有东西都写大量测试
```

推荐三层：

| 类型 | 方法 |
|---|---|
| 核心逻辑 | 自动测试 |
| Rust / FFmpeg / 模型 | 集成测试 |
| UI 视觉 | 手工 + 截图 |

---

# 17. 优先自动测试的内容

重点测试：

```text
时间格式转换
SRT 生成
文件格式校验
状态机
媒体信息解析
错误处理
FFmpeg command 构造
Transcript 数据转换
导出逻辑
```

---

# 18. 不必重点单测的内容

例如：

```text
按钮是不是 #4353FF
padding 是不是 12px
图标是不是 Lucide
```

这些更适合视觉检查。

---

# 19. TDD 推荐方式

功能或 Bug 修复推荐：

```text
RED
↓
写失败测试
↓
运行并确认正确失败

GREEN
↓
写最小实现
↓
运行并确认通过

REFACTOR
↓
整理代码
↓
再次运行测试
```

不要：

```text
先写大量代码
↓
最后补几个测试
```

---

# 20. Bug 修复流程

不要：

```text
修一下
↓
不行
↓
再改
↓
还不行
↓
继续猜
```

推荐：

```text
停止修改
↓
复现
↓
收集证据
↓
定位 Root Cause
↓
写 Regression Test
↓
最小修复
↓
验证
```

Prompt：

```text
Do not modify code yet.

Investigate this bug systematically.

Symptom:
...

Expected:
...

Actual:
...

First:

1. reproduce it
2. inspect relevant code
3. identify root cause
4. explain evidence
5. propose the smallest fix

Do not patch symptoms.
```

---

# 21. 连续修复失败时怎么办

经验规则：

```text
第一次失败
→ 继续检查

第二次失败
→ 停止修改，重新诊断

第三次失败
→ 换 Sol + Root Cause 分析
```

不要让 Codex无限叠补丁。

---

# 22. Verification 是硬约束

Codex 完成任何 Task 前必须验证。

统一要求：

```text
Before declaring completion:

1. Run relevant tests.
2. Run TypeScript typecheck.
3. Run cargo test if relevant.
4. Run cargo check.
5. Run frontend build.
6. Run relevant Tauri check/build.
7. Inspect git diff.
8. Compare implementation against acceptance criteria.

Report:

- commands executed
- pass/fail
- remaining issues

Do not claim completion based only on code inspection.
```

原则：

> 没有运行结果，就不能说“完成”。

---

# 23. 推荐验证命令

按项目实际脚本调整。

前端：

```bash
npm test
npm run typecheck
npm run build
```

Rust：

```bash
cargo test
cargo check
```

Tauri：

```bash
npm run tauri dev
npm run tauri build
```

不要每次所有命令全跑。

只跑与当前 Task 有关的命令，但阶段性必须执行完整验证。

---

# 24. Git 策略

推荐：

```text
一个 Task
≈
一个 Commit
```

Commit 示例：

```text
feat: scaffold tauri application
feat: add file drop zone
feat: read media metadata
feat: add ffmpeg conversion
feat: integrate sensevoice runtime
feat: render transcript segments
feat: add subtitle export
fix: handle unsupported media files
```

避免：

```text
update
fix things
final
final2
```

---

# 25. Commit 前检查

每次提交前：

```bash
git status
git diff
```

确认：

- 没改无关文件
- 没引入新依赖
- 没改变未要求的逻辑
- 没破坏 UI Design Tokens
- 没遗留调试代码
- 没留下临时文件

---

# 26. 避免 Codex 过度设计

在 AGENTS.md 中固定：

```text
Prefer YAGNI.

Do not build extension points for hypothetical future requirements.

Do not add abstraction unless at least two real use cases require it.

Do not add:

- plugin systems
- provider factories
- generic repositories
- unnecessary service layers
- configurable model architectures

unless required by the current task.
```

---

# 27. 依赖管理

新增依赖必须满足：

```text
当前功能确实需要
+
现有技术栈无法简单解决
+
成熟稳定
+
维护成本可接受
```

Prompt 可加：

```text
Do not add dependencies unless necessary.

If adding a dependency:
explain why existing project capabilities are insufficient.
```

---

# 28. 文档同步

只有以下内容变化时更新 Spec：

```text
产品行为改变
技术架构改变
UI Design Token 改变
公共接口改变
重大技术决策
```

不要每个小改动都重写文档。

---

# 29. Architecture Decision Record

重大技术决策可以记录：

```text
docs/decisions/
```

例如：

```text
001-use-tauri.md
002-use-sensevoice.md
003-bundle-model-locally.md
```

内容保持简短：

```text
Decision
Context
Reason
Trade-offs
```

---

# 30. 推荐使用 Superpowers 工作流

推荐映射：

```text
需求不清
→ brainstorming

已有完整 Spec
→ writing-plans

功能开发
→ test-driven-development

出现 Bug
→ systematic-debugging

任务完成
→ verification-before-completion

复杂大任务
→ subagent-driven-development
```

---

# 31. 推荐日常工作流

每天开发时：

```text
打开 Codex
↓
选择一个 Task
↓
让 Codex 读 AGENTS.md + Spec
↓
只做 Plan
↓
确认 Plan
↓
实现
↓
测试
↓
运行软件
↓
人工体验
↓
git diff
↓
commit
↓
下一个 Task
```

---

# 32. 推荐完整项目流程

```text
DEVELOPMENT_SPEC.md
        +
UI_DESIGN_SPEC.md
        ↓
    AGENTS.md
        ↓
      Plan
        ↓
     Task 1
        ↓
   Terra 实现
        ↓
      Test
        ↓
     Verify
        ↓
  Manual Check
        ↓
    Git Diff
        ↓
     Commit
        ↓
     Task 2
        ↓
      ...
```

难问题：

```text
Terra 连续失败
        ↓
停止修改
        ↓
系统诊断
        ↓
GPT-5.6 Sol
        ↓
Root Cause
        ↓
最小修复
        ↓
验证
```

---

# 33. 对个人开发者最重要的控制点

你自己不需要承担大量编码。

重点负责四件事：

## 1. 定义 Spec

告诉 Codex：

```text
要什么
不要什么
什么叫完成
```

## 2. 控制 Task 大小

保证每次只实现一个明确能力。

## 3. 控制扩展

不允许 Codex自行：

```text
加功能
换框架
重构整个项目
重新设计 UI
加复杂抽象
```

## 4. 只接受验证结果

不能因为：

```text
“代码看起来没问题”
```

就算完成。

---

# 34. 最终推荐配置

```text
主模型：
GPT-5.6 Terra

复杂问题：
GPT-5.6 Sol

小任务：
GPT-5.6 Luna
```

开发方法：

```text
Spec
→ Plan
→ Small Task
→ TDD
→ Verify
→ Manual Check
→ Commit
```

长期约束：

```text
AGENTS.md
```

产品规格：

```text
DEVELOPMENT_SPEC.md
```

UI 规格：

```text
UI_DESIGN_SPEC.md
```

任务原则：

```text
一次只实现一个可验收能力
```

---

# 35. 一句话原则

> 把 Codex 当成执行能力很强、但必须明确边界的工程师：你负责规格、约束和验收，Codex负责实现。
