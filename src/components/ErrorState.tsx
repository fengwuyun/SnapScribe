import { TriangleAlert } from "lucide-react";
import { Button } from "@/components/ui/Button";

/** Error state per UI_DESIGN_SPEC §14: red accents only, light background stays neutral. */
export function ErrorState({
  message,
  onRetry,
}: {
  message: string;
  onRetry: () => void;
}) {
  return (
    <div className="flex flex-1 flex-col items-center justify-center gap-3 rounded-lg bg-surface px-8 py-12">
      <TriangleAlert className="size-8 text-error" aria-hidden />
      <p className="text-sm font-semibold text-text-primary">无法读取该文件</p>
      <p className="max-w-md text-center text-xs leading-relaxed text-text-secondary">
        {message || "文件可能损坏或格式不受支持。"}
      </p>
      <Button variant="secondary" onClick={onRetry} className="mt-2">
        重新选择
      </Button>
    </div>
  );
}
