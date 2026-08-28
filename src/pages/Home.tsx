import { useCallback, useMemo, useRef, useState } from "react";
import { AppHeader } from "@/components/AppHeader";
import { AudioPlayer, type SeekRequest } from "@/components/AudioPlayer";
import { BottomActionBar } from "@/components/BottomActionBar";
import { DropZone } from "@/components/DropZone";
import { ErrorState } from "@/components/ErrorState";
import { FileItem } from "@/components/FileItem";
import { HistoryModal } from "@/components/HistoryModal";
import { Toast } from "@/components/Toast";
import { TranscribeProgress } from "@/components/TranscribeProgress";
import { TranscriptList } from "@/components/TranscriptList";
import { useTranscription } from "@/hooks/useTranscription";
import { buildExportName, fullText } from "@/lib/format";
import * as ipc from "@/lib/tauri";

export default function Home() {
  const { state, loadFile, clearFile, start, cancel } = useTranscription();
  const [historyOpen, setHistoryOpen] = useState(false);
  const [toast, setToast] = useState<string | null>(null);
  const [playbackSec, setPlaybackSec] = useState(0);
  const [seekRequest, setSeekRequest] = useState<SeekRequest | null>(null);
  const seekToken = useRef(0);
  const toastTimer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

  const showToast = useCallback((message: string) => {
    setToast(message);
    clearTimeout(toastTimer.current);
    toastTimer.current = setTimeout(() => setToast(null), 2200);
  }, []);

  const activeSegmentId = useMemo(() => {
    let found: string | null = null;
    for (const segment of state.segments) {
      if (segment.start <= playbackSec) found = segment.id;
      else break;
    }
    return found;
  }, [state.segments, playbackSec]);

  const pickFile = useCallback(async () => {
    try {
      const path = await ipc.selectFile();
      if (!path) return;
      await loadFile(path);
    } catch (err) {
      showToast(err instanceof Error ? err.message : String(err));
    }
  }, [loadFile, showToast]);

  const handleDropPick = useCallback(
    async (path: string) => {
      try {
        await loadFile(path);
      } catch (err) {
        showToast(err instanceof Error ? err.message : String(err));
      }
    },
    [loadFile, showToast],
  );

  const seekTo = useCallback((second: number) => {
    setSeekRequest({ at: Math.max(0, second), token: ++seekToken.current });
  }, []);

  const copyAll = useCallback(() => {
    void navigator.clipboard
      .writeText(fullText(state.segments))
      .then(() => showToast("已复制全文"))
      .catch(() => showToast("复制失败，请手动选择文本"));
  }, [state.segments, showToast]);

  const exportAs = useCallback(
    async (kind: "txt" | "srt") => {
      if (!state.file) return;
      try {
        const target = await ipc.saveFileDialog(buildExportName(state.file.fileName, kind), kind);
        if (!target) return;
        const payload = ipc.segmentsToJson(state.segments);
        if (kind === "txt") await ipc.exportTxt(payload, target);
        else await ipc.exportSrt(payload, target);
        showToast(kind === "txt" ? "TXT 导出成功" : "SRT 导出成功");
      } catch (err) {
        showToast(err instanceof Error ? err.message : String(err));
      }
    },
    [state.file, state.segments, showToast],
  );

  const showFileRow = state.phase !== "empty" && state.phase !== "error" && state.file;
  const showPlayer = state.phase === "completed";

  return (
    <div className="flex h-full flex-col bg-bg">
      <AppHeader onOpenHistory={() => setHistoryOpen(true)} />

      <main className="flex min-h-0 flex-1 justify-center overflow-hidden px-10 py-6">
        <div className="flex min-h-0 w-full max-w-[1000px] flex-col gap-5">
          {state.phase === "empty" && (
            <div className="flex flex-1 items-center">
              <DropZone onPick={(path) => void handleDropPick(path)} onInvalidFile={(name) => showToast(`暂不支持该格式:${name}`)} />
            </div>
          )}

          {showFileRow && state.file && (
            <FileItem file={state.file} onRemove={clearFile} />
          )}

          {state.phase === "transcribing" && (
            <>
              <TranscribeProgress fileName={state.file?.fileName ?? ""} progress={state.progress} />
              <TranscriptList
                segments={state.segments}
                activeSegmentId={null}
                transcribing
                nextSegmentIndex={state.progress?.segmentIndex ?? null}
                onSeek={() => {}}
              />
            </>
          )}

          {state.phase === "ready" && !state.file && (
            <div className="flex flex-1 items-center">
              <DropZone onPick={(path) => void handleDropPick(path)} onInvalidFile={(name) => showToast(`暂不支持该格式:${name}`)} />
            </div>
          )}

          {showPlayer && state.filePath && (
            <>
              <AudioPlayer
                filePath={state.filePath}
                fileName={state.file?.fileName ?? "音频文件"}
                seekRequest={seekRequest}
                onTimeUpdate={setPlaybackSec}
              />
              <TranscriptList
                segments={state.segments}
                activeSegmentId={activeSegmentId}
                transcribing={false}
                nextSegmentIndex={null}
                seekable
                onSeek={seekTo}
              />
            </>
          )}

          {state.phase === "error" && (
            <ErrorState
              message={state.error ?? ""}
              onRetry={() => {
                clearFile();
                void pickFile();
              }}
            />
          )}
        </div>
      </main>

      <BottomActionBar
        phase={state.phase}
        progress={state.progress}
        onPickFile={() => void pickFile()}
        onStart={() => void start()}
        onCancel={() => void cancel()}
        onCopy={copyAll}
        onExportTxt={() => void exportAs("txt")}
        onExportSrt={() => void exportAs("srt")}
      />

      <HistoryModal open={historyOpen} onOpenChange={setHistoryOpen} onNotify={showToast} />
      <Toast message={toast} />
    </div>
  );
}
