import { describe, expect, it } from "vitest";
import { hasTextSelection } from "@/lib/textSelection";

describe("text selection", () => {
  it("recognizes only non-empty selected text", () => {
    expect(hasTextSelection({ toString: () => " 选中文字 " } as Selection)).toBe(true);
    expect(hasTextSelection({ toString: () => "   " } as Selection)).toBe(false);
    expect(hasTextSelection(null)).toBe(false);
  });
});
