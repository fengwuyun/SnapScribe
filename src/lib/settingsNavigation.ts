export type SettingsSectionId = "storage" | "about";

export const settingsSections: ReadonlyArray<{ id: SettingsSectionId; label: string }> = [
  { id: "storage", label: "文件存储" },
  { id: "about", label: "关于" },
];
