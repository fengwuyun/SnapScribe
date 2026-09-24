import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it, vi } from "vitest";
import { ProjectContextMenu } from "@/components/ProjectContextMenu";
import type { ProjectListItem } from "@/types/project";

const project: ProjectListItem = {
  id: "project-1",
  name: "访谈记录",
  createdAt: "2026-09-24T10:00:00Z",
  updatedAt: "2026-09-24T10:00:00Z",
  status: "completed",
  durationSecs: 120,
  media: {
    origin: "imported",
    storage: "reference",
    path: "C:\\media\\interview.mp3",
    originalFileName: "interview.mp3",
    sizeBytes: 1024,
    container: "mp3",
  },
  mediaAvailable: true,
};

describe("project context menu", () => {
  it("offers the actions for the selected library project", () => {
    const markup = renderToStaticMarkup(
      <ProjectContextMenu
        project={project}
        x={80}
        y={120}
        onClose={vi.fn()}
        onOpen={vi.fn()}
        onRename={vi.fn()}
        onExport={vi.fn()}
        onOpenLocation={vi.fn()}
        onDelete={vi.fn()}
      />,
    );

    expect(markup).toContain('role="menu"');
    for (const label of ["打开项目", "重命名", "导出 TXT", "打开原文件位置", "删除项目"]) {
      expect(markup).toContain(label);
    }
  });

  it("disables opening the original location when media is unavailable", () => {
    const markup = renderToStaticMarkup(
      <ProjectContextMenu
        project={{ ...project, mediaAvailable: false }}
        x={80}
        y={120}
        onClose={vi.fn()}
        onOpen={vi.fn()}
        onRename={vi.fn()}
        onExport={vi.fn()}
        onOpenLocation={vi.fn()}
        onDelete={vi.fn()}
      />,
    );

    expect(markup).toMatch(/disabled=""[^>]*>[^<]*(?:<[^>]+>)*打开原文件位置|打开原文件位置[\s\S]*disabled=""/);
  });
});
