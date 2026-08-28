import type { ReactNode } from "react";
import { AppSidebar } from "@/components/AppSidebar";
import type { AppRoute, TopLevelPage } from "@/lib/navigation";

export function AppShell({ route, onNavigate, children }: { route: { page: TopLevelPage }; onNavigate: (route: AppRoute) => void; children: ReactNode }) {
  return <div className="flex h-full bg-bg"><AppSidebar activePage={route.page} onNavigate={(page) => onNavigate({ page })} /><main className="min-w-0 flex-1 overflow-y-auto px-10 py-8">{children}</main></div>;
}
