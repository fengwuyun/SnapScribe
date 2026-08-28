import { describe, expect, it } from "vitest";
import {
  initialTranscriptionState,
  transcriptionReducer,
} from "@/hooks/useTranscription";
import type { MediaInfo, TranscriptSegment } from "@/types/transcript";

const fileInfo: MediaInfo = {
  fileName: "meeting.mp4",
  sizeBytes: 1024,
  durationSecs: 120,
  container: "mp4",
};

const segment = (id: string): TranscriptSegment => ({
  id,
  start: 0,
  end: 1,
  text: "内容",
});

function readyState() {
  return transcriptionReducer(initialTranscriptionState, {
    type: "fileSelected",
    path: "C:/media/meeting.mp4",
    info: fileInfo,
  });
}

describe("transcriptionReducer", () => {
  it("moves empty → ready on file selection and clears prior results", () => {
    const used = transcriptionReducer(
      { ...initialTranscriptionState, phase: "completed", segments: [segment("s")] },
      { type: "fileSelected", path: "x", info: fileInfo },
    );
    expect(used.phase).toBe("ready");
    expect(used.segments).toEqual([]);
  });

  it("starts transcribing with a fresh job id", () => {
    const started = transcriptionReducer(readyState(), {
      type: "started",
      jobId: "job-1",
    });
    expect(started.phase).toBe("transcribing");
    expect(started.jobId).toBe("job-1");
    expect(started.segments).toEqual([]);
  });

  it("appends incremental segments for the active job only", () => {
    let state = transcriptionReducer(readyState(), { type: "started", jobId: "job-1" });
    state = transcriptionReducer(state, {
      type: "segments",
      jobId: "job-other",
      segments: [segment("stale")],
    });
    expect(state.segments).toEqual([]);
    state = transcriptionReducer(state, {
      type: "segments",
      jobId: "job-1",
      segments: [segment("a"), segment("b")],
    });
    expect(state.segments.map((s) => s.id)).toEqual(["a", "b"]);
  });

  it("ignores segments for another project even when the job id matches", () => {
    let state = transcriptionReducer(readyState(), {
      type: "started",
      projectId: "project-1",
      jobId: "job-1",
    });
    state = transcriptionReducer(state, {
      type: "segments",
      projectId: "project-other",
      jobId: "job-1",
      segments: [segment("wrong-project")],
    });
    expect(state.segments).toEqual([]);
  });

  it("tracks progress only for the active job", () => {
    let state = transcriptionReducer(readyState(), { type: "started", jobId: "job-1" });
    const event = {
      projectId: "",
      jobId: "job-1",
      stage: "transcribing" as const,
      percent: 10,
      processedSeconds: 12,
      totalSeconds: 120,
      segmentIndex: 1,
      segmentCount: 2,
    };
    state = transcriptionReducer(state, { type: "progress", jobId: "job-other", event });
    expect(state.progress).toBeNull();
    state = transcriptionReducer(state, { type: "progress", jobId: "job-1", event });
    expect(state.progress?.percent).toBe(10);
  });

  it("ignores events that race ahead of the started dispatch", () => {
    // Backend may emit before the UI dispatches `started`; the event is dropped
    // and the next in-job event takes over normally.
    let state = readyState();
    state = transcriptionReducer(state, {
      type: "segments",
      jobId: "job-early",
      segments: [segment("race")],
    });
    expect(state.phase).toBe("ready");
    expect(state.segments).toEqual([]);
    state = transcriptionReducer(state, { type: "started", jobId: "job-1" });
    expect(state.segments).toEqual([]);
  });

  it("completes with authoritative result segments", () => {
    let state = transcriptionReducer(readyState(), { type: "started", jobId: "job-1" });
    state = transcriptionReducer(state, {
      type: "completed",
      result: { fileName: "meeting.mp4", duration: 120, segments: [segment("final")] },
    });
    expect(state.phase).toBe("completed");
    expect(state.result?.fileName).toBe("meeting.mp4");
    expect(state.segments.map((s) => s.id)).toEqual(["final"]);
  });

  it("fails into error phase from transcribing", () => {
    let state = transcriptionReducer(readyState(), { type: "started", jobId: "job-1" });
    state = transcriptionReducer(state, { type: "failed", message: "boom" });
    expect(state.phase).toBe("error");
    expect(state.error).toBe("boom");
  });

  it("cancel returns to ready, keeps file, drops partial text", () => {
    let state = transcriptionReducer(readyState(), { type: "started", jobId: "job-1" });
    state = transcriptionReducer(state, {
      type: "segments",
      jobId: "job-1",
      segments: [segment("partial")],
    });
    state = transcriptionReducer(state, { type: "canceled" });
    expect(state.phase).toBe("ready");
    expect(state.file?.fileName).toBe("meeting.mp4");
    expect(state.segments).toEqual([]);
    expect(state.jobId).toBeNull();
  });

  it("cancel outside transcribing is a no-op", () => {
    const ready = readyState();
    expect(transcriptionReducer(ready, { type: "canceled" })).toBe(ready);
  });
});
