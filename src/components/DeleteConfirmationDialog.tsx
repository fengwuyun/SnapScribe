import { useEffect, useState } from "react";
import * as Dialog from "@radix-ui/react-dialog";
import { AlertTriangle, X } from "lucide-react";
import { Button } from "@/components/ui/Button";

export function DeleteConfirmationDialog({
  open,
  title,
  description,
  finalDescription,
  busy = false,
  onOpenChange,
  onConfirm,
}: {
  open: boolean;
  title: string;
  description: string;
  finalDescription: string;
  busy?: boolean;
  onOpenChange: (open: boolean) => void;
  onConfirm: () => void;
}) {
  const [step, setStep] = useState<1 | 2>(1);

  useEffect(() => {
    if (!open) setStep(1);
  }, [open]);

  return (
    <Dialog.Root open={open} onOpenChange={(next) => !busy && onOpenChange(next)}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 z-40 bg-black/25 backdrop-blur-[2px] data-[state=closed]:animate-fade-out data-[state=open]:animate-fade-in" />
        <Dialog.Content className="fixed left-1/2 top-1/2 z-50 w-[420px] max-w-[calc(100vw-32px)] -translate-x-1/2 -translate-y-1/2 rounded-xl border border-line bg-surface p-6 shadow-md outline-none">
          <div className="flex items-start gap-4">
            <span className="flex size-10 shrink-0 items-center justify-center rounded-round bg-[#FFF1F1] text-error">
              <AlertTriangle className="size-5" />
            </span>
            <div className="min-w-0 flex-1">
              <Dialog.Title className="text-base font-semibold">{step === 1 ? title : "请再次确认"}</Dialog.Title>
              <Dialog.Description className="mt-2 text-sm leading-6 text-text-secondary">
                {step === 1 ? description : finalDescription}
              </Dialog.Description>
            </div>
            <Dialog.Close asChild>
              <button type="button" aria-label="关闭" className="flex size-8 items-center justify-center rounded-md text-text-tertiary hover:bg-bg hover:text-text-primary">
                <X className="size-4" />
              </button>
            </Dialog.Close>
          </div>
          <div className="mt-6 flex justify-end gap-3">
            <Button variant="secondary" disabled={busy} onClick={() => onOpenChange(false)}>取消</Button>
            {step === 1 ? (
              <Button variant="secondary" className="border-error/30 text-error hover:bg-[#FFF1F1]" onClick={() => setStep(2)}>继续删除</Button>
            ) : (
              <Button variant="danger" loading={busy} onClick={onConfirm}>确认删除</Button>
            )}
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
