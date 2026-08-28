import { useEffect, useRef, useState } from "react";
import { CheckCircle2, CircleAlert, GripVertical, Pencil, Plus, Sparkles, Trash2 } from "lucide-react";
import { AIModelDialog } from "@/components/AIModelDialog";
import { DeleteConfirmationDialog } from "@/components/DeleteConfirmationDialog";
import { Button } from "@/components/ui/Button";
import * as ipc from "@/lib/tauri";
import type { AIModelConfig, AIModelStatusKind, AIServiceConfig } from "@/types/project";

const statusLabels: Record<AIModelStatusKind, string> = {
  untested: "未测试", available: "可用", timeout: "请求超时", "auth-failed": "鉴权失败",
  "rate-limited": "已限流", "service-error": "服务异常", "config-error": "配置错误", "invalid-response": "响应无效",
};

export function AIServicePage({ focusModels = false, returnToProjectId, onReturn }: { focusModels?: boolean; returnToProjectId?: string; onReturn: (projectId: string) => void }) {
  const [config, setConfig] = useState<AIServiceConfig | null>(null);
  const [instruction, setInstruction] = useState("");
  const [editing, setEditing] = useState(false);
  const [savingInstruction, setSavingInstruction] = useState(false);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [selected, setSelected] = useState<AIModelConfig | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<AIModelConfig | null>(null);
  const [deleting, setDeleting] = useState(false);
  const [message, setMessage] = useState("");
  const [draggingId, setDraggingId] = useState<string | null>(null);
  const modelsRef = useRef<HTMLElement | null>(null);

  async function load() {
    try { const next = await ipc.aiServiceGet(); setConfig(next); setInstruction(next.customInstruction); }
    catch (error) { setMessage(String(error)); }
  }

  useEffect(() => { void load(); }, []);
  useEffect(() => { if (config && focusModels) modelsRef.current?.scrollIntoView({ behavior: "smooth", block: "start" }); }, [config, focusModels]);

  async function saveInstruction() {
    setSavingInstruction(true); setMessage("");
    try { const next = await ipc.aiInstructionSave(instruction); setConfig(next); setEditing(false); setMessage("自定义指令已保存"); }
    catch (error) { setMessage(String(error)); }
    finally { setSavingInstruction(false); }
  }

  async function toggle(model: AIModelConfig) {
    try { await ipc.aiModelSetEnabled(model.id, !model.enabled); await load(); }
    catch (error) { setMessage(String(error)); }
  }

  async function moveTo(targetId: string, placeAfter: boolean) {
    if (!config || !draggingId || draggingId === targetId) return;
    const ids = config.models.map((model) => model.id).filter((id) => id !== draggingId);
    const targetIndex = ids.indexOf(targetId);
    ids.splice(targetIndex + (placeAfter ? 1 : 0), 0, draggingId);
    setDraggingId(null);
    try { setConfig(await ipc.aiModelReorder(ids)); }
    catch (error) { setMessage(String(error)); await load(); }
  }

  async function remove() {
    if (!deleteTarget) return;
    setDeleting(true);
    try { await ipc.aiModelDelete(deleteTarget.id); setDeleteTarget(null); await load(); }
    catch (error) { setMessage(String(error)); }
    finally { setDeleting(false); }
  }

  if (!config) return <p className={message ? "text-sm text-error" : "text-sm text-text-secondary"}>{message || "正在加载 AI 服务…"}</p>;
  const enabled = config.models.filter((model) => model.enabled);
  return <div className="mx-auto max-w-[1040px] pb-8">
    <div className="flex items-start justify-between"><div><h1 className="text-2xl font-bold">AI 服务</h1><p className="mt-1 text-sm text-text-secondary">管理总结指令与模型调用顺序，首选模型失败时自动尝试下一个。</p></div>{returnToProjectId && <Button variant="secondary" onClick={() => onReturn(returnToProjectId)}>返回并生成总结</Button>}</div>
    <section className="mt-6 rounded-lg border border-line bg-surface p-6 shadow-sm">
      <div className="flex items-center justify-between"><div><h2 className="text-lg font-semibold">自定义指令</h2><p className="mt-1 text-xs text-text-tertiary">保存后会附加到每一次 AI 总结请求。</p></div>{!editing && <Button variant="secondary" onClick={() => setEditing(true)}>编辑指令</Button>}</div>
      {editing ? <div className="mt-4"><textarea autoFocus value={instruction} maxLength={4000} onChange={(event) => setInstruction(event.target.value)} className="h-36 w-full resize-none rounded-md border border-line p-3 text-sm leading-6 outline-none focus:border-primary" placeholder="例如：保留人名与数据；输出简洁；待办事项需包含负责人。" /><div className="mt-3 flex items-center justify-between"><span className="text-xs text-text-tertiary">{instruction.length} / 4000</span><div className="flex gap-3"><Button variant="secondary" onClick={() => { setInstruction(config.customInstruction); setEditing(false); }}>取消</Button><Button loading={savingInstruction} onClick={() => void saveInstruction()}>保存指令</Button></div></div></div> : <p className="mt-4 whitespace-pre-wrap rounded-md bg-bg p-4 text-sm leading-6 text-text-secondary">{config.customInstruction || "尚未设置自定义指令。将使用 SnapScribe 默认总结要求。"}</p>}
    </section>
    <section ref={modelsRef} className="mt-6 scroll-mt-8 rounded-lg border border-line bg-surface p-6 shadow-sm">
      <div className="flex items-start justify-between"><div><h2 className="text-lg font-semibold">模型列表</h2><p className="mt-1 text-xs text-text-tertiary">拖动调整优先级；排序最高的已启用模型为默认模型。</p></div><Button onClick={() => { setSelected(null); setDialogOpen(true); }}><Plus className="size-4" />添加模型</Button></div>
      {config.models.length === 0 ? <div className="mt-6 flex flex-col items-center rounded-lg border border-dashed border-line py-12 text-center"><span className="flex size-12 items-center justify-center rounded-round bg-primary-soft text-primary"><Sparkles className="size-6" /></span><p className="mt-4 font-semibold">还没有可用模型</p><p className="mt-1 text-sm text-text-secondary">支持官方、第三方代理及 Ollama 等 OpenAI 兼容服务。</p><Button className="mt-5" onClick={() => setDialogOpen(true)}>添加第一个模型</Button></div> : <div className="mt-5 overflow-hidden rounded-lg border border-line">
        {config.models.map((model) => {
          const isDefault = model.enabled && enabled[0]?.id === model.id;
          const healthy = model.lastStatus.status === "available";
          return <div key={model.id} draggable onDragStart={() => setDraggingId(model.id)} onDragEnd={() => setDraggingId(null)} onDragOver={(event) => event.preventDefault()} onDrop={(event) => { const bounds = event.currentTarget.getBoundingClientRect(); void moveTo(model.id, event.clientY > bounds.top + bounds.height / 2); }} className={`grid min-h-16 grid-cols-[36px_44px_minmax(140px,1.4fr)_minmax(120px,1fr)_minmax(110px,1fr)_88px_96px] items-center gap-2 border-b border-divider px-3 transition-colors last:border-b-0 ${draggingId === model.id ? "bg-primary-soft opacity-60" : "hover:bg-bg"}`}>
            <GripVertical className="size-4 cursor-grab text-text-tertiary" /><span className="text-xs font-mono text-text-tertiary">{model.order + 1}</span>
            <div className="min-w-0"><div className="flex items-center gap-2"><span className="truncate text-sm font-semibold">{model.name}</span>{isDefault && <span className="rounded-round bg-primary-soft px-2 py-0.5 text-[10px] font-semibold text-primary">默认</span>}</div><p className="mt-0.5 truncate text-xs text-text-tertiary" title={model.baseUrl}>{model.baseUrl}</p></div>
            <span className="truncate font-mono text-xs text-text-secondary" title={model.modelId}>{model.modelId}</span>
            <span title={model.lastStatus.message} className={`flex items-center gap-1.5 text-xs font-medium ${healthy ? "text-success" : model.lastStatus.status === "untested" ? "text-text-tertiary" : "text-error"}`}>{healthy ? <CheckCircle2 className="size-4" /> : <CircleAlert className="size-4" />}{statusLabels[model.lastStatus.status]}</span>
            <button type="button" onClick={() => void toggle(model)} className={`relative h-6 w-11 rounded-round transition-colors ${model.enabled ? "bg-primary" : "bg-line"}`} aria-label={model.enabled ? "停用模型" : "启用模型"}><span className={`absolute top-1 size-4 rounded-round bg-white transition-transform ${model.enabled ? "left-6" : "left-1"}`} /></button>
            <div className="flex justify-end gap-1"><button className="flex size-8 items-center justify-center rounded-md text-text-secondary hover:bg-primary-soft hover:text-primary" onClick={() => { setSelected(model); setDialogOpen(true); }} aria-label="编辑模型"><Pencil className="size-4" /></button><button className="flex size-8 items-center justify-center rounded-md text-text-secondary hover:bg-[#FFF1F1] hover:text-error" onClick={() => setDeleteTarget(model)} aria-label="删除模型"><Trash2 className="size-4" /></button></div>
          </div>;
        })}
      </div>}
      <p className="mt-3 text-xs text-text-tertiary">仅已启用模型参与自动降级；超时、鉴权失败、限流和服务错误会立即更新状态。</p>
    </section>
    {message && <p className={`mt-4 text-sm ${message.includes("已保存") ? "text-success" : "text-error"}`}>{message}</p>}
    <AIModelDialog open={dialogOpen} model={selected} onOpenChange={setDialogOpen} onSaved={() => void load()} />
    <DeleteConfirmationDialog open={deleteTarget !== null} title="删除模型配置？" description={`将删除“${deleteTarget?.name ?? ""}”的配置与本机密钥。`} finalDescription="此操作无法撤销，但不会影响已经生成的 AI 总结。" busy={deleting} onOpenChange={(open) => !open && setDeleteTarget(null)} onConfirm={() => void remove()} />
  </div>;
}
