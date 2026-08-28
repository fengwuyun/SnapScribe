import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { AppShell } from "@/components/AppShell";
import { SettingsPageHeader } from "@/pages/SettingsPage";

describe("settings page header", () => {
  it("keeps the title, description, and section navigation in normal document flow", () => {
    const markup = renderToStaticMarkup(
      <SettingsPageHeader activeSection="storage" onNavigate={() => undefined} />,
    );

    expect(markup).toMatch(
      /<header[^>]*>[\s\S]*<h1[^>]*>设置<\/h1>[\s\S]*管理本地文件存储与应用信息[\s\S]*<nav[^>]*aria-label="设置页面导航"/,
    );
    expect(markup).not.toContain("sticky");
    expect(markup).not.toContain("top-0");
  });

  it("keeps the existing page margin without pinning the settings header", () => {
    const markup = renderToStaticMarkup(
      <AppShell route={{ page: "settings" }} onNavigate={() => undefined}>
        <SettingsPageHeader activeSection="storage" onNavigate={() => undefined} />
      </AppShell>,
    );

    expect(markup).toMatch(/<main[^>]*class="[^"]*py-0[^"]*"/);
    expect(markup).toMatch(/<header[^>]*class="[^"]*pt-8[^"]*"/);
    expect(markup).not.toContain("sticky");
  });

  it("always renders the settings title", () => {
    const markup = renderToStaticMarkup(<SettingsPageHeader activeSection="about" onNavigate={() => undefined} />);
    expect(markup).toContain("设置");
    expect(markup).not.toContain('aria-hidden="true"');
    expect(markup).not.toContain("grid-rows-[0fr]");
  });
});
