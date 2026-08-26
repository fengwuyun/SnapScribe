import { forwardRef } from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { Loader2 } from "lucide-react";
import { cn } from "@/lib/utils";

const buttonVariants = cva(
  // Base per UI_DESIGN_SPEC §10: r12 · h40 · px20 · 14px/600 · stable width
  "inline-flex h-10 items-center justify-center gap-2 rounded-md px-5 text-sm font-semibold transition-colors duration-150 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/40 disabled:pointer-events-none disabled:opacity-45",
  {
    variants: {
      variant: {
        primary: "bg-primary text-white hover:bg-primary/90",
        secondary: "border border-line bg-surface text-text-primary hover:bg-bg",
        danger: "bg-accent text-white hover:bg-accent/90",
        ghost: "text-text-secondary hover:bg-divider",
      },
    },
    defaultVariants: {
      variant: "primary",
    },
  },
);

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {
  /** Shows a spinner instead of children while keeping the button width stable. */
  loading?: boolean;
}

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, loading = false, children, disabled, ...props }, ref) => (
    <button
      ref={ref}
      className={cn(buttonVariants({ variant }), className)}
      disabled={disabled || loading}
      {...props}
    >
      {loading ? (
        <>
          <Loader2 className="size-4 animate-spin" aria-hidden />
          <span>{children}</span>
        </>
      ) : (
        children
      )}
    </button>
  ),
);
Button.displayName = "Button";
