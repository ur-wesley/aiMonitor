import { createMemo, For, Show } from "solid-js";

import { minRemainingPercent } from "~/features/usage/usage-metric";
import { UsageBar } from "~/features/usage/usage-bar";
import type { ProviderUsage } from "~/types/usage";

type ProviderCardProps = {
  readonly provider: ProviderUsage;
};

const providerHints: Record<string, string> = {
  cursor: "Sign in to Cursor on this PC",
  opencode_go: "Run /connect in OpenCode or add an API key in Settings",
  antigravity: "Sign in to Antigravity CLI or IDE",
};

export function ProviderCard(props: ProviderCardProps) {
  const hint = providerHints[props.provider.provider_id] ?? "Check provider setup";

  const remaining = createMemo(() => {
    if (props.provider.error) return null;
    return minRemainingPercent(props.provider.windows);
  });

  return (
    <article class="mb-2.5 min-w-0 rounded-lg border border-border bg-surface p-3">
      <div class="flex items-center justify-between gap-2">
        <h3 class="font-semibold">{props.provider.display_name}</h3>
        <Show when={remaining() !== null}>
          <span class="shrink-0 text-sm text-muted">{remaining()!.toFixed(0)}% left</span>
        </Show>
      </div>
      {props.provider.error ? (
        <div class="mt-2 space-y-1">
          <p class="break-words text-sm text-critical">{props.provider.error}</p>
          <p class="break-words text-xs text-muted">{hint}</p>
        </div>
      ) : (
        <div class="mt-2">
          <For each={[...props.provider.windows]}>
            {(window) => <UsageBar window={window} />}
          </For>
        </div>
      )}
    </article>
  );
}
