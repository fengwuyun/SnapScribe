import { useEffect, useRef, useState } from "react";
import { Download, ExternalLink, FolderOpen, Pencil, Trash2 } from "lucide-react";
import type { ProjectListItem } from "@/types/project";

interface ProjectContextMenuProps {
  project: ProjectListItem;
  x: number;
  y: number;
  onClose: () => void;
  onOpen: () => void;
  onRename: () => void;
  onExport: () => void;
  onOpenLocation: () => void;
  onDelete: () => void;
}

export function ProjectContextMenu({
  project,
  x,
  y,
  onClose,
  onOpen,
  onRename,
  onExport,
  onOpenLocation,
  onDelete,
}: ProjectContextMenuProps) {
  const menuRef = useRef<HTMLDivElement>(null);
  const [position, setPosition] = useState({ x, y });

  useEffect(() => {
    const menu = menuRef.current;
    if (!menu) return;
    const bounds = menu.getBoundingClientRect();
    setPosition({
      x: Math.max(8, Math.min(x, window.innerWidth - bounds.width - 8)),
      y: Math.max(8, Math.min(y, window.innerHeight - bounds.height - 8)),
    });
  }, [x, y]);

  useEffect(() => {
    function closeFromOutside(event: PointerEvent) {
      if (!menuRef.current?.contains(event.target as Node)) onClose();
    }
    function closeFromKeyboard(event: KeyboardEvent) {
      if (event.key === "Escape") onClose();
    }
    window.addEventListener("pointerdown", closeFromOutside);
    window.addEventListener("keydown", closeFromKeyboard);
    return () => {
      window.removeEventListener("pointerdown", closeFromOutside);
      window.removeEventListener("keydown", closeFromKeyboard);
    };
  }, [onClose]);

  const run = (action: () => void) => () => {
    onClose();
    action();
  };

  const itemClass = "flex w-full items-center gap-2.5 rounded-md px-3 py-2 text-left text-sm text-text-secondary transition-colors hover:bg-primary-soft hover:text-primary disabled:cursor-not-allowed disabled:opacity-40 disabled:hover:bg-transparent disabled:hover:text-text-secondary";

  return (
    <div
      ref={menuRef}
      role="menu"
      aria-label={`${project.name} 的操作`}
      className="fixed z-50 w-48 rounded-lg border border-line bg-surface p-1.5 shadow-md animate-fade-in"
      style={{ left: position.x, top: position.y }}
    >
      <button type="button" role="menuitem" className={itemClass} onClick={run(onOpen)}><FolderOpen className="size-4" />打开项目</button>
      <button type="button" role="menuitem" className={itemClass} onClick={run(onRename)}><Pencil className="size-4" />重命名</button>
      <button type="button" role="menuitem" className={itemClass} onClick={run(onExport)}><Download className="size-4" />导出 TXT</button>
      <button type="button" role="menuitem" className={itemClass} disabled={!project.mediaAvailable} onClick={run(onOpenLocation)}><ExternalLink className="size-4" />打开原文件位置</button>
      <div className="my-1 border-t border-divider" />
      <button type="button" role="menuitem" className={`${itemClass} text-error hover:bg-[#FFF1F1] hover:text-error`} onClick={run(onDelete)}><Trash2 className="size-4" />删除项目</button>
    </div>
  );
}
