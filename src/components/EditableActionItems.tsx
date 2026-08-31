import { useEffect, useRef, useState } from "react";
import { Plus, Trash2 } from "lucide-react";
import { removeActionItem, toggleActionItem, updateActionItem } from "@/lib/actionItems";
import type { ActionItem } from "@/types/project";

export function EditableActionItems({ items, onSave, onError }: {
  items: ActionItem[];
  onSave: (items: ActionItem[]) => Promise<void>;
  onError: (message: string) => void;
}) {
  const [draft, setDraft] = useState(items);
  const [saving, setSaving] = useState(false);
  const focusId = useRef<string | null>(null);
  useEffect(() => setDraft(items), [items]);

  async function persist(next: ActionItem[]) {
    setSaving(true);
    try { await onSave(next); }
    catch (error) { onError(String(error)); }
    finally { setSaving(false); }
  }

  function add() {
    const id = crypto.randomUUID();
    focusId.current = id;
    setDraft((current) => [...current, { id, text: "", completed: false }]);
  }

  return <div className="mt-3 space-y-2">
    {draft.map((item) => <div key={item.id} className="group flex items-center gap-2 rounded-md px-2 py-1 transition-colors hover:bg-bg focus-within:bg-primary-soft/40">
      <input type="checkbox" checked={item.completed} onChange={() => { const next = toggleActionItem(draft, item.id); setDraft(next); void persist(next); }} aria-label={`标记完成：${item.text}`} />
      <input
        autoFocus={focusId.current === item.id}
        value={item.text}
        onFocus={(event) => { event.currentTarget.dataset.original = item.text; focusId.current = null; }}
        onChange={(event) => setDraft(updateActionItem(draft, item.id, event.target.value))}
        onBlur={(event) => {
          const text = event.currentTarget.value.trim();
          if (!text) {
            const next = event.currentTarget.dataset.original ? updateActionItem(draft, item.id, event.currentTarget.dataset.original) : removeActionItem(draft, item.id);
            setDraft(next); return;
          }
          const next = updateActionItem(draft, item.id, text); setDraft(next); void persist(next);
        }}
        onKeyDown={(event) => {
          if (event.key === "Enter") event.currentTarget.blur();
          if (event.key === "Escape") { setDraft(updateActionItem(draft, item.id, event.currentTarget.dataset.original ?? item.text)); event.currentTarget.blur(); }
        }}
        className={`min-w-0 flex-1 bg-transparent px-2 py-1.5 text-sm outline-none focus:rounded-sm focus:ring-1 focus:ring-primary ${item.completed ? "text-text-tertiary line-through" : "text-text-primary"}`}
      />
      <button type="button" disabled={saving} aria-label={`删除待办：${item.text}`} onClick={() => { const next = removeActionItem(draft, item.id); setDraft(next); void persist(next); }} className="flex size-7 items-center justify-center rounded-md text-text-tertiary opacity-0 transition-opacity hover:bg-red-50 hover:text-error group-hover:opacity-100 focus:opacity-100"><Trash2 className="size-4" /></button>
    </div>)}
    <button type="button" onClick={add} className="flex items-center gap-2 px-2 py-2 text-sm font-medium text-primary"><Plus className="size-4" />新增待办</button>
  </div>;
}
