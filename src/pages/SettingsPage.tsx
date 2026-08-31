import { useEffect, useRef, useState } from "react";
import { ExternalLink, Github } from "lucide-react";
import { Button } from "@/components/ui/Button";
import * as ipc from "@/lib/tauri";
import { settingsSections, type SettingsSectionId } from "@/lib/settingsNavigation";
import type { AboutInfo, AppSettings } from "@/types/project";

export function SettingsPageHeader({ activeSection, onNavigate }: { activeSection: SettingsSectionId; onNavigate: (id: SettingsSectionId) => void }) {
  return <header className="-mx-2 bg-bg px-2 pt-8">
    <div>
      <h1 className="text-2xl font-bold">设置</h1>
      <p className="mt-1 text-sm text-text-secondary">管理本地文件存储与应用信息</p>
    </div>
    <nav aria-label="设置页面导航" className="mt-5 flex gap-8 border-b border-divider">
      {settingsSections.map((item) => <button key={item.id} type="button" aria-current={activeSection === item.id ? "page" : undefined} onClick={() => onNavigate(item.id)} className={`border-b-2 px-1 pb-3 text-sm transition-colors duration-150 ${activeSection === item.id ? "border-primary font-semibold text-primary" : "border-transparent text-text-secondary hover:text-text-primary"}`}>{item.label}</button>)}
    </nav>
  </header>;
}

export function SettingsPage() {
  const [draft, setDraft] = useState<AppSettings | null>(null);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState("");
  const [about, setAbout] = useState<AboutInfo | null>(null);
  const [activeSection, setActiveSection] = useState<SettingsSectionId>("storage");
  const sectionRefs = useRef<Record<SettingsSectionId, HTMLElement | null>>({ storage: null, about: null });

  useEffect(() => {
    void Promise.all([ipc.settingsGet(), ipc.aboutGet()]).then(([settings, info]) => { setDraft(settings); setAbout(info); }).catch((error) => setMessage(String(error)));
  }, []);

  useEffect(() => {
    if (!draft) return;
    const observer = new IntersectionObserver((entries) => {
      const visible = entries.filter((entry) => entry.isIntersecting).sort((a, b) => b.intersectionRatio - a.intersectionRatio)[0];
      if (visible?.target.id) setActiveSection(visible.target.id as SettingsSectionId);
    }, { rootMargin: "-64px 0px -52% 0px", threshold: [0.05, 0.25, 0.5] });
    settingsSections.forEach(({ id }) => {
      const section = sectionRefs.current[id];
      if (section) observer.observe(section);
    });
    return () => observer.disconnect();
  }, [draft, about]);

  if (!draft) return <p className={message ? "text-sm text-error" : "text-sm text-text-secondary"}>{message || "正在加载设置…"}</p>;

  async function save() {
    setSaving(true);
    setMessage("");
    try {
      const saved = await ipc.settingsSave(draft!);
      setDraft(saved);
      setMessage("设置已保存");
    } catch (error) {
      setMessage(error instanceof Error ? error.message : String(error));
    } finally {
      setSaving(false);
    }
  }

  async function chooseDataRoot() {
    const path = await ipc.selectDirectory();
    if (path) setDraft({ ...draft!, dataRoot: path });
  }

  async function openDataRoot() {
    try { await ipc.settingsOpenDataRoot(); }
    catch (error) { setMessage(String(error)); }
  }

  function navigateToSection(id: SettingsSectionId) {
    setActiveSection(id);
    sectionRefs.current[id]?.scrollIntoView({ behavior: "smooth", block: "start" });
  }

  return <div className="mx-auto max-w-[900px] pb-8">
    <SettingsPageHeader activeSection={activeSection} onNavigate={navigateToSection} />
    <div className="mt-6 grid gap-6">
      <section id="storage" ref={(node) => { sectionRefs.current.storage = node; }} className="scroll-mt-16 rounded-lg border border-line bg-surface p-6 shadow-sm">
        <h2 className="text-lg font-semibold">文件存储</h2>
        <label className="mt-5 block text-sm font-medium">转录数据保存位置</label>
        <div className="mt-2 flex gap-2"><input value={draft.dataRoot} onChange={(event) => setDraft({ ...draft, dataRoot: event.target.value })} className="h-10 min-w-0 flex-1 rounded-md border border-line px-3 text-sm outline-none focus:border-primary" /><Button variant="secondary" onClick={() => void openDataRoot()}>打开当前文件夹</Button><Button variant="secondary" onClick={() => void chooseDataRoot()}>选择位置</Button></div>
        <p className="mt-2 text-xs text-text-tertiary">更改后点击保存，现有项目会复制到新位置，旧位置保留。</p>
        <label className="mt-5 block text-sm font-medium">导入音视频策略
          <select value={draft.importStrategy} onChange={(event) => setDraft({ ...draft, importStrategy: event.target.value as AppSettings["importStrategy"] })} className="mt-2 h-10 w-full rounded-md border border-line bg-surface px-3 text-sm">
            <option value="reference">仅引用原文件（默认 / 推荐）</option>
            <option value="copy">复制到 SnapScribe</option>
            <option value="move">移动到 SnapScribe（便于管理）</option>
          </select>
        </label>
        {draft.importStrategy === "move" && <p className="mt-2 text-xs text-warning">导入成功后原位置文件将被删除，SnapScribe 内只保留一份。</p>}
        <p className="mt-3 text-xs text-text-tertiary">软件录音默认保存，可在项目详情中单独删除，不影响文本和总结。</p>
      </section>
      {about && <section id="about" ref={(node) => { sectionRefs.current.about = node; }} className="scroll-mt-16 rounded-lg border border-line bg-surface p-6 shadow-sm">
        <h2 className="text-lg font-semibold">关于 SnapScribe</h2>
        <div className="mt-5 grid gap-4 sm:grid-cols-2">
          <div><p className="text-xs text-text-tertiary">当前版本</p><p className="mt-1 font-mono text-sm font-semibold">v{about.version}</p></div>
          <div><p className="text-xs text-text-tertiary">版本时间</p><p className="mt-1 font-mono text-sm font-semibold">{about.buildTime}</p></div>
        </div>
        <button type="button" onClick={() => void ipc.openRepository().catch((error) => setMessage(String(error)))} className="mt-5 flex w-full items-center gap-3 rounded-md border border-line px-4 py-3 text-left transition-colors duration-150 hover:border-primary hover:bg-primary-soft focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary">
          <span className="flex size-9 items-center justify-center rounded-md bg-[#111111] text-white"><Github className="size-5" /></span>
          <span className="min-w-0 flex-1"><span className="block text-sm font-semibold">GitHub 仓库</span><span className="mt-0.5 block truncate text-xs text-text-tertiary">{about.repositoryUrl}</span></span>
          <ExternalLink className="size-4 text-text-tertiary" />
        </button>
      </section>}
    </div>
    <div className="mt-6 flex items-center justify-end gap-4">{message && <span className="text-sm text-text-secondary">{message}</span>}<Button loading={saving} onClick={() => void save()}>保存设置</Button></div>
  </div>;
}
