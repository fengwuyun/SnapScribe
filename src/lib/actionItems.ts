import type { ActionItem } from "@/types/project";

export const updateActionItem = (items: ActionItem[], id: string, text: string) =>
  items.map((item) => item.id === id ? { ...item, text } : item);

export const toggleActionItem = (items: ActionItem[], id: string) =>
  items.map((item) => item.id === id ? { ...item, completed: !item.completed } : item);

export const removeActionItem = (items: ActionItem[], id: string) =>
  items.filter((item) => item.id !== id);

