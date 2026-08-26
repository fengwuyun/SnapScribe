import { useEffect, useState } from "react";
import { Check, Copy, FileText, Pencil, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { Dialog, DialogContent } from "@/components/ui/Dialog";
import type { HistoryEntry } from "@/types/transcript";
import { formatDateTime, formatSize, fullText } from "@/lib/format";
import * as ipc from "@/lib/tauri";

type Panel =
  | { kind: "list" }
  | { kind: "preview"; name: string; content: string }
  | { kind: "confirmDelete"; name: string };

/**
 * History management per DEVELOPMENT_SPEC §8: list / preview / rename /
 * delete over the auto-saved TXT transcripts. No search, no tags, no batch.
 */
export function HistoryModal({
  open,
  onOpenChange,
  onNotify,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onNotify: (message: string) => void;
}) {
  const [entries, setEntries] = useState<HistoryEntry[]>([]);
  const [panel, setPanel] = useState<Panel>({ kind: "list" });
  const [renaming, setRenaming] = useState<{ name: string; draft: string } | null>(null);
  const [dirPath, setDirPath] = useState("");

  async function refresh() {
    setEntries(await ipc.historyList());
  }

  useEffect(() => {
    if (!open) return;
    setPanel({ kind: "list" });
    setRenaming(null);
    void refresh();
    void ipc.historyDirPath().then(setDirPath);
  }, [open]);

  async function handleRename(oldName: string, draft: string) {
    const finalName = await ipc.historyRename(oldName, draft.trim());
    setRenaming(null);
    await refresh();
    onNotify(`已重命名为 ${finalName}`);
  }

  async function handleDelete(name: string) {
    await ipc.historyDelete(name);
    setPanel({ kind: "list" });
    await refresh();
    onNotify("已删除");
  }

  async function openPreview(name: string) {
    const content = await ipc.historyRead(name);
    setPanel({ kind: "preview", name, content });
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent title="历史转写" onCloseLabel="关闭历史记录" className="w-[560px]">
        {panel.kind === "list" && (
          <div className="flex max-h-[420px] min-h-[200px] flex-col gap-2 overflow-y-auto pr-1">
            {entries.length === 0 && (
              <p className="py-10 text-center text-xs text-text-tertiary">还没有历史转写记录</p>
            )}
            {entries.map((entry) =>
              renaming?.name === entry.fileName ? (
                <div key={entry.fileName} className="flex items-center gap-2 rounded-md border border-line p-2">
                  <Input
                    autoFocus
                    value={renaming.draft}
                    onChange={(e) => setRenaming({ name: entry.fileName, draft: e.target.value })}
                    onKeyDown={(e) => {
                      if (e.key === "Enter") void handleRename(entry.fileName, renaming.draft);
                      if (e.key === "Escape") setRenaming(null);
                    }}
                    aria-label="新文件名"
                  />
                  <button
                    type="button"
                    aria-label="确认重命名"
                    onClick={() => void handleRename(entry.fileName, renaming.draft)}
                    className="rounded-sm p-1.5 text-success hover:bg-primary-soft"
                  >
                    <Check className="size-4" />
                  </button>
                </div>
              ) : (
                <div
                  key={entry.fileName}
                  className="group flex items-center gap-3 rounded-md border border-transparent px-3 py-2.5 transition-colors duration-150 hover:border-divider hover:bg-bg"
                >
                  <span className="flex size-[38px] shrink-0 items-center justify-center rounded-[10px] bg-primary-soft text-primary">
                    <FileText className="size-5" aria-hidden />
                  </span>
                  <button
                    type="button"
                    onClick={() => void openPreview(entry.fileName)}
                    className="min-w-0 flex-1 text-left"
                    aria-label={`预览 ${entry.fileName}`}
                  >
                    <p className="truncate text-sm font-semibold text-text-primary">{entry.fileName}</p>
                    <p className="mt-0.5 text-xs text-text-tertiary">
                      {formatSize(entry.sizeBytes)} · {formatDateTime(entry.modifiedMs)}
                    </p>
                  </button>
                  <button
                    type="button"
                    aria-label={`重命名 ${entry.fileName}`}
                    onClick={() => setRenaming({ name: entry.fileName, draft: entry.fileName.replace(/\.txt$/i, "") })}
                    className="rounded-sm p-1.5 text-text-tertiary transition-colors duration-150 hover:bg-primary-soft hover:text-primary"
                  >
                    <Pencil className="size-4" />
                  </button>
                  <button
                    type="button"
                    aria-label={`删除 ${entry.fileName}`}
                    onClick={() => setPanel({ kind: "confirmDelete", name: entry.fileName })}
                    className="rounded-sm p-1.5 text-text-tertiary transition-colors duration-150 hover:bg-accent-soft/40 hover:text-accent"
                  >
                    <Trash2 className="size-4" />
                  </button>
                </div>
              ),
            )}
          </div>
        )}

        {panel.kind === "preview" && (
          <div className="flex flex-col gap-3">
            <p className="truncate text-sm font-semibold text-text-primary">{panel.name}</p>
            <pre className="max-h-[320px] overflow-y-auto whitespace-pre-wrap rounded-md border border-divider bg-bg p-4 font-sans text-sm leading-[1.7] text-text-primary select-text">
              {panel.content}
            </pre>
            <div className="flex justify-end gap-3">
              <Button
                variant="secondary"
                onClick={() => {
                  void navigator.clipboard.writeText(fullText([{ text: panel.content }]));
                  onNotify("已复制全文");
                }}
              >
                <Copy className="size-4" aria-hidden />
                复制全文
              </Button>
              <Button variant="secondary" onClick={() => setPanel({ kind: "list" })}>
                返回列表
              </Button>
            </div>
          </div>
        )}

        {panel.kind === "confirmDelete" && (
          <div className="flex flex-col gap-5">
            <p className="text-sm text-text-primary">
              确认删除 <span className="font-semibold">{panel.name}</span>？此操作不可恢复。
            </p>
            <div className="flex justify-end gap-3">
              <Button variant="secondary" onClick={() => setPanel({ kind: "list" })}>
                取消
              </Button>
              <Button variant="danger" onClick={() => void handleDelete(panel.name)}>
                删除
              </Button>
            </div>
          </div>
        )}

        {dirPath && panel.kind === "list" && (
          <p className="mt-4 truncate border-t border-divider pt-3 text-xs text-text-tertiary" title={dirPath}>
            保存位置：{dirPath}
          </p>
        )}
      </DialogContent>
    </Dialog>
  );
}
