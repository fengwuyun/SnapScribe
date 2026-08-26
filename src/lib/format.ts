/** Formatting helpers shared by the UI. Pure functions — unit-tested in tests/format.test.ts. */

/** `MM:SS`, or `H:MM:SS` once the media is at least one hour long. */
export function formatClock(seconds: number): string {
  const total = Math.max(0, Math.floor(seconds));
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = total % 60;
  const mm = String(m).padStart(2, "0");
  const ss = String(s).padStart(2, "0");
  return h > 0 ? `${h}:${mm}:${ss}` : `${mm}:${ss}`;
}

/** Human-readable file size, e.g. `128 MB` or `856 KB`. */
export function formatSize(bytes: number): string {
  if (bytes >= 1024 ** 3) return `${trim(bytes / 1024 ** 3)} GB`;
  if (bytes >= 1024 ** 2) return `${trim(bytes / 1024 ** 2)} MB`;
  if (bytes >= 1024) return `${trim(bytes / 1024)} KB`;
  return `${bytes} B`;
}

function trim(value: number): string {
  return value.toFixed(value >= 100 ? 0 : 1).replace(/\.0$/, "");
}

const WEEKDAYS = ["日", "一", "二", "三", "四", "五", "六"] as const;

/** `2026-08-26 14:30 周三` style local timestamp for history entries. */
export function formatDateTime(ms: number): string {
  if (!Number.isFinite(ms) || ms <= 0) return "";
  const d = new Date(ms);
  const pad = (n: number) => String(n).padStart(2, "0");
  return (
    `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ` +
    `${pad(d.getHours())}:${pad(d.getMinutes())} 周${WEEKDAYS[d.getDay()]}`
  );
}

/** Default export file name derived from the source file: `meeting.txt` / `meeting.srt`. */
export function buildExportName(sourceFileName: string, ext: "txt" | "srt"): string {
  const stem = sourceFileName.replace(/\.[^.]+$/, "") || "transcript";
  return `${stem}.${ext}`;
}

/** Plain text of all segments joined by newlines (copy & TXT export content). */
export function fullText(segments: { text: string }[]): string {
  return segments.map((s) => s.text.trim()).join("\n");
}

/** Extensions accepted in the drop zone; mirrors the Rust open-file filter. */
export const ACCEPTED_EXTENSIONS = ["mp4", "mov", "mkv", "mp3", "m4a", "wav"] as const;

export function isAcceptedFile(fileName: string): boolean {
  const ext = fileName.split(".").pop()?.toLowerCase() ?? "";
  return (ACCEPTED_EXTENSIONS as readonly string[]).includes(ext);
}
