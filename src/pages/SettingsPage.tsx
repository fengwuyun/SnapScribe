import { useEffect, useState } from "react";
import { CheckCircle2, ExternalLink, Github, XCircle } from "lucide-react";
import { Button } from "@/components/ui/Button";
import * as ipc from "@/lib/tauri";
import type { AboutInfo, AppSettings } from "@/types/project";

export function SettingsPage() {
  const [draft, setDraft] = useState<AppSettings | null>(null);
  const [saving, setSaving] = useState(false);
  const [testing, setTesting] = useState(false);
  const [message, setMessage] = useState("");
  const [connectionState, setConnectionState] = useState<{ status: "idle" | "testing" | "success" | "failure"; message: string }>({ status: "idle", message: "" });
  const [about, setAbout] = useState<AboutInfo | null>(null);

  useEffect(() => {
    void Promise.all([ipc.settingsGet(), ipc.aboutGet()]).then(([settings, info]) => { setDraft(settings); setAbout(info); }).catch((error) => setMessage(String(error)));
  }, []);

  if (!draft) return <p className={message ? "text-sm text-error" : "text-sm text-text-secondary"}>{message || "正在加载设置…"}</p>;

  async function save() {
    setSaving(true);
    setMessage("");
    try {
      const saved = await ipc.settingsSave(draft!);
      setDraft(saved);
      setMessage("设置已保存");
    } catch (error) {
      setMessage(error instanceof Error ? error.message : String(error));
    } finally {
      setSaving(false);
    }
  }

  async function testConnection() {
    setTesting(true);
    setConnectionState({ status: "testing", message: "正在连接…" });
    try {
      const result = await ipc.aiTestConnection(draft!);
      setConnectionState({ status: result.ok ? "success" : "failure", message: result.message });
    } catch (error) {
      setConnectionState({ status: "failure", message: error instanceof Error ? error.message : String(error) });
    } finally {
      setTesting(false);
    }
  }

  function updateAI(next: Partial<AppSettings["ai"]>) {
    setDraft({ ...draft!, ai: { ...draft!.ai, ...next } });
    setConnectionState({ status: "idle", message: "" });
  }

  async function chooseDataRoot() {
    const path = await ipc.selectDirectory();
    if (path) setDraft({ ...draft!, dataRoot: path });
  }

  return <div className="mx-auto max-w-[860px]">
    <h1 className="text-2xl font-bold">设置</h1>
    <div className="mt-6 grid gap-6">
      <section className="rounded-lg border border-line bg-surface p-6">
        <h2 className="text-lg font-semibold">文件存储</h2>
        <label className="mt-5 block text-sm font-medium">转录数据保存位置</label>
        <div className="mt-2 flex gap-2"><input value={draft.dataRoot} onChange={(event) => setDraft({ ...draft, dataRoot: event.target.value })} className="h-10 min-w-0 flex-1 rounded-md border border-line px-3 text-sm outline-none focus:border-primary" /><Button variant="secondary" onClick={() => void chooseDataRoot()}>选择位置</Button></div>
        <p className="mt-2 text-xs text-text-tertiary">更改后点击保存，现有项目会复制到新位置，旧位置保留。</p>
        <label className="mt-5 block text-sm font-medium">导入音视频策略
          <select value={draft.importStrategy} onChange={(event) => setDraft({ ...draft, importStrategy: event.target.value as AppSettings["importStrategy"] })} className="mt-2 h-10 w-full rounded-md border border-line bg-surface px-3 text-sm">
            <option value="reference">仅引用原文件（默认 / 推荐）</option>
            <option value="copy">复制到 SnapScribe</option>
          </select>
        </label>
        <p className="mt-3 text-xs text-text-tertiary">软件录音默认保存，可在项目详情中单独删除，不影响文本和总结。</p>
      </section>
      <section className="rounded-lg border border-line bg-surface p-6">
        <h2 className="text-lg font-semibold">AI 服务</h2>
        <div className="mt-5 grid gap-4">
          <label className="text-sm font-medium">服务类型<input value="OpenAI Compatible" disabled className="mt-2 h-10 w-full rounded-md border border-line bg-bg px-3 text-sm" /></label>
          <label className="text-sm font-medium">API Base URL<input value={draft.ai.baseUrl} onChange={(event) => updateAI({ baseUrl: event.target.value })} placeholder="https://api.openai.com/v1" className="mt-2 h-10 w-full rounded-md border border-line px-3 text-sm outline-none focus:border-primary" /></label>
          <label className="text-sm font-medium">API Key
            <input type="password" value={draft.apiKey ?? ""} onChange={(event) => { setDraft({ ...draft, apiKey: event.target.value, clearApiKey: false }); setConnectionState({ status: "idle", message: "" }); }} placeholder={draft.ai.hasApiKey ? "已保存，留空保持不变" : "输入 API Key"} className="mt-2 h-10 w-full rounded-md border border-line px-3 text-sm outline-none focus:border-primary" />
          </label>
          {draft.ai.hasApiKey && <label className="flex items-center gap-2 text-xs text-text-secondary"><input type="checkbox" checked={draft.clearApiKey ?? false} onChange={(event) => setDraft({ ...draft, clearApiKey: event.target.checked, apiKey: event.target.checked ? "" : draft.apiKey })} />保存时清除已存 API Key</label>}
          <label className="text-sm font-medium">模型名称<input value={draft.ai.model} onChange={(event) => updateAI({ model: event.target.value })} placeholder="gpt-4o-mini" className="mt-2 h-10 w-full rounded-md border border-line px-3 text-sm outline-none focus:border-primary" /></label>
          <div className="flex min-h-10 items-center gap-3">
            <Button variant="secondary" loading={testing} onClick={() => void testConnection()}>测试连接</Button>
            {connectionState.status === "testing" && <span className="text-xs font-medium text-text-secondary">{connectionState.message}</span>}
            {connectionState.status === "success" && <span className="flex items-center gap-1.5 text-xs font-semibold text-success"><CheckCircle2 className="size-4" />{connectionState.message}</span>}
            {connectionState.status === "failure" && <span className="flex min-w-0 items-center gap-1.5 text-xs font-semibold text-error"><XCircle className="size-4 shrink-0" /><span className="truncate" title={connectionState.message}>{connectionState.message}</span></span>}
            {connectionState.status === "idle" && <span className="text-xs text-text-tertiary">测试不会自动保存设置。</span>}
          </div>
        </div>
      </section>
      {about && <section className="rounded-lg border border-line bg-surface p-6">
        <h2 className="text-lg font-semibold">关于 SnapScribe</h2>
        <div className="mt-5 grid gap-4 sm:grid-cols-2">
          <div><p className="text-xs text-text-tertiary">当前版本</p><p className="mt-1 font-mono text-sm font-semibold">v{about.version}</p></div>
          <div><p className="text-xs text-text-tertiary">版本时间</p><p className="mt-1 font-mono text-sm font-semibold">{about.buildTime}</p></div>
        </div>
        <button type="button" onClick={() => void ipc.openRepository().catch((error) => setMessage(String(error)))} className="mt-5 flex w-full items-center gap-3 rounded-md border border-line px-4 py-3 text-left transition-colors duration-150 hover:border-primary hover:bg-primary-soft focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary">
          <span className="flex size-9 items-center justify-center rounded-md bg-[#111111] text-white"><Github className="size-5" /></span>
          <span className="min-w-0 flex-1"><span className="block text-sm font-semibold">GitHub 仓库</span><span className="mt-0.5 block truncate text-xs text-text-tertiary">{about.repositoryUrl}</span></span>
          <ExternalLink className="size-4 text-text-tertiary" />
        </button>
      </section>}
    </div>
    <div className="mt-6 flex items-center justify-end gap-4">{message && <span className="text-sm text-text-secondary">{message}</span>}<Button loading={saving} onClick={() => void save()}>保存设置</Button></div>
  </div>;
}
