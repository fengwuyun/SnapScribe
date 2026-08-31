import { describe, expect, it } from "vitest";
import { promoteModel, savedModelRetestDraft } from "@/lib/aiModelWorkflow";

describe("AI model workflow", () => {
  it("moves the chosen model to the front without changing the remaining order", () => {
    expect(promoteModel(["first", "second", "third"], "third")).toEqual([
      "third",
      "first",
      "second",
    ]);
  });

  it("retests the saved model only after a successful connection test", () => {
    const draft = {
      name: "GLM",
      protocol: "openai-chat" as const,
      baseUrl: "https://open.bigmodel.cn/api/paas/v4",
      modelId: "glm-4.7-flash",
      endpointMode: "auto" as const,
      authType: "bearer" as const,
      timeoutSecs: 60,
      enabled: true,
    };

    expect(savedModelRetestDraft(draft, "saved-id", { status: "available", message: "连接成功" }))
      .toEqual({ ...draft, id: "saved-id" });
    expect(savedModelRetestDraft(draft, "saved-id", { status: "auth-failed", message: "鉴权失败" }))
      .toBeNull();
    expect(savedModelRetestDraft(draft, "saved-id", null)).toBeNull();
  });
});
