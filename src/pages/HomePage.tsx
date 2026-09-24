import { useCallback, useEffect, useRef, useState } from "react";
import { Mic, Users } from "lucide-react";
import { DropZone } from "@/components/DropZone";
import { RecordingPanel } from "@/components/RecordingPanel";
import { Button } from "@/components/ui/Button";
import { importAndOpenProject } from "@/lib/importProject";
import type { AppRoute } from "@/lib/navigation";
import * as ipc from "@/lib/tauri";
import type { ProjectListItem } from "@/types/project";
import type { RecordingStartResult } from "@/types/project";
import { formatClock, truncateProjectName } from "@/lib/format";
import { appendWaveformLevel } from "@/lib/recordingUi";
import { readDiarizationPreference, writeDiarizationPreference } from "@/lib/homePreferences";
import type { TranscriptSegment, TranscriptSpeaker } from "@/types/transcript";

export function HomePage({ onNavigate }: { onNavigate: (route: AppRoute) => void }) {
  const [recent, setRecent] = useState<ProjectListItem[]>([]);
  const [error, setError] = useState("");
  const [importing, setImporting] = useState(false);
  const [recording, setRecording] = useState<RecordingStartResult | null>(null);
  const [recordingSecs, setRecordingSecs] = useState(0);
  const [recordingPaused, setRecordingPaused] = useState(false);
  const [stoppingRecording, setStoppingRecording] = useState(false);
  const [waveformLevels, setWaveformLevels] = useState<number[]>([]);
  const [liveSegments, setLiveSegments] = useState<TranscriptSegment[]>([]);
  const [liveSpeakers, setLiveSpeakers] = useState<TranscriptSpeaker[]>([]);
  const [diarizationEnabled, setDiarizationEnabled] = useState(readDiarizationPreference);
  const recordingRef = useRef<RecordingStartResult | null>(null);

  useEffect(() => {
    void ipc.projectList("", "recentlyUpdated").then((items) => setRecent(items.slice(0, 5))).catch((err) => setError(String(err)));
  }, []);

  useEffect(() => { recordingRef.current = recording; }, [recording]);

  useEffect(() => {
    if (!recording || recordingPaused) return;
    const timer = window.setInterval(() => setRecordingSecs((value) => value + 1), 1000);
    return () => window.clearInterval(timer);
  }, [recording, recordingPaused]);

  useEffect(() => {
    const cleanups: Array<() => void> = [];
    void Promise.all([
      ipc.listenRecordingLevel((event) => {
        if (event.payload.recordingId === recordingRef.current?.recordingId) {
          setWaveformLevels((levels) => appendWaveformLevel(levels, event.payload.level));
        }
      }),
      ipc.listenSegments((event) => {
        if (event.payload.projectId === recordingRef.current?.projectId) {
          setLiveSegments((segments) => [...segments, ...event.payload.segments]);
          setLiveSpeakers((current) => {
            const merged = new Map(current.map((speaker) => [speaker.id, speaker]));
            event.payload.speakers.forEach((speaker) => merged.set(speaker.id, speaker));
            return [...merged.values()];
          });
        }
      }),
      ipc.listenFailed((event) => {
        if (event.payload.projectId === recordingRef.current?.projectId) setError(event.payload.message);
      }),
    ]).then((items) => cleanups.push(...items));
    return () => {
      cleanups.forEach((cleanup) => cleanup());
      const active = recordingRef.current;
      if (active) void ipc.recordingCancel(active.recordingId).catch(() => undefined);
    };
  }, []);

  async function toggleRecording() {
    setError("");
    try {
      if (!recording) {
        const started = await ipc.recordingStart(diarizationEnabled);
        setRecording(started);
        setRecordingSecs(0);
        setRecordingPaused(false);
        setWaveformLevels([]);
        setLiveSegments([]);
        setLiveSpeakers([]);
      } else {
        await stopRecording();
      }
    } catch (err) { setError(err instanceof Error ? err.message : String(err)); }
  }

  async function toggleRecordingPause() {
    if (!recording || stoppingRecording) return;
    const next = !recordingPaused;
    try {
      await ipc.recordingSetPaused(recording.recordingId, next);
      setRecordingPaused(next);
    } catch (err) { setError(err instanceof Error ? err.message : String(err)); }
  }

  async function stopRecording() {
    if (!recording || stoppingRecording) return;
    setStoppingRecording(true);
    setError("");
    try {
      const result = await ipc.recordingStop(recording.recordingId);
      recordingRef.current = null;
      setRecording(null);
      onNavigate({ page: "project", projectId: result.projectId, from: "home" });
    } catch (err) { setError(err instanceof Error ? err.message : String(err)); }
    finally { setStoppingRecording(false); }
  }

  async function cancelRecording() {
    if (!recording) return;
    try { await ipc.recordingCancel(recording.recordingId); recordingRef.current = null; setRecording(null); setRecordingSecs(0); setRecordingPaused(false); setLiveSegments([]); setLiveSpeakers([]); setWaveformLevels([]); }
    catch (err) { setError(err instanceof Error ? err.message : String(err)); }
  }

  const importFile = useCallback(async (path: string) => {
    setError(""); setImporting(true);
    try { await importAndOpenProject(path, "home", diarizationEnabled, ipc.createProjectFromMedia, onNavigate); }
    catch (err) { setError(err instanceof Error ? err.message : String(err)); }
    finally { setImporting(false); }
  }, [diarizationEnabled, onNavigate]);

  return <div className="mx-auto w-full max-w-[980px]">
    <header className="mb-6 flex items-center justify-between"><div><h1 className="text-2xl font-bold">快速开始</h1><p className="mt-1 text-sm text-text-secondary">{recording ? "录音正在本地保存并生成近实时字幕" : "导入文件后将自动开始本地转录"}</p></div>{recording ? <span className="inline-flex h-9 items-center gap-2 rounded-md border border-line bg-surface px-3 text-sm font-medium text-text-secondary"><span className={`size-2 rounded-round ${recordingPaused ? "bg-warning" : "bg-error"}`} />{recordingPaused ? "已暂停" : "录音中"} · <span className="font-mono">{formatClock(recordingSecs)}</span></span> : <div className="flex items-center gap-3"><label className="flex h-9 cursor-pointer items-center gap-2 rounded-md border border-line bg-surface px-3 text-sm text-text-secondary transition-colors hover:border-primary/40"><Users className="size-4 text-primary" /><span>区分说话人</span><input type="checkbox" className="peer sr-only" checked={diarizationEnabled} onChange={(event) => { const enabled = event.target.checked; setDiarizationEnabled(enabled); writeDiarizationPreference(enabled); }} /><span className="relative h-5 w-9 rounded-round bg-line transition-colors peer-checked:bg-primary after:absolute after:left-0.5 after:top-0.5 after:size-4 after:rounded-round after:bg-white after:shadow-sm after:transition-transform peer-checked:after:translate-x-4" /></label><Button onClick={() => void toggleRecording()}><Mic className="size-4" aria-hidden />开始录音</Button></div>}</header>
    {recording ? <RecordingPanel seconds={recordingSecs} paused={recordingPaused} stopping={stoppingRecording} levels={waveformLevels} segments={liveSegments} speakers={liveSpeakers} diarizationEnabled={diarizationEnabled} onTogglePause={() => void toggleRecordingPause()} onStop={() => void stopRecording()} onCancel={() => void cancelRecording()} /> : <DropZone onPick={(path) => void importFile(path)} onInvalidFile={(name) => setError(`暂不支持该格式：${name}`)} />}
    {importing && <p className="mt-3 text-sm text-primary">正在创建项目并启动转录…</p>}{error && <p className="mt-3 text-sm text-error">{error}</p>}
    {!recording && <section className="mt-8"><div className="mb-3 flex items-center justify-between"><h2 className="text-lg font-semibold">最近文件</h2><button type="button" onClick={() => onNavigate({ page: "library" })} className="text-sm font-medium text-primary">查看全部 →</button></div><div className="overflow-hidden rounded-lg border border-line bg-surface">{recent.length === 0 ? <p className="py-10 text-center text-sm text-text-tertiary">还没有转录项目</p> : recent.map((project) => <button key={project.id} type="button" title={project.name} onClick={() => onNavigate({ page: "project", projectId: project.id, from: "home" })} className="flex w-full items-center justify-between border-b border-divider px-5 py-4 text-left last:border-b-0 hover:bg-[#F8F9FF]"><span><span className="block text-sm font-semibold">{truncateProjectName(project.name)}</span><span className="mt-1 block text-xs text-text-tertiary">{project.media.storage === "reference" ? "导入文件（引用）" : "SnapScribe 文件"}</span></span><span className="font-mono text-xs text-text-secondary">{formatClock(project.durationSecs)}</span></button>)}</div></section>}
  </div>;
}
