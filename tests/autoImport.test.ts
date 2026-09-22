import { describe, expect, it, vi } from "vitest";
import { importAndOpenProject } from "@/lib/importProject";

describe("automatic import flow", () => {
  it("creates and opens a project without a separate start confirmation", async () => {
    const calls: string[] = [];
    const createProject = vi.fn(async (path: string, diarizationEnabled: boolean) => {
      calls.push(`create:${path}:${diarizationEnabled}`);
      return { projectId: "project-1", jobId: "job-1" };
    });
    const navigate = vi.fn((route) => calls.push(`open:${route.projectId}`));

    await importAndOpenProject("C:/media/meeting.wav", "home", true, createProject, navigate);

    expect(calls).toEqual(["create:C:/media/meeting.wav:true", "open:project-1"]);
  });
});
