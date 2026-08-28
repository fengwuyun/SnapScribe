export type TopLevelPage = "home" | "library" | "settings";

export type AppRoute =
  | { page: TopLevelPage }
  | { page: "project"; projectId: string; from: "home" | "library" };

export const topLevelPages = [
  { page: "home", label: "首页" },
  { page: "library", label: "文件库" },
  { page: "settings", label: "设置" },
] as const;

export function projectRoute(
  projectId: string,
  from: "home" | "library",
): AppRoute {
  return { page: "project", projectId, from };
}
