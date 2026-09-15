import { type JSX, splitProps } from "solid-js";
import { cva, type VariantProps } from "class-variance-authority";

import { cn } from "~/lib/utils";

const buttonVariants = cva(
  "inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors disabled:opacity-50 disabled:pointer-events-none",
  {
    variants: {
      variant: {
        default: "bg-accent text-white hover:bg-accent/90",
        outline: "border border-border bg-surface hover:bg-border/40",
        ghost: "hover:bg-border/40",
      },
      size: {
        default: "h-9 px-4 py-2",
        sm: "h-8 px-3",
        icon: "h-8 w-8",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  },
);

export type ButtonProps = VariantProps<typeof buttonVariants> & {
  readonly class?: string;
  readonly type?: "button" | "submit" | "reset";
  readonly disabled?: boolean;
  readonly onClick?: (event: MouseEvent) => void;
  readonly children?: JSX.Element;
};

export function Button(props: ButtonProps) {
  const [local, rest] = splitProps(props, ["class", "variant", "size"]);
  return (
    <button
      type={rest.type ?? "button"}
      class={cn(buttonVariants({ variant: local.variant, size: local.size }), local.class)}
      disabled={rest.disabled}
      onClick={rest.onClick}
    >
      {rest.children}
    </button>
  );
}
