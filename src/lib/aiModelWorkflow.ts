import type { AIModelDraft, AIModelStatus } from "@/types/project";

export function promoteModel(orderedIds: string[], modelId: string): string[] {
  if (!orderedIds.includes(modelId)) return orderedIds;
  return [modelId, ...orderedIds.filter((id) => id !== modelId)];
}

export function savedModelRetestDraft(
  draft: AIModelDraft,
  savedModelId: string,
  testedStatus: AIModelStatus | null,
): AIModelDraft | null {
  return testedStatus?.status === "available" ? { ...draft, id: savedModelId } : null;
}
