import type { AIModelStatusKind } from "@/types/project";

export const SAVED_API_KEY_MASK = "******";

const AI_MODEL_STATUS_LABELS: Record<AIModelStatusKind, string> = {
  untested: "未测试",
  available: "连通",
  timeout: "请求超时",
  "auth-failed": "鉴权失败",
  "rate-limited": "已限流",
  "service-error": "服务异常",
  "config-error": "配置错误",
  "invalid-response": "响应无效",
};

export function getApiKeyDisplayValue(
  hasSavedKey: boolean,
  draftKey: string | undefined,
  editingSavedKey: boolean,
) {
  if (draftKey) return draftKey;
  return hasSavedKey && !editingSavedKey ? SAVED_API_KEY_MASK : "";
}

export function getAIModelStatusLabel(status: AIModelStatusKind) {
  return AI_MODEL_STATUS_LABELS[status];
}
