# ADR-001: 采用 funasr-llamacpp 替代 sherpa-onnx 作为本地推理运行时

状态：已接受 · 日期：2026-07（开发期）

## Decision

ASR 推理使用工作区自带的 `funasr-llamacpp-windows-x64` 预编译包（`llama-funasr-sensevoice.exe`），模型为 SenseVoiceSmall GGUF q8 + FSMN-VAD GGUF。

## Context

开发文档原定 sherpa-onnx + SenseVoiceSmall INT8。工作区实际提供的是 llama.cpp 版 FunASR 预编译 Windows x64 工具集，CLI 已实测可用；sherpa-onnx 运行时与对应模型均不存在。

## Reason

同一识别模型（SenseVoiceSmall）；GGUF q8 量化与 INT8 同级；零 Python、单可执行文件子进程调用即可集成；免去 sherpa-onnx 动态库（onnxruntime.dll 等）的获取、打包与版本匹配工作。

## Trade-offs

- 每次调用冷启动加载模型（约数百 MB 内存映射）；若实测逐段开销显著，需演进为常驻子进程协议。
- 输出契约依赖该 CLI 的 stdout 格式（--srt），更换运行时需改 asr.rs 一处适配层。
