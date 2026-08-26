import * as DialogPrimitive from "@radix-ui/react-dialog";
import { X } from "lucide-react";
import { cn } from "@/lib/utils";

export const Dialog = DialogPrimitive.Root;
export const DialogTrigger = DialogPrimitive.Trigger;
export const DialogClose = DialogPrimitive.Close;

/**
 * Modal per UI_DESIGN_SPEC §15: white, r18, padding 24, shadow-md,
 * right-aligned footer buttons, close icon top-right.
 */
export function DialogContent({
  className,
  children,
  title,
  onCloseLabel,
}: {
  className?: string;
  children: React.ReactNode;
  title: string;
  onCloseLabel: string;
}) {
  return (
    <DialogPrimitive.Portal>
      <DialogPrimitive.Overlay className="fixed inset-0 z-40 bg-black/25 transition-opacity duration-150" />
      <DialogPrimitive.Content
        className={cn(
          "fixed left-1/2 top-1/2 z-50 w-[480px] max-w-[calc(100vw-48px)] -translate-x-1/2 -translate-y-1/2 rounded-[18px] bg-surface p-6 shadow-md outline-none animate-fade-in",
          className,
        )}
      >
        <div className="mb-4 flex items-start justify-between">
          <DialogPrimitive.Title className="text-lg font-semibold text-text-primary">
            {title}
          </DialogPrimitive.Title>
          <DialogPrimitive.Close
            aria-label={onCloseLabel}
            className="rounded-sm p-1 text-text-tertiary transition-colors duration-150 hover:bg-divider hover:text-text-secondary"
          >
            <X className="size-4" />
          </DialogPrimitive.Close>
        </div>
        {children}
      </DialogPrimitive.Content>
    </DialogPrimitive.Portal>
  );
}
