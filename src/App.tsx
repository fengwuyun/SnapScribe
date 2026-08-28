import { useState } from "react";
import { AppShell } from "@/components/AppShell";
import { HomePage } from "@/pages/HomePage";
import { LibraryPage } from "@/pages/LibraryPage";
import { ProjectDetailPage } from "@/pages/ProjectDetailPage";
import { SettingsPage } from "@/pages/SettingsPage";
import type { AppRoute } from "@/lib/navigation";

export default function App() {
  const [route, setRoute] = useState<AppRoute>({ page: "home" });

  if (route.page === "project") {
    return <ProjectDetailPage projectId={route.projectId} onBack={() => setRoute({ page: route.from })} />;
  }

  return (
    <AppShell route={route} onNavigate={setRoute}>
      {route.page === "home" && <HomePage onNavigate={setRoute} />}
      {route.page === "library" && <LibraryPage onNavigate={setRoute} />}
      {route.page === "settings" && <SettingsPage />}
    </AppShell>
  );
}
