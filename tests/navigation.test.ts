import { describe, expect, it } from "vitest";
import { projectRoute, topLevelPages } from "@/lib/navigation";

describe("desktop navigation", () => {
  it("exposes exactly Home, Library, and Settings as top-level pages", () => {
    expect(topLevelPages.map((item) => item.page)).toEqual(["home", "library", "settings"]);
    expect(topLevelPages.map((item) => item.label)).toEqual(["首页", "文件库", "设置"]);
  });

  it("keeps the originating page when opening project detail", () => {
    expect(projectRoute("project-1", "library")).toEqual({
      page: "project",
      projectId: "project-1",
      from: "library",
    });
  });
});
