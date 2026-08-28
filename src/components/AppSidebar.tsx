import { FileText, Home, Settings } from "lucide-react";
import { topLevelPages, type TopLevelPage } from "@/lib/navigation";
import { cn } from "@/lib/utils";

const icons = { home: Home, library: FileText, settings: Settings } as const;

export function AppSidebar({ activePage, onNavigate }: { activePage: TopLevelPage; onNavigate: (page: TopLevelPage) => void }) {
  return <aside className="flex w-48 shrink-0 flex-col border-r border-divider bg-surface px-4 py-5"><div className="mb-8 flex items-center gap-3 px-2"><span className="flex size-8 items-center justify-center rounded-sm bg-primary text-sm font-bold text-white">S</span><div><p className="text-sm font-bold">SnapScribe</p><p className="text-[11px] text-text-tertiary">本地音视频转文字</p></div></div><nav aria-label="一级导航" className="space-y-2">{topLevelPages.map((item) => { const Icon = icons[item.page]; const selected = activePage === item.page; return <button key={item.page} type="button" onClick={() => onNavigate(item.page)} className={cn("flex h-10 w-full items-center gap-3 rounded-md px-3 text-sm transition-colors duration-150", selected ? "bg-primary-soft font-semibold text-primary" : "text-text-secondary hover:bg-bg hover:text-text-primary")}><Icon className="size-4" aria-hidden />{item.label}</button>; })}</nav></aside>;
}
