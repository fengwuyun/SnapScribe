# SnapScribe

本地 Windows 桌面工具：把视频 / 录音拖进来，自动转成带时间戳的文字。完全本地运行——不联网、不登录、不需要 API Key，也不要求用户安装 Python、FFmpeg 或下载模型。

技术栈：Tauri 2 · React 18 · TypeScript · Tailwind CSS · Rust · FFmpeg · funasr-llamacpp（SenseVoiceSmall GGUF q8 + FSMN-VAD）。

## 核心特性

- 拖拽或点击导入 MP4 / MOV / MKV / MP3 / M4A / WAV
- **分段渐进式转录**：FFmpeg 一次完成「提取音轨 → 16kHz 单声道 PCM → 按 60s 切片」，Rust 逐段调用 SenseVoice 识别，每段完成立即推送到界面实时追加渲染
- 进度按已转写时长实时推进；可随时取消（终止子进程并清理临时文件）
- 带时间戳的结果列表，点击时间戳跳转播放位置（内置播放器）
- 复制全文 / 导出 TXT / 导出 SRT（UTF-8）
- 历史 TXT 管理：每次转写完成自动保存一份到历史目录，支持预览 / 重命名 / 删除

## 开发

环境要求：Node ≥ 20、Rust（MSVC）、WebView2。

```bash
npm install            # 安装前端依赖
node scripts/download-models.mjs      # 下载 ASR 模型到 src-tauri/resources/models/（约 256MB）
node scripts/download-ffmpeg.mjs      # 下载 ffmpeg/ffprobe 到 src-tauri/resources/bin/
node scripts/download-asr-runtime.mjs # 下载并校验官方 SenseVoice Windows x64 运行时
npm run tauri dev      # 启动开发实例
```

模型来源：[SenseVoiceSmall-GGUF](https://hf-mirror.com/FunAudioLLM/SenseVoiceSmall-GGUF)（q8）与 [fsmn-vad-GGUF](https://hf-mirror.com/FunAudioLLM/fsmn-vad-GGUF)。网络受限时脚本默认走 hf-mirror.com，可用参数覆盖 base URL。

运行时定位顺序（见 `src-tauri/src/runtime.rs`）：打包资源目录 → 可执行文件同级目录 → 开发期仓库内 `src-tauri/resources/bin`、`funasr-llamacpp-windows-x64/`、`models/` → 系统 PATH（仅 ffmpeg/ffprobe）。缺资源时报错并逐项列出缺失项。

## 测试与验证

```bash
npm test               # 前端单元测试（vitest）
npx tsc --noEmit       # 类型检查
cargo test             # Rust 单元测试（在 src-tauri/ 下）
npm run tauri dev      # 手工冒烟
```

`testdata/` 内含 SAPI 合成的中文音视频样本，可用于全链路手工回归。

## 打包

```bash
npm run tauri build
```

NSIS 安装包内置 ffmpeg.exe、ffprobe.exe、llama-funasr-sensevoice.exe 与两个 GGUF 模型，最终用户零配置离线使用。

## 文档

- `DEVELOPMENT_SPEC.md` —— 产品目标、边界、管线契约（唯一事实源）
- `UI_DESIGN_SPEC.md` —— Design Tokens 与组件视觉规范
- `AGENTS.md` —— 工程规则
- `docs/architecture.md`、`docs/decisions/` —— 架构说明与技术决策记录
