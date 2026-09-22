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

/** Project timestamps are persisted as Unix seconds. */
export function formatProjectCreatedAt(value: string): string {
  const seconds = Number(value);
  if (!Number.isFinite(seconds) || seconds <= 0) return "—";
  const date = new Date(seconds * 1000);
  const pad = (part: number) => String(part).padStart(2, "0");
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())} ${pad(date.getHours())}:${pad(date.getMinutes())}`;
}

/** Default export file name derived from the source file: `meeting.txt` / `meeting.srt`. */
export function buildExportName(sourceFileName: string, ext: "txt" | "srt"): string {
  const stem = sourceFileName.replace(/\.[^.]+$/, "") || "transcript";
  return `${stem}.${ext}`;
}

/** Summary export keeps the project stem and adds a visible content suffix. */
export function buildSummaryExportName(sourceFileName: string): string {
  const stem = sourceFileName.replace(/\.[^.]+$/, "") || "transcript";
  return `${stem}-AI总结.txt`;
}

/** Display-only truncation. The persisted project name is never changed. */
export function truncateProjectName(name: string, maxLength = 10): string {
  const characters = Array.from(name);
  return characters.length > maxLength
    ? `${characters.slice(0, maxLength).join("")}…`
    : name;
}

export interface HighlightPart {
  text: string;
  highlighted: boolean;
}

/** Split visible text without regular expressions so user input stays literal. */
export function splitHighlightParts(text: string, query: string): HighlightPart[] {
  const needle = query.trim();
  if (!needle) return [{ text, highlighted: false }];
  const source = text.toLocaleLowerCase();
  const target = needle.toLocaleLowerCase();
  const parts: HighlightPart[] = [];
  let cursor = 0;
  while (cursor < text.length) {
    const index = source.indexOf(target, cursor);
    if (index < 0) {
      parts.push({ text: text.slice(cursor), highlighted: false });
      break;
    }
    if (index > cursor) parts.push({ text: text.slice(cursor, index), highlighted: false });
    parts.push({ text: text.slice(index, index + needle.length), highlighted: true });
    cursor = index + needle.length;
  }
  return parts.length > 0 ? parts : [{ text, highlighted: false }];
}

export function buildSummaryText(summary: {
  summary: string;
  keyPoints: string[];
  actionItems: Array<{ text: string }>;
}): string {
  const keyPoints = summary.keyPoints.map((item) => `- ${item.trim()}`).join("\n");
  const actionItems = summary.actionItems.map((item) => `- ${item.text.trim()}`).join("\n");
  return [
    `摘要\n${summary.summary.trim()}`,
    `关键要点\n${keyPoints}`,
    `待办事项\n${actionItems}`,
  ].join("\n\n");
}

/** Plain text of all segments joined by newlines (copy & TXT export content). */
export function fullText<
  T extends { text: string; start?: number; speakerId?: string },
  S extends { id: string; name: string },
>(
  segments: T[],
  speakers: S[] = [] as S[],
): string {
  return segments.map((segment) => {
    const name = segment.speakerId
      ? speakers.find((speaker) => speaker.id === segment.speakerId)?.name.trim()
      : "";
    return name && segment.start !== undefined
      ? `[${formatClock(segment.start)}] ${name}：${segment.text.trim()}`
      : segment.text.trim();
  }).join("\n");
}

/** Extensions accepted in the drop zone; mirrors the Rust open-file filter. */
export const ACCEPTED_EXTENSIONS = ["mp4", "mov", "mkv", "mp3", "m4a", "wav"] as const;

export function isAcceptedFile(fileName: string): boolean {
  const ext = fileName.split(".").pop()?.toLowerCase() ?? "";
  return (ACCEPTED_EXTENSIONS as readonly string[]).includes(ext);
}
