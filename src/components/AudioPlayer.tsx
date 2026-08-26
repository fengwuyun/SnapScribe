import { useEffect, useRef, useState } from "react";
import { Pause, Play, Volume2 } from "lucide-react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { formatClock } from "@/lib/format";

export interface SeekRequest {
  at: number;
  token: number;
}

/**
 * AudioPlayer per UI_DESIGN_SPEC §12: a compact navigation aid for the
 * transcript — play/pause, seek bar, current time and duration.
 * Timestamp clicks elsewhere raise a SeekRequest handled here.
 */
export function AudioPlayer({
  filePath,
  seekRequest,
  onTimeUpdate,
}: {
  filePath: string;
  seekRequest: SeekRequest | null;
  onTimeUpdate: (second: number) => void;
}) {
  const audioRef = useRef<HTMLAudioElement>(null);
  const [playing, setPlaying] = useState(false);
  const [current, setCurrent] = useState(0);
  const [duration, setDuration] = useState(0);

  const src = convertFileSrc(filePath);

  useEffect(() => {
    const audio = audioRef.current;
    if (!seekRequest || !audio) return;
    audio.currentTime = seekRequest.at;
    setCurrent(seekRequest.at);
    onTimeUpdate(seekRequest.at);
    void audio.play().then(
      () => setPlaying(true),
      () => setPlaying(false),
    );
    // token distinguishes repeated seeks to the same position.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [seekRequest?.token]);

  function togglePlay() {
    const audio = audioRef.current;
    if (!audio) return;
    if (audio.paused) {
      void audio.play().then(
        () => setPlaying(true),
        () => setPlaying(false),
      );
    } else {
      audio.pause();
      setPlaying(false);
    }
  }

  return (
    <section
      aria-label="播放器"
      className="flex h-13 items-center gap-4 rounded-lg border border-line bg-surface px-5 py-3"
    >
      <button
        type="button"
        onClick={togglePlay}
        aria-label={playing ? "暂停" : "播放"}
        className="flex size-9 shrink-0 items-center justify-center rounded-round bg-primary text-white transition-colors duration-150 hover:bg-primary/90"
      >
        {playing ? <Pause className="size-4" /> : <Play className="ml-0.5 size-4" />}
      </button>

      <span className="w-14 shrink-0 text-right font-mono text-xs font-medium text-text-secondary">
        {formatClock(current)}
      </span>

      <input
        type="range"
        min={0}
        max={duration || 0}
        step={0.1}
        value={Math.min(current, duration || 0)}
        onChange={(e) => {
          const at = Number(e.target.value);
          if (audioRef.current) audioRef.current.currentTime = at;
          setCurrent(at);
          onTimeUpdate(at);
        }}
        aria-label="播放进度"
        className="h-1 flex-1 cursor-pointer appearance-none rounded-round bg-line accent-[#4353FF]"
      />

      <span className="w-14 shrink-0 font-mono text-xs text-text-tertiary">
        {formatClock(duration)}
      </span>

      <Volume2 className="size-4 shrink-0 text-text-tertiary" aria-hidden />

      <audio
        ref={audioRef}
        src={src}
        onTimeUpdate={(e) => {
          setCurrent(e.currentTarget.currentTime);
          onTimeUpdate(e.currentTarget.currentTime);
        }}
        onLoadedMetadata={(e) => setDuration(e.currentTarget.duration)}
        onEnded={() => setPlaying(false)}
        className="hidden"
      />
    </section>
  );
}
