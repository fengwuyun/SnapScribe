import { useEffect, useState } from "react";
import { Users } from "lucide-react";
import { Button } from "@/components/ui/Button";
import { Dialog, DialogContent } from "@/components/ui/Dialog";
import { speakerBadgeClass, validateSpeakerDrafts } from "@/lib/speakers";
import type { TranscriptSpeaker } from "@/types/transcript";

export function SpeakerManagerDialog({
  open,
  speakers,
  busy,
  onOpenChange,
  onSave,
}: {
  open: boolean;
  speakers: TranscriptSpeaker[];
  busy: boolean;
  onOpenChange: (open: boolean) => void;
  onSave: (speakers: TranscriptSpeaker[], warning?: string) => void;
}) {
  const [drafts, setDrafts] = useState<TranscriptSpeaker[]>(speakers);
  const [error, setError] = useState("");

  useEffect(() => {
    if (open) {
      setDrafts(speakers.map((speaker) => ({ ...speaker })));
      setError("");
    }
  }, [open, speakers]);

  function save() {
    const normalized = drafts.map((speaker) => ({ ...speaker, name: speaker.name.trim() }));
    const validation = validateSpeakerDrafts(normalized);
    if (!validation.ok) {
      setError(validation.message);
      return;
    }
    onSave(normalized, validation.warning);
  }

  return <Dialog open={open} onOpenChange={(next) => !busy && onOpenChange(next)}>
    <DialogContent title="管理说话人" onCloseLabel="关闭说话人管理" className="w-[520px]">
      <p className="mb-5 text-sm leading-6 text-text-secondary">修改名称会批量应用到该说话人的全部转录段落，不改变原始识别结果。</p>
      <div className="max-h-[360px] space-y-3 overflow-y-auto pr-1">
        {drafts.map((speaker, index) => <label key={speaker.id} className="flex items-center gap-3 rounded-lg border border-line bg-bg px-3 py-3">
          <span className={`inline-flex h-7 min-w-20 items-center justify-center rounded-round px-2.5 text-xs font-semibold ${speakerBadgeClass(speaker.colorIndex)}`}>
            说话人 {index + 1}
          </span>
          <input
            value={speaker.name}
            onChange={(event) => {
              setError("");
              setDrafts((items) => items.map((item) => item.id === speaker.id ? { ...item, name: event.target.value } : item));
            }}
            className="h-9 min-w-0 flex-1 rounded-md border border-line bg-surface px-3 text-sm outline-none transition-colors focus:border-primary focus:ring-2 focus:ring-primary/10"
            aria-label={`说话人 ${index + 1} 名称`}
          />
        </label>)}
        {drafts.length === 0 && <div className="py-8 text-center text-sm text-text-tertiary"><Users className="mx-auto mb-2 size-5" />暂无说话人</div>}
      </div>
      <div className="mt-4 min-h-5">{error && <p className="text-sm text-error">{error}</p>}</div>
      <div className="mt-4 flex justify-end gap-3">
        <Button variant="secondary" disabled={busy} onClick={() => onOpenChange(false)}>取消</Button>
        <Button loading={busy} disabled={drafts.length === 0} onClick={save}>保存名称</Button>
      </div>
    </DialogContent>
  </Dialog>;
}
