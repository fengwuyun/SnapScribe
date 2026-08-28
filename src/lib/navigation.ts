export type TopLevelPage = "home" | "library" | "ai-service" | "settings";

export type AppRoute =
  | { page: Exclude<TopLevelPage, "ai-service"> }
  | { page: "ai-service"; focus?: "models"; returnToProjectId?: string }
  | { page: "project"; projectId: string; from: "home" | "library" };

export const topLevelPages = [
  { page: "home", label: "首页" },
  { page: "library", label: "文件库" },
  { page: "ai-service", label: "AI 服务" },
  { page: "settings", label: "设置" },
] as const;

export function projectRoute(
  projectId: string,
  from: "home" | "library",
): AppRoute {
  return { page: "project", projectId, from };
}
