import { useEffect, useState } from "react";
import { Download, ExternalLink, Pencil, Search, Trash2 } from "lucide-react";
import { DeleteConfirmationDialog } from "@/components/DeleteConfirmationDialog";
import { ProjectContextMenu } from "@/components/ProjectContextMenu";
import { SortDropdown } from "@/components/SortDropdown";
import type { AppRoute } from "@/lib/navigation";
import * as ipc from "@/lib/tauri";
import type { ProjectListItem, ProjectSort } from "@/types/project";
import { buildExportName, formatClock, formatProjectCreatedAt, splitHighlightParts, truncateProjectName } from "@/lib/format";

function HighlightedProjectName({ name, query }: { name: string; query: string }) {
  const visibleName = truncateProjectName(name);
  return <>{splitHighlightParts(visibleName, query).map((part, index) => part.highlighted
    ? <mark key={`${part.text}-${index}`} className="rounded-[3px] bg-primary-soft px-0.5 font-semibold text-primary">{part.text}</mark>
    : <span key={`${part.text}-${index}`}>{part.text}</span>)}</>;
}

export function LibraryPage({ onNavigate }: { onNavigate: (route: AppRoute) => void }) {
  const [query, setQuery] = useState("");
  const [sort, setSort] = useState<ProjectSort>("recentlyUpdated");
  const [items, setItems] = useState<ProjectListItem[]>([]);
  const [error, setError] = useState("");
  const [deleteTarget, setDeleteTarget] = useState<ProjectListItem | null>(null);
  const [contextMenu, setContextMenu] = useState<{ project: ProjectListItem; x: number; y: number } | null>(null);
  const [deleting, setDeleting] = useState(false);

  async function refresh() {
    try {
      setItems(await ipc.projectList(query, sort));
      setError("");
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }

  useEffect(() => { void refresh(); }, [query, sort]);

  async function remove() {
    if (!deleteTarget) return;
    setDeleting(true);
    try { await ipc.projectDelete(deleteTarget.id); setDeleteTarget(null); await refresh(); }
    catch (err) { setError(err instanceof Error ? err.message : String(err)); }
    finally { setDeleting(false); }
  }

  async function rename(project: ProjectListItem) {
    const name = window.prompt("项目名称", project.name);
    if (!name || name.trim() === project.name) return;
    try { await ipc.projectRename(project.id, name); await refresh(); }
    catch (err) { setError(err instanceof Error ? err.message : String(err)); }
  }

  async function exportTxt(project: ProjectListItem) {
    const path = await ipc.saveFileDialog(buildExportName(project.name, "txt"), "txt");
    if (!path) return;
    try { await ipc.projectExport(project.id, "txt", path); }
    catch (err) { setError(err instanceof Error ? err.message : String(err)); }
  }

  async function openLocation(project: ProjectListItem) {
    try { await ipc.projectOpenMediaLocation(project.id); }
    catch (err) { setError(err instanceof Error ? err.message : String(err)); }
  }

  return <div className="mx-auto w-full max-w-[1100px]">
    <h1 className="text-2xl font-bold">文件库</h1>
    <p className="mt-1 text-sm text-text-secondary">管理转录项目，而不是媒体文件</p>
    <div className="mt-6 flex items-center justify-between gap-4">
      <label className="flex h-10 max-w-md flex-1 items-center gap-2 rounded-md border border-line bg-surface px-3"><Search className="size-4 text-text-tertiary" aria-hidden /><input value={query} onChange={(event) => setQuery(event.target.value)} placeholder="搜索项目名称或转录内容" className="min-w-0 flex-1 bg-transparent text-sm outline-none" /></label>
      <SortDropdown value={sort} onChange={setSort} />
    </div>
    {error && <p className="mt-3 text-sm text-error">{error}</p>}
    <div className="mt-4 overflow-hidden rounded-lg border border-line bg-surface">
      <div className="grid grid-cols-[180px_72px_120px_minmax(140px,1fr)_156px] gap-3 border-b border-divider px-5 py-3 text-xs text-text-tertiary"><span>项目名称</span><span>时长</span><span>来源</span><span>创建时间</span><span>操作</span></div>
      {items.length === 0 ? <p className="py-12 text-center text-sm text-text-tertiary">没有匹配的转录项目</p> : items.map((project) => <div key={project.id} onContextMenu={(event) => { event.preventDefault(); setContextMenu({ project, x: event.clientX, y: event.clientY }); }} className="grid grid-cols-[180px_72px_120px_minmax(140px,1fr)_156px] items-center gap-3 border-b border-divider px-5 py-3.5 transition-colors duration-150 last:border-b-0 hover:bg-[#F8F9FF]">
        <button type="button" title={project.name} onClick={() => onNavigate({ page: "project", projectId: project.id, from: "library" })} className="truncate text-left text-sm font-semibold hover:text-primary"><HighlightedProjectName name={project.name} query={query} /></button>
        <span className="font-mono text-xs text-text-secondary">{formatClock(project.durationSecs)}</span>
        <span className="text-xs text-text-secondary">{project.media.storage === "reference" ? "导入（引用）" : "SnapScribe 保存"}</span>
        <span className="font-mono text-xs text-text-secondary">{formatProjectCreatedAt(project.createdAt)}</span>
        <div className="inline-flex w-fit items-center overflow-hidden rounded-md border border-line bg-surface shadow-sm">
          <button type="button" aria-label={`重命名 ${project.name}`} title="重命名" onClick={() => void rename(project)} className="flex size-8 items-center justify-center text-text-tertiary transition-colors duration-150 hover:bg-primary-soft hover:text-primary focus-visible:z-10 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary"><Pencil className="size-4" /></button>
          <button type="button" aria-label={`导出 ${project.name}`} title="导出 TXT" onClick={() => void exportTxt(project)} className="flex size-8 items-center justify-center border-l border-divider text-text-tertiary transition-colors duration-150 hover:bg-primary-soft hover:text-primary focus-visible:z-10 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary"><Download className="size-4" /></button>
          <button type="button" aria-label={`打开 ${project.name} 的原文件位置`} title={project.mediaAvailable ? "打开原文件位置" : "原始媒体不可用"} disabled={!project.mediaAvailable} onClick={() => void openLocation(project)} className="flex size-8 items-center justify-center border-l border-divider text-text-tertiary transition-colors duration-150 hover:bg-primary-soft hover:text-primary focus-visible:z-10 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary disabled:cursor-not-allowed disabled:opacity-35"><ExternalLink className="size-4" /></button>
          <button type="button" aria-label={`删除 ${project.name}`} title="删除项目" onClick={() => setDeleteTarget(project)} className="flex size-8 items-center justify-center border-l border-divider text-text-tertiary transition-colors duration-150 hover:bg-[#FFF1F1] hover:text-error focus-visible:z-10 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-error"><Trash2 className="size-4" /></button>
        </div>
      </div>)}
    </div>
    {contextMenu && <ProjectContextMenu
      project={contextMenu.project}
      x={contextMenu.x}
      y={contextMenu.y}
      onClose={() => setContextMenu(null)}
      onOpen={() => onNavigate({ page: "project", projectId: contextMenu.project.id, from: "library" })}
      onRename={() => void rename(contextMenu.project)}
      onExport={() => void exportTxt(contextMenu.project)}
      onOpenLocation={() => void openLocation(contextMenu.project)}
      onDelete={() => setDeleteTarget(contextMenu.project)}
    />}
    <DeleteConfirmationDialog
      open={deleteTarget !== null}
      title="删除转录项目？"
      description={`“${deleteTarget?.name ?? ""}”的转录文本和 AI 总结也会被删除。此操作不可恢复。`}
      busy={deleting}
      onOpenChange={(open) => !open && setDeleteTarget(null)}
      onConfirm={() => void remove()}
    />
  </div>;
}
