import { useEffect, useLayoutEffect, useRef } from "react";
import type { TranscriptSegment } from "@/types/transcript";
import { formatClock } from "@/lib/format";
import { cn } from "@/lib/utils";

/**
 * Progressive transcript list per UI_DESIGN_SPEC §13. New segments append at
 * the bottom with a 150ms fade-in; auto-scroll follows only while the user
 * has not scrolled away; waiting shows a single quiet placeholder line.
 */
export function TranscriptList({
  segments,
  activeSegmentId,
  transcribing,
  nextSegmentIndex,
  seekable = false,
  onSeek,
}: {
  segments: TranscriptSegment[];
  activeSegmentId: string | null;
  /** True while transcription is running (shows the waiting line). */
  transcribing: boolean;
  nextSegmentIndex: number | null;
  /** Timestamps become seek buttons only once a player is available. */
  seekable?: boolean;
  onSeek: (second: number) => void;
}) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const followRef = useRef(true);

  useLayoutEffect(() => {
    if (followRef.current && scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [segments.length]);

  // Reset follow when a fresh list starts.
  useEffect(() => {
    followRef.current = true;
  }, [segments.length === 0]);

  function handleScroll() {
    const el = scrollRef.current;
    if (!el) return;
    followRef.current = el.scrollHeight - el.scrollTop - el.clientHeight < 48;
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      {segments.length > 0 && (
        <p className="px-1 pb-1 text-xs text-text-tertiary">已接收 {segments.length} 段</p>
      )}
      <div
        ref={scrollRef}
        onScroll={handleScroll}
        className="min-h-0 flex-1 overflow-y-auto rounded-lg border border-divider bg-surface"
      >
        <ul className="divide-y divide-divider">
          {segments.map((segment) => (
            <li
              key={segment.id}
              className={cn(
                "animate-fade-in px-4 py-3 transition-colors duration-150 hover:bg-[#F8F9FF]",
                segment.id === activeSegmentId && "rounded-md bg-primary-soft",
              )}
            >
              <div className="flex gap-3 leading-[1.7]">
                {seekable ? (
                  <button
                    type="button"
                    onClick={() => onSeek(segment.start)}
                    aria-label={`跳转到 ${formatClock(segment.start)}`}
                    className="w-16 shrink-0 text-left font-mono text-xs font-medium text-primary transition-colors duration-150 hover:text-primary/80"
                  >
                    {formatClock(segment.start)}
                  </button>
                ) : (
                  <span className="w-16 shrink-0 font-mono text-xs font-medium text-primary">
                    {formatClock(segment.start)}
                  </span>
                )}
                <p className="text-sm text-text-primary">{segment.text}</p>
              </div>
            </li>
          ))}
        </ul>
        {transcribing && (
          <p className="px-4 py-3 text-xs text-text-tertiary">
            正在识别第 {nextSegmentIndex ?? "?"} 段…
          </p>
        )}
      </div>
    </div>
  );
}
