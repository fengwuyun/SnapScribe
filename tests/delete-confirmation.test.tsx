import { describe, expect, it } from "vitest";
// @ts-expect-error Node types are intentionally excluded from the desktop app tsconfig.
import { readFileSync } from "node:fs";

describe("delete confirmation", () => {
  it("uses one confirmation step for destructive deletes", () => {
    const source = readFileSync(new URL("../src/components/DeleteConfirmationDialog.tsx", import.meta.url), "utf8");
    expect(source).not.toContain("继续删除");
    expect(source).not.toContain("请再次确认");
    expect(source).toContain('variant="danger"');
    expect(source).toContain("onClick={onConfirm}");
  });
});
