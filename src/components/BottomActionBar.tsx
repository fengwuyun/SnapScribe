import { Copy, Download, FileText, Play } from "lucide-react";
import { Button } from "@/components/ui/Button";
import type { AppPhase } from "@/hooks/useTranscription";
import type { ProgressEvent } from "@/types/transcript";

/** Bottom action bar per UI_DESIGN_SPEC §18: one primary action at a time. */
export function BottomActionBar({
  phase,
  progress,
  onPickFile,
  onStart,
  onCancel,
  onCopy,
  onExportTxt,
  onExportSrt,
}: {
  phase: AppPhase;
  progress: ProgressEvent | null;
  onPickFile: () => void;
  onStart: () => void;
  onCancel: () => void;
  onCopy: () => void;
  onExportTxt: () => void;
  onExportSrt: () => void;
}) {
  const transcribing = phase === "transcribing";
  const completed = phase === "completed";

  return (
    <footer className="shrink-0 border-t border-divider bg-surface px-8 py-4">
      <div className="mx-auto flex w-full max-w-[1000px] items-center justify-end gap-3">
        {transcribing ? (
          <>
            <Button
              variant="secondary"
              onClick={onCancel}
              aria-label="取消转写"
            >
              取消转写
            </Button>
            <Button variant="primary" loading aria-busy>
              {progress?.stage === "splitting"
                ? "正在切片…"
                : `正在转写 ${progress ? `${progress.percent}%` : ""}`}
            </Button>
          </>
        ) : (
          <>
            <Button variant="ghost" onClick={onPickFile}>
              选择文件
            </Button>
            {completed && (
              <>
                <Button variant="secondary" onClick={onCopy}>
                  <Copy className="size-4" aria-hidden />
                  复制全文
                </Button>
                <Button variant="secondary" onClick={onExportTxt} aria-label="导出 TXT">
                  <Download className="size-4" aria-hidden />
                  导出 TXT
                </Button>
                <Button variant="secondary" onClick={onExportSrt} aria-label="导出 SRT">
                  <FileText className="size-4" aria-hidden />
                  导出 SRT
                </Button>
              </>
            )}
            {phase === "ready" && (
              <Button variant="primary" onClick={onStart}>
                <Play className="size-4" aria-hidden />
                开始转写
              </Button>
            )}
          </>
        )}
      </div>
    </footer>
  );
}
