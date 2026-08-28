import { useEffect, useMemo, useRef, useState } from "react";
import { ArrowLeft, Check, Clipboard, Download, ExternalLink, Link2, Pause, Pencil, Play, Save, Search, Trash2, X, XCircle } from "lucide-react";
import { AudioPlayer, type SeekRequest } from "@/components/AudioPlayer";
import { DeleteConfirmationDialog } from "@/components/DeleteConfirmationDialog";
import { TranscriptList } from "@/components/TranscriptList";
import { TranscribeProgress } from "@/components/TranscribeProgress";
import { Button } from "@/components/ui/Button";
import { buildExportName, buildSummaryExportName, buildSummaryText, fullText } from "@/lib/format";
import * as ipc from "@/lib/tauri";
import type { ProjectDetail, TranscriptionJobStatus } from "@/types/project";
import type { ProgressEvent, TranscriptSegment } from "@/types/transcript";

export function ProjectDetailPage({ projectId, onBack, onConfigureAI }: { projectId: string; onBack: () => void; onConfigureAI: () => void }) {
  const [detail, setDetail] = useState<ProjectDetail | null>(null);
  const [tab, setTab] = useState<"transcript" | "summary">("transcript");
  const [query, setQuery] = useState("");
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState<TranscriptSegment[]>([]);
  const [message, setMessage] = useState("");
  const [generatingSummary, setGeneratingSummary] = useState(false);
  const [seekRequest, setSeekRequest] = useState<SeekRequest | null>(null);
  const [playbackTime, setPlaybackTime] = useState(0);
  const [progress, setProgress] = useState<ProgressEvent | null>(null);
  const [jobStatus, setJobStatus] = useState<TranscriptionJobStatus | null>(null);
  const [deleteMediaOpen, setDeleteMediaOpen] = useState(false);
  const [deletingMedia, setDeletingMedia] = useState(false);
  const [copied, setCopied] = useState(false);
  const seekToken = useRef(0);

  async function load() {
    try {
      const next = await ipc.projectGet(projectId);
      setDetail(next);
      setJobStatus(await ipc.transcriptionJobStatus(projectId));
      if (!editing) setDraft(next.transcript.segments);
      setMessage("");
    } catch (err) {
      setMessage(err instanceof Error ? err.message : String(err));
    }
  }

  useEffect(() => {
    void load();
    const cleanups: Array<() => void> = [];
    void Promise.all([
      ipc.listenSegments((event) => { if (event.payload.projectId === projectId) setTimeout(() => void load(), 30); }),
      ipc.listenProgress((event) => { if (event.payload.projectId === projectId) setProgress(event.payload); }),
      ipc.listenCompleted<unknown>((event) => { if (event.payload.projectId === projectId) void load(); }),
      ipc.listenCanceled((event) => { if (event.payload.projectId === projectId) void load(); }),
      ipc.listenFailed((event) => { if (event.payload.projectId === projectId) void load(); }),
    ]).then((items) => cleanups.push(...items));
    return () => cleanups.forEach((cleanup) => cleanup());
  }, [projectId, editing]);

  const visibleSegments = useMemo(() => {
    const segments = editing ? draft : detail?.transcript.segments ?? [];
    const needle = query.trim().toLocaleLowerCase();
    return needle ? segments.filter((segment) => segment.text.toLocaleLowerCase().includes(needle)) : segments;
  }, [detail, draft, editing, query]);

  const activeSegmentId = useMemo(() => {
    const segments = detail?.transcript.segments ?? [];
    return [...segments].reverse().find((segment) => playbackTime >= segment.start)?.id ?? null;
  }, [detail, playbackTime]);

  async function toggleTranscriptionPause() {
    if (!jobStatus) return;
    try {
      setJobStatus(await ipc.setTranscriptionPaused(projectId, !jobStatus.paused));
    } catch (err) { setMessage(err instanceof Error ? err.message : String(err)); }
  }

  async function cancelTranscription() {
    try {
      await ipc.cancelProjectTranscription(projectId);
      setMessage("正在取消转录，已完成的文字会保留…");
    } catch (err) { setMessage(err instanceof Error ? err.message : String(err)); }
  }

  async function rename(name: string) {
    if (!detail || name.trim() === detail.project.name) return;
    try { await ipc.projectRename(projectId, name); await load(); }
    catch (err) { setMessage(err instanceof Error ? err.message : String(err)); }
  }

  async function saveTranscript() {
    try {
      await ipc.projectSaveTranscript(projectId, draft);
      setEditing(false);
      await load();
      setMessage("转录文本已保存");
    } catch (err) { setMessage(err instanceof Error ? err.message : String(err)); }
  }

  async function relink() {
    const path = await ipc.selectFile();
    if (!path) return;
    try { await ipc.projectRelinkMedia(projectId, path); await load(); setMessage("原始文件已重新关联"); }
    catch (err) { setMessage(err instanceof Error ? err.message : String(err)); }
  }

  async function exportCurrentTab() {
    if (!detail) return;
    if (tab === "summary" && !detail.summary) {
      setMessage("请先生成 AI 总结");
      return;
    }
    const defaultName = tab === "summary"
      ? buildSummaryExportName(detail.project.name)
      : buildExportName(detail.project.name, "txt");
    const path = await ipc.saveFileDialog(defaultName, "txt");
    if (!path) return;
    try { await ipc.projectExport(projectId, tab === "summary" ? "summary" : "txt", path); setMessage(tab === "summary" ? "AI 总结导出成功" : "转录文本导出成功"); }
    catch (err) { setMessage(err instanceof Error ? err.message : String(err)); }
  }

  async function copyCurrentTab() {
    if (!detail) return;
    const content = tab === "summary"
      ? detail.summary ? buildSummaryText(detail.summary) : ""
      : fullText(detail.transcript.segments);
    if (!content) {
      setMessage(tab === "summary" ? "请先生成 AI 总结" : "暂无可复制的转录文本");
      return;
    }
    try {
      await navigator.clipboard.writeText(content);
      setCopied(true);
      setMessage(tab === "summary" ? "AI 总结已复制" : "转录文本已复制");
      window.setTimeout(() => setCopied(false), 1600);
    } catch (err) { setMessage(err instanceof Error ? err.message : "复制失败"); }
  }

  async function deleteMedia() {
    setDeletingMedia(true);
    try { await ipc.projectDeleteManagedMedia(projectId); setDeleteMediaOpen(false); await load(); setMessage("媒体文件已删除，文本仍保留"); }
    catch (err) { setMessage(err instanceof Error ? err.message : String(err)); }
    finally { setDeletingMedia(false); }
  }

  async function generateSummary() {
    try {
      const config = await ipc.aiServiceGet();
      if (!config.models.some((model) => model.enabled)) {
        onConfigureAI();
        return;
      }
    } catch (err) {
      setMessage(err instanceof Error ? err.message : String(err));
      return;
    }
    setGeneratingSummary(true);
    setMessage("");
    try {
      await ipc.aiGenerateSummary(projectId);
      await load();
      setMessage("AI 总结已生成");
    } catch (err) {
      const reason = err instanceof Error ? err.message : String(err);
      if (reason.includes("NO_AI_MODELS_CONFIGURED")) onConfigureAI();
      else setMessage(reason);
    } finally {
      setGeneratingSummary(false);
    }
  }

  if (!detail) return <div className="p-8"><button onClick={onBack}>← 返回</button><p className="mt-8 text-sm text-text-secondary">{message || "正在加载项目…"}</p></div>;
  const canDeleteMedia = detail.mediaAvailable && detail.project.media.storage !== "reference";

  return <div className="flex h-full min-h-0 flex-col overflow-hidden bg-bg">
    <header className="flex h-14 shrink-0 items-center gap-3 border-b border-divider bg-surface px-6">
      <button type="button" onClick={onBack} aria-label="返回" className="flex size-8 items-center justify-center rounded-sm hover:bg-bg"><ArrowLeft className="size-4" /></button>
      <input key={detail.project.name} defaultValue={detail.project.name} onBlur={(event) => void rename(event.target.value)} className="min-w-0 flex-1 truncate bg-transparent text-sm font-semibold outline-none focus:text-primary" aria-label="项目名称" />
      <Button variant="secondary" className="h-8 px-3" onClick={() => void copyCurrentTab()}>{copied ? <Check className="size-4 text-success" /> : <Clipboard className="size-4" />}复制全文</Button>
      <Button variant="secondary" className="h-8 px-3" onClick={() => void exportCurrentTab()}><Download className="size-4" />导出</Button>
      {detail.mediaAvailable && <button type="button" onClick={() => void ipc.projectOpenMediaLocation(projectId)} className="flex size-8 items-center justify-center rounded-sm text-text-secondary hover:bg-bg" title="打开原文件位置"><ExternalLink className="size-4" /></button>}
      {canDeleteMedia && <button type="button" onClick={() => setDeleteMediaOpen(true)} className="flex size-8 items-center justify-center rounded-sm text-text-secondary hover:bg-[#FFF1F1] hover:text-error" title="删除媒体文件"><Trash2 className="size-4" /></button>}
    </header>
    <main className="flex min-h-0 flex-1 flex-col overflow-hidden px-8 pt-4">
      {detail.project.status === "transcribing" && <div className="mb-3 shrink-0 rounded-lg border border-line bg-surface px-4 py-3 shadow-sm">
        <div className="flex items-end gap-4">
          <div className="min-w-0 flex-1"><TranscribeProgress fileName={detail.project.media.originalFileName || detail.project.name} progress={jobStatus?.paused && progress ? { ...progress, stage: "paused" } : progress} /></div>
          <div className="flex shrink-0 gap-2">
            <Button variant="secondary" className="h-9 px-3" onClick={() => void toggleTranscriptionPause()} disabled={!jobStatus?.running}>{jobStatus?.paused ? <Play className="size-4" /> : <Pause className="size-4" />}{jobStatus?.paused ? "继续" : "暂停"}</Button>
            <Button variant="secondary" className="h-9 px-3 text-error hover:bg-[#FFF1F1]" onClick={() => void cancelTranscription()} disabled={!jobStatus?.running}><XCircle className="size-4" />取消</Button>
          </div>
        </div>
      </div>}

      <div className="flex shrink-0 gap-8 border-b border-divider">
        <button onClick={() => setTab("transcript")} className={`border-b-2 px-1 pb-3 text-sm transition-colors duration-150 ${tab === "transcript" ? "border-primary font-semibold text-primary" : "border-transparent text-text-secondary hover:text-text-primary"}`}>转录文本</button>
        <button onClick={() => setTab("summary")} className={`border-b-2 px-1 pb-3 text-sm transition-colors duration-150 ${tab === "summary" ? "border-primary font-semibold text-primary" : "border-transparent text-text-secondary hover:text-text-primary"}`}>AI 总结</button>
      </div>
      {message && <p className="shrink-0 py-2 text-sm text-text-secondary">{message}</p>}

      {tab === "transcript" ? <div className="flex min-h-0 flex-1 flex-col py-3">
        <div className="mb-3 flex shrink-0 items-center justify-between gap-4">
          <label className="flex h-9 max-w-sm flex-1 items-center gap-2 rounded-md border border-line bg-surface px-3"><Search className="size-4 text-text-tertiary" /><input value={query} onChange={(event) => setQuery(event.target.value)} placeholder="搜索转录内容" className="min-w-0 flex-1 bg-transparent text-sm outline-none" /></label>
          {editing ? <div className="flex gap-2"><Button variant="secondary" className="h-9" onClick={() => { setEditing(false); setDraft(detail.transcript.segments); }}><X className="size-4" />取消</Button><Button className="h-9" onClick={() => void saveTranscript()}><Save className="size-4" />保存文本</Button></div> : <Button variant="secondary" className="h-9" onClick={() => { setDraft(detail.transcript.segments); setEditing(true); }}><Pencil className="size-4" />编辑</Button>}
        </div>
        {!detail.mediaAvailable && <div className="mb-3 flex shrink-0 items-center justify-between rounded-md border border-[#F3D7A2] bg-[#FFF9ED] px-4 py-2.5 text-sm text-text-secondary"><span>原始媒体不可用，文本和总结仍可正常查看。</span><Button variant="secondary" className="h-8 px-3" onClick={() => void relink()}><Link2 className="size-4" />重新关联原始文件</Button></div>}
        {editing ? <div className="min-h-0 flex-1 overflow-y-auto rounded-lg border border-line bg-surface">{visibleSegments.map((segment) => <div key={segment.id} className="flex gap-4 border-b border-divider p-4 last:border-b-0"><button type="button" onClick={() => setSeekRequest({ at: segment.start, token: ++seekToken.current })} className="w-16 shrink-0 font-mono text-xs text-primary">{Math.floor(segment.start / 60).toString().padStart(2, "0")}:{Math.floor(segment.start % 60).toString().padStart(2, "0")}</button><textarea value={segment.text} onChange={(event) => setDraft((items) => items.map((item) => item.id === segment.id ? { ...item, text: event.target.value } : item))} rows={2} className="min-h-16 flex-1 resize-y rounded-md border border-line px-3 py-2 text-sm leading-6 outline-none focus:border-primary" /></div>)}</div> : <div className="flex min-h-0 flex-1"><TranscriptList segments={visibleSegments} activeSegmentId={activeSegmentId} transcribing={detail.project.status === "transcribing" && !jobStatus?.paused} nextSegmentIndex={progress?.segmentIndex ?? null} seekable={detail.mediaAvailable} onSeek={(second) => setSeekRequest({ at: second, token: ++seekToken.current })} /></div>}
      </div> : <div className="min-h-0 flex-1 overflow-y-auto py-5">{detail.summary ? <div className="mx-auto max-w-3xl space-y-4"><section className="rounded-lg border border-line bg-surface p-6 shadow-sm"><h2 className="text-base font-semibold">摘要</h2><p className="mt-3 whitespace-pre-wrap text-sm leading-7">{detail.summary.summary}</p></section><section className="rounded-lg border border-line bg-surface p-6 shadow-sm"><h2 className="text-base font-semibold">关键要点</h2><ul className="mt-3 list-disc space-y-2 pl-5 text-sm leading-6">{detail.summary.keyPoints.map((item, index) => <li key={index}>{item}</li>)}</ul></section><section className="rounded-lg border border-line bg-surface p-6 shadow-sm"><h2 className="text-base font-semibold">待办事项</h2><ul className="mt-3 space-y-2 text-sm leading-6">{detail.summary.actionItems.map((item, index) => <li key={index}>□ {item}</li>)}</ul></section></div> : <div className="flex h-full items-center justify-center"><div className="text-center"><h2 className="text-lg font-semibold">尚未生成 AI 总结</h2><p className="mt-2 text-sm text-text-secondary">基于当前转录文本生成摘要、关键要点和待办事项。</p><Button loading={generatingSummary} onClick={() => void generateSummary()} className="mt-6">生成 AI 总结</Button></div></div>}</div>}

      {detail.mediaAvailable && detail.project.media.path && <div className="shrink-0 border-t border-divider bg-bg py-3"><AudioPlayer filePath={detail.project.media.path} fileName={detail.project.media.originalFileName || detail.project.name} seekRequest={seekRequest} onTimeUpdate={setPlaybackTime} /></div>}
    </main>
    <DeleteConfirmationDialog open={deleteMediaOpen} title="删除录音文件？" description="录音文件删除后将无法播放，但转录文本和 AI 总结会继续保留。" finalDescription="此操作不可恢复。确认永久删除 SnapScribe 保存的原始录音文件吗？" busy={deletingMedia} onOpenChange={setDeleteMediaOpen} onConfirm={() => void deleteMedia()} />
  </div>;
}
