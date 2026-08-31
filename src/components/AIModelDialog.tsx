import { useEffect, useState } from "react";
import * as Dialog from "@radix-ui/react-dialog";
import { CheckCircle2, Eye, EyeOff, X, XCircle } from "lucide-react";
import { Button } from "@/components/ui/Button";
import { getApiKeyDisplayValue } from "@/lib/aiModelPresentation";
import { savedModelRetestDraft } from "@/lib/aiModelWorkflow";
import * as ipc from "@/lib/tauri";
import type { AIModelConfig, AIModelDraft, AIModelStatus } from "@/types/project";

function initialDraft(model: AIModelConfig | null): AIModelDraft {
  return model ? {
    id: model.id, name: model.name, protocol: model.protocol, baseUrl: model.baseUrl,
    modelId: model.modelId, endpointMode: model.endpointMode, customPath: model.customPath,
    authType: model.authType, authHeader: model.authHeader, timeoutSecs: model.timeoutSecs,
    enabled: model.enabled,
  } : {
    name: "", protocol: "openai-chat", baseUrl: "", modelId: "", endpointMode: "auto",
    authType: "bearer", timeoutSecs: 60, enabled: true,
  };
}

export function AIModelDialog({ open, model, onOpenChange, onSaved }: {
  open: boolean;
  model: AIModelConfig | null;
  onOpenChange: (open: boolean) => void;
  onSaved: () => void;
}) {
  const isPreset = model?.presetKey === "glm-4.7-flash";
  const [draft, setDraft] = useState<AIModelDraft>(() => initialDraft(model));
  const [advanced, setAdvanced] = useState(false);
  const [showKey, setShowKey] = useState(false);
  const [editingSavedKey, setEditingSavedKey] = useState(false);
  const [busy, setBusy] = useState(false);
  const [testing, setTesting] = useState(false);
  const [error, setError] = useState("");
  const [testStatus, setTestStatus] = useState<AIModelStatus | null>(null);

  useEffect(() => {
    if (open) {
      setDraft(initialDraft(model));
      setAdvanced(Boolean(model && (model.endpointMode !== "auto" || model.authType !== "bearer" || model.timeoutSecs !== 60)));
      setError(""); setTestStatus(null); setShowKey(false); setEditingSavedKey(false);
    }
  }, [open, model]);

  function update(next: Partial<AIModelDraft>) {
    setDraft((current) => ({ ...current, ...next }));
    setTestStatus(null);
  }

  async function test() {
    setTesting(true); setError("");
    try { setTestStatus(await ipc.aiModelTest(draft)); }
    catch (reason) { setError(String(reason)); }
    finally { setTesting(false); }
  }

  async function save() {
    setBusy(true); setError("");
    try {
      const saved = model ? await ipc.aiModelUpdate(model.id, draft) : await ipc.aiModelCreate(draft);
      const retestDraft = savedModelRetestDraft(draft, saved.id, testStatus);
      if (retestDraft) await ipc.aiModelTest(retestDraft);
      onSaved(); onOpenChange(false);
    } catch (reason) { setError(String(reason)); }
    finally { setBusy(false); }
  }

  const inputClass = "mt-2 h-10 w-full rounded-md border border-line bg-surface px-3 text-sm outline-none transition-colors focus:border-primary";
  return <Dialog.Root open={open} onOpenChange={(next) => !busy && onOpenChange(next)}>
    <Dialog.Portal>
      <Dialog.Overlay className="fixed inset-0 z-40 bg-black/25 backdrop-blur-[2px]" />
      <Dialog.Content className="fixed left-1/2 top-1/2 z-50 max-h-[88vh] w-[560px] max-w-[calc(100vw-32px)] -translate-x-1/2 -translate-y-1/2 overflow-y-auto rounded-xl border border-line bg-surface p-6 shadow-md outline-none">
        <div className="flex items-start justify-between"><div><Dialog.Title className="text-lg font-semibold">{model ? "编辑模型" : "添加模型"}</Dialog.Title><Dialog.Description className="mt-1 text-sm text-text-secondary">兼容 OpenAI Chat Completions 接口及本地兼容服务。</Dialog.Description></div><Dialog.Close asChild><button className="flex size-8 items-center justify-center rounded-md text-text-tertiary hover:bg-bg"><X className="size-4" /></button></Dialog.Close></div>
        <div className="mt-6 grid gap-4">
          <label className="text-sm font-medium">显示名称<input className={inputClass} readOnly={isPreset} value={draft.name} onChange={(event) => update({ name: event.target.value })} placeholder="例如：本地 Ollama" /></label>
          <label className="text-sm font-medium">API Base URL<input className={inputClass} readOnly={isPreset} value={draft.baseUrl} onChange={(event) => update({ baseUrl: event.target.value })} placeholder="http://localhost:11434/v1" /></label>
          <label className="text-sm font-medium">模型 ID<input className={inputClass} readOnly={isPreset} value={draft.modelId} onChange={(event) => update({ modelId: event.target.value })} placeholder="例如：qwen2.5:7b" /></label>
          {draft.authType !== "none" && <label className="text-sm font-medium">API Key
            <div className="relative"><input className={`${inputClass} pr-10`} type={showKey ? "text" : "password"} value={getApiKeyDisplayValue(Boolean(model?.hasApiKey), draft.apiKey, editingSavedKey)} onFocus={() => { if (model?.hasApiKey && !draft.apiKey) setEditingSavedKey(true); }} onBlur={() => { if (model?.hasApiKey && !draft.apiKey) setEditingSavedKey(false); }} onChange={(event) => { setEditingSavedKey(true); update({ apiKey: event.target.value || undefined, clearApiKey: false }); }} placeholder={model?.hasApiKey ? "已保存，留空保持不变" : "输入 API Key"} /><button type="button" onClick={() => setShowKey((value) => !value)} className="absolute right-2 top-3.5 flex size-7 items-center justify-center text-text-tertiary">{showKey ? <EyeOff className="size-4" /> : <Eye className="size-4" />}</button></div>
          </label>}
          {!isPreset && <button type="button" onClick={() => setAdvanced((value) => !value)} className="w-fit text-sm font-semibold text-primary">{advanced ? "收起高级设置" : "展开高级设置"}</button>}
          {!isPreset && advanced && <div className="grid gap-4 rounded-md bg-bg p-4 sm:grid-cols-2">
            <label className="text-sm font-medium">请求地址模式<select className={inputClass} value={draft.endpointMode} onChange={(event) => update({ endpointMode: event.target.value as AIModelDraft["endpointMode"] })}><option value="auto">自动拼接 /chat/completions</option><option value="full-url">Base URL 即完整地址</option><option value="custom-path">使用自定义路径</option></select></label>
            {draft.endpointMode === "custom-path" && <label className="text-sm font-medium">自定义路径<input className={inputClass} value={draft.customPath ?? ""} onChange={(event) => update({ customPath: event.target.value })} placeholder="chat/completions" /></label>}
            <label className="text-sm font-medium">认证方式<select className={inputClass} value={draft.authType} onChange={(event) => update({ authType: event.target.value as AIModelDraft["authType"] })}><option value="bearer">Bearer Token</option><option value="x-api-key">x-api-key</option><option value="custom-header">自定义 Header</option><option value="none">无需认证</option></select></label>
            {draft.authType === "custom-header" && <label className="text-sm font-medium">Header 名称<input className={inputClass} value={draft.authHeader ?? ""} onChange={(event) => update({ authHeader: event.target.value })} placeholder="Authorization" /></label>}
            <label className="text-sm font-medium">超时时间（秒）<input className={inputClass} type="number" min={5} max={300} value={draft.timeoutSecs} onChange={(event) => update({ timeoutSecs: Number(event.target.value) || 60 })} /></label>
          </div>}
          <label className="flex items-center gap-2 text-sm"><input type="checkbox" checked={draft.enabled} onChange={(event) => update({ enabled: event.target.checked })} />启用此模型</label>
        </div>
        <div className="mt-5 min-h-6">{error && <p className="text-sm text-error">{error}</p>}{testStatus && <p className={`flex items-center gap-2 text-sm font-semibold ${testStatus.status === "available" ? "text-success" : "text-error"}`}>{testStatus.status === "available" ? <CheckCircle2 className="size-4" /> : <XCircle className="size-4" />}{testStatus.message}</p>}</div>
        <div className="mt-4 flex justify-between"><Button variant="secondary" loading={testing} onClick={() => void test()}>测试连接</Button><div className="flex gap-3"><Button variant="secondary" disabled={busy} onClick={() => onOpenChange(false)}>取消</Button><Button loading={busy} onClick={() => void save()}>保存模型</Button></div></div>
      </Dialog.Content>
    </Dialog.Portal>
  </Dialog.Root>;
}
