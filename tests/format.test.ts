import { describe, expect, it } from "vitest";
import {
  buildExportName,
  formatClock,
  formatDateTime,
  formatSize,
  fullText,
  isAcceptedFile,
} from "@/lib/format";

describe("formatClock", () => {
  it("formats minutes and seconds under one hour", () => {
    expect(formatClock(0)).toBe("00:00");
    expect(formatClock(65)).toBe("01:05");
    expect(formatClock(3599.6)).toBe("59:59");
  });

  it("switches to hours at 3600 seconds", () => {
    expect(formatClock(3600)).toBe("1:00:00");
    expect(formatClock(4925)).toBe("1:22:05");
  });

  it("clamps negative input", () => {
    expect(formatClock(-3)).toBe("00:00");
  });
});

describe("formatSize", () => {
  it("picks the human unit", () => {
    expect(formatSize(500)).toBe("500 B");
    expect(formatSize(856 * 1024)).toBe("856 KB");
    expect(formatSize(128 * 1024 ** 2)).toBe("128 MB");
    expect(formatSize(1.5 * 1024 ** 3)).toBe("1.5 GB");
  });
});

describe("formatDateTime", () => {
  it("renders local date with weekday", () => {
    // 2026-08-26 is a Wednesday in Asia/Shanghai if the machine TZ differs this
    // only checks structure.
    const text = formatDateTime(new Date(2026, 7, 26, 14, 30).getTime());
    expect(text).toMatch(/^\d{4}-\d{2}-\d{2} \d{2}:\d{2} 周[日一二三四五六]$/);
  });

  it("returns empty for invalid timestamps", () => {
    expect(formatDateTime(0)).toBe("");
    expect(formatDateTime(Number.NaN)).toBe("");
  });
});

describe("buildExportName", () => {
  it("replaces the source extension", () => {
    expect(buildExportName("meeting.mp4", "txt")).toBe("meeting.txt");
    expect(buildExportName("录音.20260826.m4a", "srt")).toBe("录音.20260826.srt");
  });

  it("falls back to a generic stem", () => {
    expect(buildExportName("", "srt")).toBe("transcript.srt");
  });
});

describe("fullText", () => {
  it("joins segment texts line by line and trims edges", () => {
    expect(fullText([{ text: " 第一句 " }, { text: "第二句" }])).toBe("第一句\n第二句");
    expect(fullText([])).toBe("");
  });
});

describe("isAcceptedFile", () => {
  it("accepts supported extensions case-insensitively", () => {
    expect(isAcceptedFile("A.MP4")).toBe(true);
    expect(isAcceptedFile("b.Mov")).toBe(true);
    expect(isAcceptedFile("c.wav")).toBe(true);
  });

  it("rejects unsupported extensions", () => {
    expect(isAcceptedFile("x.exe")).toBe(false);
    expect(isAcceptedFile("noext")).toBe(false);
  });
});
