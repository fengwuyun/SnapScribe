import { useMemo } from "react";
import type { ProgressEvent } from "@/types/transcript";
import { formatClock } from "@/lib/format";

const BAR_COUNT = 40;

/** Deterministic pseudo-waveform heights (28px band) — decorative only. */
function useBars(): number[] {
  return useMemo(() => {
    let seed = 42;
    return Array.from({ length: BAR_COUNT }, () => {
      seed = (seed * 9301 + 49297) % 233280;
      return 8 + Math.round((seed / 233280) * 20);
    });
  }, []);
}

/**
 * TranscribeProgress per UI_DESIGN_SPEC §11: file label + percent,
 * waveform hint, thin progress bar, processed/total and segment counters.
 * The preparing/splitting stages keep the waveform unlit.
 */
export function TranscribeProgress({
  fileName,
  progress,
}: {
  fileName: string;
  progress: ProgressEvent | null;
}) {
  const bars = useBars();
  const percent = progress?.percent ?? 0;
  const litCount = Math.round((percent / 100) * BAR_COUNT);
  const stageLabel = !progress
    ? "正在准备…"
    : progress.stage === "paused"
      ? "转录已暂停"
    : progress.stage === "transcribing"
      ? `正在转写 ${fileName}`
      : progress.stage === "splitting"
        ? "正在提取音频并切片…"
        : "正在读取媒体信息…";

  return (
    <section aria-label="转写进度" className="rounded-lg bg-surface p-6">
      <div className="flex items-baseline justify-between">
        <p className="text-sm font-semibold text-text-primary">{stageLabel}</p>
        <p className="font-mono text-xs font-medium text-primary">{percent}%</p>
      </div>

      <div className="mt-4 flex h-7 items-end gap-1" aria-hidden>
        {bars.map((height, i) => (
          <span
            key={i}
            style={{ height }}
            className={`flex-1 rounded-sm transition-colors duration-150 ${
              i < litCount ? "bg-primary" : "bg-[#C9CBE0]"
            }`}
          />
        ))}
      </div>

      <div className="mt-3 h-1 overflow-hidden rounded-round bg-line" role="progressbar" aria-valuenow={percent} aria-valuemin={0} aria-valuemax={100}>
        <div
          className="h-full rounded-round bg-primary transition-[width] duration-150 ease-linear"
          style={{ width: `${percent}%` }}
        />
      </div>

      <p className="mt-3 font-mono text-xs text-text-tertiary">
        {progress
          ? progress.stage === "transcribing" || progress.stage === "paused"
            ? `已处理 ${formatClock(progress.processedSeconds)} / ${formatClock(progress.totalSeconds)} · 第 ${progress.segmentIndex} / ${progress.segmentCount} 段`
            : `总时长 ${formatClock(progress.totalSeconds)}`
          : ""}
      </p>
    </section>
  );
}
