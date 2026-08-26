# DEVELOPMENT_SPEC

> SnapScribe 产品与技术规格（唯一事实源）。UI 视觉约束见 `UI_DESIGN_SPEC.md`。
> 来源：《本地视频录音转文字工具_开发文档》+ 已确认的开发方案 + 新增需求（切分渐进转录、历史 TXT 管理）。

---

## 1. 产品目标

Windows 本地视频 / 录音转文字桌面工具：把视频或录音拖入软件，自动转换成带时间戳的文字，支持复制、导出与历史管理。

核心目标：简单易用、界面现代统一、安装即用、完全本地运行（无登录 / API Key / 模型下载 / Python / FFmpeg 安装）、低维护成本。

## 2. 功能边界

### 2.1 必须实现

| 功能 | 优先级 |
|---|---|
| 拖拽导入 / 点击选择文件 | P0 |
| MP4 / MOV / MKV / MP3 / M4A / WAV 支持 | P0 |
| 文件信息展示（名称、大小、时长） | P0 |
| 开始转写 / 转写进度 / 取消转写 | P0 |
| **音频预切分**：转码为 16kHz 单声道 PCM 并按固定窗切片 | P0 |
| **分段转录**：逐段推理，每段完成立即推送前端 | P0 |
| **渐进式结果展示**：Transcript 列表实时追加渲染 | P0 |
| 时间戳展示 | P0 |
| 复制全文 / 导出 TXT / 导出 SRT | P0 |
| 错误提示（损坏 / 不支持文件不崩溃） | P0 |
| 点击时间戳跳转播放位置 / 播放暂停 | P1 |
| **历史 TXT 管理**：自动保存的转录文本列表 / 预览 / 重命名 / 删除 | P1 |

### 2.2 明确不做

登录注册、云同步、在线 API、AI 摘要润色翻译、说话人分离、实时录音转写、多模型切换、模型下载管理、自动更新、项目管理、数据库、团队协作。

历史管理的实现边界：应用数据目录下纯 `.txt` 文件 + 文件系统元数据，无元数据库、无标签、无搜索。

## 3. 技术架构

```text
React UI (WebView2)
  ↓ invoke / event
Tauri 2
  ↓
Rust
  ├── ffmpeg.rs   FFmpeg/ffprobe 子进程：媒体信息、转码 + 切片
  ├── asr.rs      funasr-llamacpp 子进程：SenseVoiceSmall GGUF q8 + FSMN-VAD
  ├── pipeline.rs 编排：切片 → 逐段转录 → 事件推送 → 清理；取消控制
  ├── export.rs   TXT / SRT 内容生成与写出
  └── history.rs  历史目录扫描 / 读 / 改名 / 删除
```

技术栈：Tauri 2 + React 18 + TypeScript + Tailwind CSS 3 + shadcn/ui 约定 + Lucide Icons + FFmpeg + funasr-llamacpp（替代原方案的 sherpa-onnx，见 ADR-001）。包管理使用 npm。

## 4. 核心链路（分段渐进转录）

```text
start_transcription(path)
  ↓ preparing        ffprobe 校验并读取 duration
  ↓ splitting        ffmpeg -i in -vn -ac 1 -ar 16000 -c:a pcm_s16le -f segment -segment_time 60 seg_%04d.wav
  ↓ transcribing     对每个 seg_i.wav 串行执行:
                     llama-funasr-sensevoice -m sensevoice-small-q8.gguf --vad fsmn-vad.gguf -a seg_i.wav --srt
                       → 解析 stdout SRT → 时间戳 += i×60s → TranscriptSegment[]
                       → emit "transcript://segments"（增量段落）
                       → emit "transcript://progress"
  ↓ completed        合并 TranscriptResult，写入历史目录 TXT，清理临时段文件
```

契约：

- 段严格串行转录；进度百分比 = 已转写秒数 / 总秒数，单调不减。
- 每段完成即刻发事件，禁止攒到最后一次性发送。
- `cancel_transcription` 终止当前子进程与循环，删除临时目录。
- 切割点可能落在词中间（每段最多影响首尾词），MVP 接受（ADR-002）。
- ASR 每段冷启动加载模型；若实测开销显著再评估常驻进程协议（接口不变）。

## 5. 数据结构与事件协议

```ts
interface TranscriptSegment { id: string; start: number; end: number; text: string }
interface TranscriptResult { fileName: string; duration: number; language?: string; segments: TranscriptSegment[] }

interface MediaInfo { fileName: string; sizeBytes: number; durationSecs: number; container: string }
interface HistoryEntry { fileName: string; sizeBytes: number; modifiedMs: number }
```

事件（`app.emit`，前端 `listen` 订阅）：

| 通道 | 载荷 | 说明 |
|---|---|---|
| `transcript://progress` | `{ stage:"preparing"\|"splitting"\|"transcribing", percent, processedSeconds, totalSeconds, segmentIndex, segmentCount }` | 进度 |
| `transcript://segments` | `{ jobId, segments: TranscriptSegment[] }` | 增量新完成的段落 |
| `transcript://completed` | `{ result: TranscriptResult }` | 全部完成 |
| `transcript://failed` | `{ message }` | 失败 |

## 6. 应用状态机

```ts
type AppState = "empty" | "ready" | "transcribing" | "completed" | "error";
```

禁止零散 Boolean。"切分中 / 转录中"是 `transcribing` 内部的 stage 展示，不新增顶层状态。

## 7. Tauri Commands

```ts
select_file(): string | null                    // 原生打开对话框
get_media_info(path): MediaInfo
start_transcription(path): string               // 返回 jobId，异步执行
cancel_transcription(jobId): void
export_txt(segmentsJson, path): void            // 另存对话框后调用
export_srt(segmentsJson, path): void
save_file_dialog(defaultName, ext): string | null
history_list(): HistoryEntry[]
history_read(fileName): string
history_rename(oldName, newName): string        // 返回最终名（重名自动加序号）
history_delete(fileName): void
history_dir(): string                           // 供"打开所在文件夹"展示路径
```

非法输入（路径逃逸 `..`、非法文件名字符）一律返回错误，不 panic。

## 8. 历史管理规格

- 目录：`app_data_dir/transcripts/`（`%APPDATA%/<bundle-id>/transcripts`）。
- 转写完成时自动保存一份全文 `<原文件名>-<yyyyMMdd-HHmmss>.txt`（UTF-8 无 BOM），同时保留手动导出到任意位置的流程。
- 列表项显示文件名、大小、修改时间；操作：预览（只读弹层 + 复制按钮）、重命名（内联输入，校验非法字符 `\ / : * ? " < > |` 与空名，重名追加 `-2` 序号）、删除（二次确认）。
- 不做搜索、排序切换、批量操作。

## 9. 资源打包

发布包内置：`ffmpeg.exe`、`ffprobe.exe`、`llama-funasr-sensevoice.exe`、`sensevoice-small-q8.gguf`、`fsmn-vad.gguf`。开发期通过运行时定位顺序解析：打包资源目录 → 系统 PATH（ffmpeg/ffprobe）→ 工作区 `models/` 与 `funasr-llamacpp-windows-x64/`（模型与 CLI）。最终用户零下载零配置。

## 10. 验收标准（MVP 完整路径）

安装打开 → 拖入 MP4/MP3/M4A/WAV → 正确识别文件 → 开始转写 → 显示明确进度且**段落文本随转写逐步出现** → 完成展示带时间戳全文 → 点时间戳跳转播放 → 复制全文 → 导出 TXT / SRT（中文正常、SRT 格式正确）→ 历史列表出现该记录且可预览/重命名/删除。全程不联网、不登录、不选模型、不配置参数。
