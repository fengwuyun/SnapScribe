import { FileAudio, FileVideo, X } from "lucide-react";
import type { MediaInfo } from "@/types/transcript";
import { formatClock, formatSize } from "@/lib/format";

const VIDEO_CONTAINERS = new Set(["mp4", "mov", "mkv"]);

/** FileItem per UI_DESIGN_SPEC §9. */
export function FileItem({ file, onRemove }: { file: MediaInfo; onRemove: () => void }) {
  const isVideo = VIDEO_CONTAINERS.has(file.container);
  return (
    <div className="flex items-center gap-3 rounded-md border border-primary-soft bg-surface px-3.5 py-3">
      <span
        className={cnIcon(isVideo)}
        aria-hidden
      >
        {isVideo ? <FileVideo className="size-5" /> : <FileAudio className="size-5" />}
      </span>
      <div className="min-w-0 flex-1">
        <p className="truncate text-sm font-semibold text-text-primary">{file.fileName}</p>
        <p className="mt-0.5 text-xs text-text-tertiary">
          {formatSize(file.sizeBytes)} · {formatClock(file.durationSecs)}
        </p>
      </div>
      <button
        type="button"
        onClick={onRemove}
        aria-label={`移除 ${file.fileName}`}
        className="rounded-sm p-1 text-text-tertiary transition-colors duration-150 hover:bg-divider hover:text-text-secondary"
      >
        <X className="size-4" />
      </button>
    </div>
  );
}

function cnIcon(isVideo: boolean): string {
  const base =
    "flex size-[38px] shrink-0 items-center justify-center rounded-[10px] bg-primary-soft";
  return isVideo ? `${base} text-primary` : `${base} text-primary`;
}
