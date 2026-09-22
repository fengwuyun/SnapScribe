import { useEffect, useRef } from "react";
import { Mic, Pause, Play, Square, X } from "lucide-react";
import { TranscriptList } from "@/components/TranscriptList";
import { Button } from "@/components/ui/Button";
import { formatClock } from "@/lib/format";
import type { TranscriptSegment, TranscriptSpeaker } from "@/types/transcript";

function RecordingWaveform({ levels, paused }: { levels: number[]; paused: boolean }) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const draw = () => {
      const width = canvas.clientWidth;
      const height = canvas.clientHeight;
      const scale = window.devicePixelRatio || 1;
      canvas.width = Math.max(1, Math.floor(width * scale));
      canvas.height = Math.max(1, Math.floor(height * scale));
      const context = canvas.getContext("2d");
      if (!context) return;
      context.scale(scale, scale);
      context.clearRect(0, 0, width, height);
      const count = 64;
      const gap = 3;
      const barWidth = Math.max(2, (width - gap * (count - 1)) / count);
      const values = [...Array(Math.max(0, count - levels.length)).fill(0), ...levels].slice(-count);
      context.fillStyle = paused ? "#C9CBE0" : "#4353FF";
      values.forEach((level, index) => {
        const normalized = Math.max(0.06, Math.min(1, level));
        const barHeight = 6 + normalized * (height - 10);
        context.beginPath();
        context.roundRect(index * (barWidth + gap), (height - barHeight) / 2, barWidth, barHeight, Math.min(3, barWidth / 2));
        context.fill();
      });
    };
    draw();
    const observer = new ResizeObserver(draw);
    observer.observe(canvas);
    return () => observer.disconnect();
  }, [levels, paused]);

  return <canvas ref={canvasRef} className="h-20 w-full" aria-label={paused ? "录音波形已暂停" : "实时录音波形"} />;
}

export function RecordingPanel({
  seconds,
  paused,
  stopping,
  levels,
  segments,
  speakers,
  diarizationEnabled,
  onTogglePause,
  onStop,
  onCancel,
}: {
  seconds: number;
  paused: boolean;
  stopping: boolean;
  levels: number[];
  segments: TranscriptSegment[];
  speakers: TranscriptSpeaker[];
  diarizationEnabled: boolean;
  onTogglePause: () => void;
  onStop: () => void;
  onCancel: () => void;
}) {
  return <section className="min-h-[500px] overflow-hidden rounded-lg border border-line bg-surface transition-colors duration-150">
    <div className="border-b border-divider px-8 py-7 text-center">
      <div className="inline-flex items-center gap-2 rounded-round bg-primary-soft px-3 py-1.5 text-xs font-medium text-primary">
        <span className={`size-2 rounded-round ${paused ? "bg-warning" : "bg-error"}`} aria-hidden />
        {paused ? "录音已暂停" : "正在录音"}
      </div>
      <p className="mt-4 font-mono text-3xl font-semibold tracking-tight text-text-primary">{formatClock(seconds)}</p>
      <div className="mx-auto mt-4 max-w-3xl"><RecordingWaveform levels={levels} paused={paused} /></div>
      <p className="mt-3 text-xs text-text-tertiary">每 15 秒完成一个本地切片并追加近实时文字</p>
      <div className="mt-5 flex items-center justify-center gap-3">
        <button type="button" onClick={onTogglePause} disabled={stopping} aria-label={paused ? "继续录音" : "暂停录音"} className="flex size-10 items-center justify-center rounded-round border border-line bg-surface text-text-primary transition-colors duration-150 hover:bg-bg focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary disabled:opacity-45">
          {paused ? <Play className="size-4" /> : <Pause className="size-4" />}
        </button>
        <button type="button" onClick={onStop} disabled={stopping} aria-label="停止并保存录音" className="flex size-12 items-center justify-center rounded-round bg-error text-white transition-colors duration-150 hover:bg-error/90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-error/40 disabled:opacity-45">
          <Square className="size-4 fill-current" />
        </button>
        <Button variant="secondary" className="h-10" onClick={onCancel} disabled={stopping}><X className="size-4" />取消录音</Button>
      </div>
    </div>
    <div className="flex h-[250px] min-h-0 flex-col px-6 py-5">
      <div className="mb-3 flex items-center justify-between">
        <div className="flex items-center gap-2"><Mic className="size-4 text-primary" /><h2 className="text-sm font-semibold">近实时字幕</h2></div>
        <span className="text-xs text-text-tertiary">{diarizationEnabled ? "说话人标签将在录音结束后校正 · " : ""}已接收 {segments.length} 段</span>
      </div>
      <TranscriptList segments={segments} speakers={speakers} activeSegmentId={null} transcribing={!paused && !stopping} nextSegmentIndex={Math.floor(seconds / 15) + 1} onSeek={() => {}} />
    </div>
  </section>;
}
