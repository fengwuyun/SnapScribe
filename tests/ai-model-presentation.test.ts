import { describe, expect, it } from "vitest";
import {
  SAVED_API_KEY_MASK,
  getApiKeyDisplayValue,
  getAIModelStatusLabel,
} from "@/lib/aiModelPresentation";

describe("AI model presentation", () => {
  it("shows a display-only mask for a saved API key until the user edits it", () => {
    expect(SAVED_API_KEY_MASK).toBe("******");
    expect(getApiKeyDisplayValue(true, undefined, false)).toBe("******");
    expect(getApiKeyDisplayValue(true, undefined, true)).toBe("");
    expect(getApiKeyDisplayValue(true, "new-secret", true)).toBe("new-secret");
  });

  it("labels a successfully tested model as connected", () => {
    expect(getAIModelStatusLabel("available")).toBe("连通");
  });
});
