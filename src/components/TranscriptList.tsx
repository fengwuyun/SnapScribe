import { useEffect, useLayoutEffect, useRef } from "react";
import type { TranscriptSegment, TranscriptSpeaker } from "@/types/transcript";
import { formatClock } from "@/lib/format";
import { speakerBadgeClass, speakerDisplayName } from "@/lib/speakers";
import { cn } from "@/lib/utils";
import { hasTextSelection } from "@/lib/textSelection";

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
  onCopy,
  speakers = [],
}: {
  segments: TranscriptSegment[];
  activeSegmentId: string | null;
  /** True while transcription is running (shows the waiting line). */
  transcribing: boolean;
  nextSegmentIndex: number | null;
  /** Timestamps become seek buttons only once a player is available. */
  seekable?: boolean;
  onSeek: (second: number) => void;
  onCopy?: (message: string) => void;
  speakers?: TranscriptSpeaker[];
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
    <div className="flex h-full min-h-0 flex-1 flex-col">
      {segments.length > 0 && (
        <p className="px-1 pb-1 text-xs text-text-tertiary">已接收 {segments.length} 段</p>
      )}
      <div
        ref={scrollRef}
        onScroll={handleScroll}
        data-testid="transcript-scroll-region"
        className="min-h-0 flex-1 overflow-y-auto overscroll-contain rounded-lg border border-divider bg-surface"
      >
        <ul className="divide-y divide-divider">
          {segments.map((segment) => {
            const speaker = speakers.find((item) => item.id === segment.speakerId);
            const speakerName = speakerDisplayName(segment.speakerId, speakers);
            const copyText = speakerName ? `[${formatClock(segment.start)}] ${speakerName}：${segment.text}` : segment.text;
            return (
            <li
              key={segment.id}
              className={cn(
                "animate-fade-in transition-colors duration-150 hover:bg-[#F8F9FF]",
                segment.id === activeSegmentId && "rounded-md bg-primary-soft",
              )}
            >
              {seekable ? (
                <div className="flex w-full gap-3 px-4 py-3 text-left leading-[1.7]">
                  <button type="button" onClick={() => onSeek(segment.start)} aria-label={`从 ${formatClock(segment.start)} 播放`} className="w-16 shrink-0 font-mono text-xs font-medium text-primary outline-none focus-visible:ring-2 focus-visible:ring-primary">
                    {formatClock(segment.start)}
                  </button>
                  {speaker && speakerName && <span className={`mt-0.5 inline-flex h-6 shrink-0 items-center rounded-round px-2 text-xs font-semibold ${speakerBadgeClass(speaker.colorIndex)}`}>{speakerName}</span>}
                  <span
                    className="min-w-0 flex-1 cursor-text select-text text-sm text-text-primary"
                    onClick={() => { if (!hasTextSelection(window.getSelection())) onSeek(segment.start); }}
                    onDoubleClick={() => { void navigator.clipboard.writeText(copyText).then(() => onCopy?.("已复制该段")).catch(() => onCopy?.("复制失败")); }}
                  >{segment.text}</span>
                </div>
              ) : (
                <div className="flex gap-3 px-4 py-3 leading-[1.7]">
                  <span className="w-16 shrink-0 font-mono text-xs font-medium text-primary">
                    {formatClock(segment.start)}
                  </span>
                  {speaker && speakerName && <span className={`mt-0.5 inline-flex h-6 shrink-0 items-center rounded-round px-2 text-xs font-semibold ${speakerBadgeClass(speaker.colorIndex)}`}>{speakerName}</span>}
                  <p className="cursor-text select-text text-sm text-text-primary" onDoubleClick={() => { void navigator.clipboard.writeText(copyText).then(() => onCopy?.("已复制该段")).catch(() => onCopy?.("复制失败")); }}>{segment.text}</p>
                </div>
              )}
            </li>
          );})}
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
