import {
  createEffect,
  createMemo,
  createSignal,
  For,
  onCleanup,
  onMount,
  Show,
} from "solid-js";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";

import { Button } from "~/components/ui/button";
import { ProviderCard } from "~/features/usage/provider-card";
import {
  getUsageSnapshot,
  onUsageUpdated,
  placeUsageWindow,
  quitApp,
  refreshUsage,
  showSettingsWindow,
} from "~/services/usage";
import type { UsageSnapshot } from "~/types/usage";

const POPUP_WIDTH = 340;
const OUTER_PADDING = 8;
const CARD_PADDING = 16;
const MIN_POPUP_HEIGHT = 220;

export function UsagePopup() {
  const [snapshot, setSnapshot] = createSignal<UsageSnapshot | null>(null);
  const [isRefreshing, setIsRefreshing] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  let headerRef: HTMLElement | undefined;
  let providersRef: HTMLElement | undefined;
  let footerRef: HTMLElement | undefined;

  const lastUpdated = createMemo(() => {
    const value = snapshot()?.fetched_at;
    if (!value) return "—";
    return new Date(value).toLocaleString();
  });

  const providers = createMemo(() =>
    [...(snapshot()?.providers ?? [])].sort((a, b) =>
      a.provider_id.localeCompare(b.provider_id),
    ),
  );

  const syncWindowSize = () => {
    const headerH = headerRef?.offsetHeight ?? 0;
    const providersH = providersRef?.scrollHeight ?? 0;
    const footerH = footerRef?.offsetHeight ?? 0;
    const contentHeight = headerH + providersH + footerH + CARD_PADDING + OUTER_PADDING * 2;
    const height = Math.max(MIN_POPUP_HEIGHT, contentHeight);
    const width = POPUP_WIDTH + OUTER_PADDING * 2;

    void getCurrentWindow()
      .setSize(new LogicalSize(width, height))
      .then(() => placeUsageWindow())
      .catch(() => undefined);
  };

  createEffect(() => {
    providers();
    error();
    requestAnimationFrame(() => syncWindowSize());
  });

  onMount(() => {
    let dispose: (() => void) | undefined;
    onCleanup(() => dispose?.());

    requestAnimationFrame(() => syncWindowSize());

    void getUsageSnapshot().match(
      (data) => setSnapshot(data),
      (err) => setError(err.message),
    );

    void onUsageUpdated((data) => setSnapshot(data)).match(
      (unlisten) => {
        dispose = unlisten;
      },
      (err) => setError(err.message),
    );
  });

  const handleRefresh = () => {
    if (isRefreshing()) return;
    setIsRefreshing(true);
    void refreshUsage().match(
      (data) => {
        setSnapshot(data);
        setError(null);
        setIsRefreshing(false);
      },
      (err) => {
        setError(err.message);
        setIsRefreshing(false);
      },
    );
  };

  return (
    <div class="box-border flex h-screen w-full flex-col p-2">
      <div class="flex min-h-0 flex-1 flex-col overflow-hidden rounded-[10px] border border-border bg-bg/70 shadow-xl backdrop-blur-xl">
        <div class="flex min-h-0 flex-1 flex-col p-2">
          <header ref={headerRef} class="mb-2 flex shrink-0 min-w-0 items-center gap-2">
            <h1 class="min-w-0 flex-1 truncate text-lg font-semibold">AI Usage</h1>
            <span class="shrink-0 text-xs text-muted">{lastUpdated()}</span>
            <Button
              variant="ghost"
              size="icon"
              disabled={isRefreshing()}
              onClick={handleRefresh}
              aria-label="Refresh usage"
            >
              <span
                class={`i-[mdi--refresh] size-4 ${isRefreshing() ? "animate-spin" : ""}`}
                aria-hidden="true"
              />
            </Button>
          </header>

          <Show when={error()}>
            <p class="mb-2 shrink-0 text-sm text-critical">{error()}</p>
          </Show>

          <section
            ref={providersRef}
            class="app-scroll app-scroll-y min-h-0 flex-1 overflow-y-auto"
          >
            <For each={providers()} fallback={<p class="text-sm text-muted">Loading…</p>}>
              {(provider) => <ProviderCard provider={provider} />}
            </For>
          </section>

          <footer ref={footerRef} class="mt-2 flex shrink-0 justify-end gap-2">
            <Button variant="outline" size="sm" onClick={() => void showSettingsWindow()}>
              Settings
            </Button>
            <Button variant="outline" size="sm" onClick={() => void quitApp()}>
              Quit
            </Button>
          </footer>
        </div>
      </div>
    </div>
  );
}
