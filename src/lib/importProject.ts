import type { AppRoute } from "@/lib/navigation";
import type { StartProjectResult } from "@/types/project";

export async function importAndOpenProject(
  path: string,
  from: "home" | "library",
  diarizationEnabled: boolean,
  createProject: (path: string, diarizationEnabled: boolean) => Promise<StartProjectResult>,
  navigate: (route: AppRoute) => void,
): Promise<StartProjectResult> {
  const result = await createProject(path, diarizationEnabled);
  navigate({ page: "project", projectId: result.projectId, from });
  return result;
}
