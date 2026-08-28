import { useEffect, useRef, useState } from "react";
import { Check, ChevronDown } from "lucide-react";
import { cn } from "@/lib/utils";
import type { ProjectSort } from "@/types/project";

const OPTIONS: Array<{ value: ProjectSort; label: string }> = [
  { value: "recentlyUpdated", label: "最近更新" },
  { value: "createdAt", label: "创建时间" },
  { value: "name", label: "名称" },
  { value: "duration", label: "时长" },
];

export function SortDropdown({ value, onChange }: { value: ProjectSort; onChange: (value: ProjectSort) => void }) {
  const [open, setOpen] = useState(false);
  const rootRef = useRef<HTMLDivElement>(null);
  const selected = OPTIONS.find((option) => option.value === value) ?? OPTIONS[0];

  useEffect(() => {
    function closeOnOutsideClick(event: MouseEvent) {
      if (!rootRef.current?.contains(event.target as Node)) setOpen(false);
    }
    function closeOnEscape(event: KeyboardEvent) {
      if (event.key === "Escape") setOpen(false);
    }
    document.addEventListener("mousedown", closeOnOutsideClick);
    document.addEventListener("keydown", closeOnEscape);
    return () => {
      document.removeEventListener("mousedown", closeOnOutsideClick);
      document.removeEventListener("keydown", closeOnEscape);
    };
  }, []);

  return (
    <div ref={rootRef} className="relative w-40 shrink-0">
      <button
        type="button"
        aria-haspopup="listbox"
        aria-expanded={open}
        onClick={() => setOpen((current) => !current)}
        className={cn(
          "flex h-10 w-full items-center justify-between gap-3 rounded-md border bg-surface px-3.5 text-sm font-medium shadow-sm outline-none transition-all duration-150",
          open ? "border-primary ring-2 ring-primary/10" : "border-line hover:border-primary/50",
        )}
      >
        <span>{selected.label}</span>
        <ChevronDown className={cn("size-4 text-text-tertiary transition-transform duration-150", open && "rotate-180 text-primary")} />
      </button>
      {open && (
        <div role="listbox" aria-label="项目排序" className="absolute right-0 top-12 z-20 w-full origin-top-right animate-fade-in overflow-hidden rounded-lg border border-line bg-surface p-1.5 shadow-md">
          {OPTIONS.map((option) => {
            const active = option.value === value;
            return (
              <button
                key={option.value}
                type="button"
                role="option"
                aria-selected={active}
                onClick={() => { onChange(option.value); setOpen(false); }}
                className={cn(
                  "flex w-full items-center justify-between rounded-md px-3 py-2 text-left text-sm transition-colors duration-150",
                  active ? "bg-primary-soft font-semibold text-primary" : "text-text-primary hover:bg-bg",
                )}
              >
                {option.label}
                {active && <Check className="size-4" />}
              </button>
            );
          })}
        </div>
      )}
    </div>
  );
}
