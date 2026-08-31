import { describe, expect, it } from "vitest";
import { formatActionItemsForCopy, removeActionItem, toggleActionItem, updateActionItem } from "@/lib/actionItems";

describe("editable action items", () => {
  const item = { id: "a", text: "处理合同", completed: false };

  it("updates text without losing identity", () => {
    expect(updateActionItem([item], "a", "确认合同")[0]).toEqual({ ...item, text: "确认合同" });
  });

  it("toggles completion and removes by stable id", () => {
    expect(toggleActionItem([item], "a")[0].completed).toBe(true);
    expect(removeActionItem([item], "a")).toEqual([]);
  });

  it("formats all action items for one-click copying", () => {
    expect(formatActionItemsForCopy([
      { id: "a", text: "处理合同", completed: false },
      { id: "b", text: "通知财务", completed: true },
    ])).toBe("待办事项\n☐ 处理合同\n☑ 通知财务");
  });
});
