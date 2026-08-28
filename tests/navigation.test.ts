import { describe, expect, it } from "vitest";
import { projectRoute, topLevelPages } from "@/lib/navigation";
import { settingsSections } from "@/lib/settingsNavigation";

describe("desktop navigation", () => {
  it("exposes AI Service alongside the core top-level pages", () => {
    expect(topLevelPages.map((item) => item.page)).toEqual(["home", "library", "ai-service", "settings"]);
    expect(topLevelPages.map((item) => item.label)).toEqual(["首页", "文件库", "AI 服务", "设置"]);
  });

  it("keeps the originating page when opening project detail", () => {
    expect(projectRoute("project-1", "library")).toEqual({
      page: "project",
      projectId: "project-1",
      from: "library",
    });
  });
});

describe("settings section navigation", () => {
  it("follows the actual settings content order", () => {
    expect(settingsSections).toEqual([
      { id: "storage", label: "文件存储" },
      { id: "about", label: "关于" },
    ]);
  });
});
