import { createMemo } from "solid-js";

import { circleDashOffset, remainingStrokeClass } from "~/features/usage/usage-metric";
import { cn } from "~/lib/utils";

const CIRCUMFERENCE = 78.5398; // 2 * Math.PI * 12.5

export type CircleDiagramProps = {
  readonly id: string;
  readonly label: string;
  readonly monogram: string;
  readonly remaining: number;
  readonly tooltip: string;
  readonly showPercentage: boolean;
  readonly size?: number;
};

export function CircleDiagram(props: CircleDiagramProps) {
  const size = () => props.size ?? 30;

  const offset = createMemo(() => circleDashOffset(props.remaining, CIRCUMFERENCE));

  const strokeClass = createMemo(() => remainingStrokeClass(props.remaining));

  const centerText = createMemo(() => {
    if (props.showPercentage) {
      const rounded = Math.round(props.remaining);
      return rounded >= 100 ? "99" : `${rounded}`;
    }
    return props.monogram;
  });

  const textFontSize = createMemo(() => {
    const text = centerText();
    if (text.length >= 3) return "text-[8px]";
    if (text.length === 2) return "text-[9px]";
    return "text-[10px]";
  });

  return (
    <div
      class="group relative flex shrink-0 cursor-pointer items-center justify-center transition-transform hover:scale-105 active:scale-95"
      title={props.tooltip}
    >
      <svg
        width={size()}
        height={size()}
        viewBox="0 0 32 32"
        class="overflow-visible"
        aria-label={props.tooltip}
      >
        {/* Track */}
        <circle
          cx="16"
          cy="16"
          r="12.5"
          fill="none"
          stroke="currentColor"
          stroke-width="2.6"
          class="text-white/10"
        />

        {/* Progress Arc */}
        <circle
          cx="16"
          cy="16"
          r="12.5"
          fill="none"
          stroke="currentColor"
          stroke-width="2.6"
          stroke-dasharray={CIRCUMFERENCE.toFixed(2)}
          stroke-dashoffset={offset().toFixed(2)}
          stroke-linecap="round"
          transform="rotate(-90 16 16)"
          class={cn("transition-[stroke-dashoffset] duration-500 ease-out", strokeClass())}
        />

        {/* Center Label */}
        <text
          x="16"
          y="16.5"
          text-anchor="middle"
          dominant-baseline="central"
          class={cn(
            "pointer-events-none select-none font-bold tracking-tight fill-fg transition-colors",
            textFontSize(),
          )}
        >
          {centerText()}
        </text>
      </svg>
    </div>
  );
}
