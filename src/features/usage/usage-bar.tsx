import { createMemo } from "solid-js";

import { remainingPercent, remainingStatusClass } from "~/features/usage/usage-metric";
import { cn } from "~/lib/utils";
import type { UsageWindowMetric } from "~/types/usage";

type UsageBarProps = {
  readonly window: UsageWindowMetric;
};

function formatReset(resetsAt: string | null): string {
  if (!resetsAt) return "";
  const reset = new Date(resetsAt);
  const delta = reset.getTime() - Date.now();
  if (delta <= 0) return "resets soon";
  const hours = Math.floor(delta / 3_600_000);
  const minutes = Math.floor((delta % 3_600_000) / 60_000);
  if (hours >= 24) return `resets in ${Math.floor(hours / 24)}d ${hours % 24}h`;
  if (hours >= 1) return `resets in ${hours}h ${minutes}m`;
  return `resets in ${minutes}m`;
}

export function UsageBar(props: UsageBarProps) {
  const remaining = createMemo(() => remainingPercent(props.window));

  const displayText = createMemo(() => `${remaining().toFixed(0)}% left`);

  return (
    <div class="mb-1.5 space-y-1">
      <div class="flex items-center justify-between text-sm">
        <span>{props.window.label}</span>
        <span>{displayText()}</span>
      </div>
      <div class="h-2 overflow-hidden rounded-full bg-border/60">
        <div
          class={cn("h-full rounded-full transition-all", remainingStatusClass(remaining()))}
          style={{ width: `${remaining()}%` }}
        />
      </div>
      <p class="text-[11px] text-muted">{formatReset(props.window.resets_at)}</p>
    </div>
  );
}
