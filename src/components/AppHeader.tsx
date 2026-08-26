import { History } from "lucide-react";

/** App header per UI_DESIGN_SPEC §7: 56px, white, bottom divider, history entry right. */
export function AppHeader({ onOpenHistory }: { onOpenHistory: () => void }) {
  return (
    <header className="flex h-14 shrink-0 items-center justify-between border-b border-divider bg-surface px-8">
      <div className="flex items-center gap-2">
        <span className="text-base font-bold tracking-tight text-text-primary">SnapScribe</span>
        <span className="text-xs text-text-tertiary">本地音视频转文字</span>
      </div>
      <button
        type="button"
        onClick={onOpenHistory}
        aria-label="打开历史记录"
        className="flex h-8 items-center gap-1.5 rounded-sm px-2 text-xs font-medium text-text-secondary transition-colors duration-150 hover:bg-primary-soft hover:text-primary"
      >
        <History className="size-4" aria-hidden />
        历史
      </button>
    </header>
  );
}
