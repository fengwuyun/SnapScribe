import { forwardRef } from "react";
import { cn } from "@/lib/utils";

/** Text input per UI_DESIGN_SPEC: r12, 1px border, 14px text. */
export const Input = forwardRef<HTMLInputElement, React.InputHTMLAttributes<HTMLInputElement>>(
  ({ className, ...props }, ref) => (
    <input
      ref={ref}
      className={cn(
        "h-9 w-full rounded-md border border-line bg-surface px-3 text-sm text-text-primary outline-none transition-colors duration-150 placeholder:text-text-tertiary focus:border-primary",
        className,
      )}
      {...props}
    />
  ),
);
Input.displayName = "Input";
