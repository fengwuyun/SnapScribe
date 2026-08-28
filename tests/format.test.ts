import { describe, expect, it } from "vitest";
import {
  buildExportName,
  buildSummaryExportName,
  buildSummaryText,
  formatClock,
  formatDateTime,
  formatProjectCreatedAt,
  formatSize,
  fullText,
  isAcceptedFile,
  splitHighlightParts,
  truncateProjectName,
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

describe("formatProjectCreatedAt", () => {
  it("formats persisted unix seconds as local date and time", () => {
    expect(formatProjectCreatedAt("1787882400")).toMatch(/^2026-08-28 \d{2}:\d{2}$/);
  });

  it("returns a dash for malformed timestamps", () => {
    expect(formatProjectCreatedAt("not-a-time")).toBe("—");
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

describe("project presentation", () => {
  it("shows the first ten Unicode characters followed by an ellipsis", () => {
    expect(truncateProjectName("十个字项目名称刚好呀", 10)).toBe("十个字项目名称刚好呀");
    expect(truncateProjectName("这是一个超过十个字的项目名称", 10)).toBe("这是一个超过十个字的…");
    expect(truncateProjectName("🎙️录音项目名称很长很长", 10)).toBe("🎙️录音项目名称很长…");
  });

  it("splits every case-insensitive search match for theme highlighting", () => {
    expect(splitHighlightParts("Meeting meeting 复盘", "MEETING")).toEqual([
      { text: "Meeting", highlighted: true },
      { text: " ", highlighted: false },
      { text: "meeting", highlighted: true },
      { text: " 复盘", highlighted: false },
    ]);
    expect(splitHighlightParts("会员分账讨论", "分账")).toEqual([
      { text: "会员", highlighted: false },
      { text: "分账", highlighted: true },
      { text: "讨论", highlighted: false },
    ]);
  });

  it("adds the AI summary suffix before the text extension", () => {
    expect(buildSummaryExportName("产品评审会议.m4a")).toBe("产品评审会议-AI总结.txt");
  });

  it("builds copy and export text from all summary sections", () => {
    expect(buildSummaryText({
      summary: "本次确定上线计划。",
      keyPoints: ["周五发布", "保留回滚方案"],
      actionItems: ["完成验收"],
    })).toBe("摘要\n本次确定上线计划。\n\n关键要点\n- 周五发布\n- 保留回滚方案\n\n待办事项\n- 完成验收");
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
