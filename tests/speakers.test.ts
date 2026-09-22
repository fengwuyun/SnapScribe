import { describe, expect, it } from "vitest";
import { speakerDisplayName, validateSpeakerDrafts } from "@/lib/speakers";

describe("speaker presentation", () => {
  const speakers = [{ id: "speaker-1", name: "刘德华", colorIndex: 0 }];

  it("resolves the editable display name without exposing the stable id", () => {
    expect(speakerDisplayName("speaker-1", speakers)).toBe("刘德华");
    expect(speakerDisplayName(undefined, speakers)).toBe("");
  });

  it("rejects blank names after trimming", () => {
    expect(validateSpeakerDrafts([{ id: "speaker-1", name: "   ", colorIndex: 0 }])).toEqual({
      ok: false,
      message: "说话人名称不能为空",
    });
  });

  it("allows duplicate display names with a warning", () => {
    expect(validateSpeakerDrafts([
      { id: "speaker-1", name: "张三", colorIndex: 0 },
      { id: "speaker-2", name: "张三", colorIndex: 1 },
    ])).toEqual({
      ok: true,
      warning: "存在同名说话人，保存后仍会保持两个独立说话人",
    });
  });
});
