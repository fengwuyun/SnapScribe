# 架构

## 模块职责

### 前端（src/）

| 模块 | 职责 |
|---|---|
| `pages/Home.tsx` | 单页面组装，持有 AppState 状态机 |
| `hooks/useTranscription.ts` | 状态机实现、Tauri 事件订阅、段落增量合并 |
| `components/*` | Header / DropZone / FileItem / TranscribeProgress / AudioPlayer / TranscriptList·Item / BottomActionBar / EmptyState / ErrorState / HistoryModal / Toast |
| `lib/tauri.ts` | invoke 封装（唯一 Tauri API 出口）|
| `lib/format.ts` | 时长格式化、SRT/TXT 文本生成辅助、文件大小格式化 |
| `types/transcript.ts` | 与 Rust 侧 serde 结构一一对应的 TS 类型 |

React 不解析 ASR 原始输出、不触碰文件系统；一切本地能力经 `lib/tauri.ts` → Rust Command。

### 后端（src-tauri/src/）

| 模块 | 职责 |
|---|---|
| `main.rs` | Tauri 启动、Command 注册、app_data 目录初始化 |
| `commands.rs` | Command 薄层：参数校验后委托具体模块 |
| `media.rs` | ffprobe 调用与 MediaInfo 解析 |
| `ffmpeg.rs` | ffmpeg 参数构造（转码+切片）、子进程执行、临时目录管理 |
| `asr.rs` | funasr CLI 参数构造、SRT 解析、时间戳偏移、TranscriptSegment 规范化 |
| `pipeline.rs` | 编排 preparing→splitting→transcribing→completed；事件发送；取消令牌；串行循环 |
| `export.rs` | TXT/SRT 内容生成与写出 |
| `history.rs` | 历史目录扫描/读取/重命名/删除 |

## 运行时定位

`resolve_runtime()`：打包资源目录优先，未打包时回退 PATH 的 ffmpeg/ffprobe、工作区 funasr 目录与 `models/`。定位失败在调用点报错（fail loud），不静默跳过。

## 关键数据流

1. **转录**：`start_transcription(path)` 返回 jobId 并 spawn 异步任务；进度/段落/完成/失败全部经事件通道推送（见 DEVELOPMENT_SPEC §5）。Command 本身不阻塞等待结果。
2. **取消**：`cancel_transcription(jobId)` 置位 AtomicBool → pipeline 在段间检查退出；当前 ASR 子进程被 kill；`Drop` 清理临时目录。
3. **导出**：前端把当前 segments JSON 传给 export 命令，Rust 生成文本并写盘。
4. **历史**：completed 时 Rust 自动写入 `app_data/transcripts/<名>-<时间戳>.txt`；列表来自文件系统实时扫描。

## 测试布局

- Rust：各模块 `#[cfg(test)]` 覆盖纯函数（SRT 解析、偏移、参数构造、TXT/SRT 生成、重命名规则）。
- 前端：vitest 覆盖 `lib/format.ts` 与 useTranscription 的 reducer 迁移。
- 集成：真实媒体文件全链路手工冒烟（见 DEVELOPMENT_SPEC §10）。
