import { describe, expect, it } from "vitest";
import { removeActionItem, toggleActionItem, updateActionItem } from "@/lib/actionItems";

describe("editable action items", () => {
  const item = { id: "a", text: "处理合同", completed: false };

  it("updates text without losing identity", () => {
    expect(updateActionItem([item], "a", "确认合同")[0]).toEqual({ ...item, text: "确认合同" });
  });

  it("toggles completion and removes by stable id", () => {
    expect(toggleActionItem([item], "a")[0].completed).toBe(true);
    expect(removeActionItem([item], "a")).toEqual([]);
  });
});
