# SnapScribe AI 服务多模型设计

## 目标

将 AI 服务从设置页拆为一级页面，支持全局自定义指令、多模型配置、拖动排序、顺序故障转移、运行状态更新，以及未配置模型时从 AI 总结自动跳转。

## 产品范围

- 一级导航顺序：`首页 / 文件库 / AI 服务 / 设置`。
- 设置页只保留文件存储与关于。
- AI 服务页包含自定义指令卡片和模型列表卡片。
- 模型列表第一条已启用模型自动成为默认模型；默认状态不单独持久化。
- 用户可添加、编辑、启用、停用、删除、测试和拖动排序模型。
- 添加与编辑使用同一个模态框。
- 当前版本只实现 OpenAI Chat Completions 兼容协议，但 Base URL、模型 ID、认证方式和请求路径可自定义。
- 自定义指令加入每次 AI 请求；强制输出格式约束始终置于最后。
- 没有已启用模型时，项目 AI 总结页跳转到 AI 服务模型列表并保留返回项目上下文。

## 请求与故障转移

每次请求读取已启用模型并按 `order` 升序建立不可变快照。每个模型最多尝试一次，成功后立即停止。用户主动取消立即停止；其他不能产生有效总结的错误会更新模型状态并继续下一个模型。

状态分类：

- `untested`
- `available`
- `timeout`
- `auth-failed`（HTTP 401/403）
- `rate-limited`（HTTP 429）
- `service-error`（HTTP 5xx 或网络错误）
- `config-error`（URL、模型、HTTP 4xx 等配置问题）
- `invalid-response`（响应或总结 JSON 无法解析）

状态仅反映最近一次结果，不自动停用、不自动改序。全部失败时返回每个模型的失败原因，已有总结不被覆盖。

## 数据与密钥

AI 配置独立保存为 `ai-service.json`：

```ts
interface AIServiceConfig {
  schemaVersion: 1;
  customInstruction: string;
  models: AIModelConfig[];
}

interface AIModelConfig {
  id: string;
  name: string;
  protocol: "openai-chat";
  baseUrl: string;
  modelId: string;
  endpointMode: "auto" | "full-url" | "custom-path";
  customPath?: string;
  authType: "bearer" | "x-api-key" | "custom-header" | "none";
  authHeader?: string;
  timeoutSecs: number;
  enabled: boolean;
  order: number;
  hasApiKey: boolean;
  lastStatus: AIModelRuntimeStatus;
}
```

API Key 不进入 JSON。每个模型使用独立 DPAPI 加密文件，文件名由模型 UUID 派生。模型删除时删除对应密钥。前端读取模型时只得到 `hasApiKey`。

旧 `settings.json` 中完整的单模型配置自动迁移为名称“现有模型”的第一条启用模型，并迁移旧 DPAPI 密钥；迁移完成后保留旧字段兼容读取，但设置页不再编辑旧字段。

## 页面与路由

新增 `ai-service` 一级路由。路由可携带：

```ts
{
  page: "ai-service";
  focus?: "models";
  returnToProjectId?: string;
}
```

AI 服务页保存模型后，如果存在 `returnToProjectId`，显示“返回项目生成总结”，不自动产生付费请求。

## 交互

- 自定义指令使用查看/编辑双态，保存按钮显式提交。
- 模型拖动使用原生 HTML5 Drag and Drop；显示紫色插入线，落下后立即保存顺序。
- 每行同时提供测试、编辑、启停和删除；删除需要二次确认。
- 模态框基础字段：名称、Base URL、API Key、模型 ID、启用。
- 高级字段：请求地址模式、自定义路径、认证方式、认证 Header、超时。
- 测试连接不保存模型；保存不强制测试成功。

## 版本与交付

- 正式版本统一升级到 `0.1.4`：`package.json`、`package-lock.json`、`Cargo.toml`、`Cargo.lock`、`tauri.conf.json`。
- 运行前端测试、类型检查、Rust 测试、格式检查和生产构建。
- 最终运行完整 Tauri NSIS 构建，生成 `SnapScribe_0.1.4_x64-setup.exe`，复制到 `outputs` 并校验 SHA-256。
- 不提交、不推送。
