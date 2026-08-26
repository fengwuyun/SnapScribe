import { useEffect, useRef, useState } from "react";
import { UploadCloud } from "lucide-react";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { isAcceptedFile } from "@/lib/format";
import { selectFile } from "@/lib/tauri";
import { cn } from "@/lib/utils";

/**
 * DropZone per UI_DESIGN_SPEC §8: dashed card, round primary icon, drag-over
 * highlight and click-to-pick fallback. Drop paths arrive through the Tauri
 * webview drag-drop event, which carries absolute paths on Windows.
 */
export function DropZone({
  onPick,
  onInvalidFile,
}: {
  onPick: (path: string) => void;
  onInvalidFile: (fileName: string) => void;
}) {
  const [dragOver, setDragOver] = useState(false);
  // Guards against double-handling while a previous drop is being processed.
  const busyRef = useRef(false);

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void getCurrentWebview()
      .onDragDropEvent((event) => {
        const payload = event.payload;
        if (payload.type === "enter" || payload.type === "over") {
          setDragOver(true);
        } else if (payload.type === "leave") {
          setDragOver(false);
        } else if (payload.type === "drop") {
          setDragOver(false);
          if (busyRef.current) return;
          busyRef.current = true;
          try {
            const path = payload.paths[0];
            if (!path) return;
            const fileName = path.split(/[\\/]/).pop() ?? path;
            if (!isAcceptedFile(fileName)) {
              onInvalidFile(fileName);
              return;
            }
            onPick(path);
          } finally {
            busyRef.current = false;
          }
        }
      })
      .then((fn) => {
        if (disposed) fn();
        else unlisten = fn;
      });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [onPick, onInvalidFile]);

  async function openPicker() {
    const path = await selectFile();
    if (path) onPick(path);
  }

  return (
    <div
      role="button"
      tabIndex={0}
      aria-label="拖入视频或录音，或点击选择文件"
      onClick={() => void openPicker()}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          void openPicker();
        }
      }}
      className={cn(
        "flex w-full cursor-pointer flex-col items-center gap-4 rounded-lg border-2 border-dashed border-[#C9CBE0] bg-surface px-8 py-12 transition-all duration-150",
        "hover:border-primary hover:bg-[#F8F9FF]",
        dragOver && "scale-[1.01] border-primary bg-primary-soft",
      )}
    >
      <span className="flex size-12 items-center justify-center rounded-round bg-primary">
        <UploadCloud className="size-6 text-white" aria-hidden />
      </span>
      <div className="text-center">
        <p className="text-sm font-semibold text-text-primary">拖入视频或录音</p>
        <p className="mt-1 text-xs text-text-tertiary">或点击选择文件</p>
      </div>
      <p className="text-xs text-text-tertiary">MP4 · MOV · MKV · MP3 · M4A · WAV</p>
    </div>
  );
}
