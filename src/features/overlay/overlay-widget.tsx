import { createEffect, createMemo, createSignal, For, onCleanup, onMount, Show } from "solid-js";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";

import { CircleDiagram } from "~/features/overlay/circle-diagram";
import { formatReset, getMonogram, getShortLabel } from "~/features/overlay/overlay-helpers";
import { minRemainingPercent, remainingPercent } from "~/features/usage/usage-metric";
import { cn } from "~/lib/utils";
import {
  getSettings,
  getUsageSnapshot,
  hideOverlayWindow,
  onUsageUpdated,
  saveOverlayPosition,
  showSettingsWindow,
  toggleUsageWindow,
} from "~/services/usage";
import type { AppSettings, UsageSnapshot } from "~/types/usage";

type DisplayCircleItem = {
  readonly id: string;
  readonly label: string;
  readonly shortLabel: string;
  readonly monogram: string;
  readonly remaining: number;
  readonly tooltip: string;
};

export function OverlayWidget() {
  const [snapshot, setSnapshot] = createSignal<UsageSnapshot | null>(null);
  const [settings, setSettings] = createSignal<AppSettings | null>(null);
  const [contextMenuOpen, setContextMenuOpen] = createSignal(false);
  const [menuPos, setMenuPos] = createSignal<{ x: number; y: number }>({ x: 0, y: 0 });

  let pillRef: HTMLDivElement | undefined = undefined;
  let saveTimer: ReturnType<typeof setTimeout> | undefined;

  const handlePositionChanged = (x: number, y: number) => {
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => {
      void saveOverlayPosition(x, y);
    }, 600);
  };

  const syncWindowSize = () => {
    const el = pillRef as HTMLElement | undefined;
    if (!el) return;
    const rect = el.getBoundingClientRect();
    const width = Math.max(64, Math.ceil(rect.width));
    const height = Math.max(28, Math.ceil(rect.height));

    void getCurrentWindow()
      .setSize(new LogicalSize(width, height))
      .catch(() => undefined);
  };

  onMount(() => {
    let unlistenUsage: (() => void) | undefined;
    let unlistenMoved: (() => void) | undefined;

    void getUsageSnapshot().match(
      (data) => setSnapshot(data),
      () => undefined,
    );

    void getSettings().match(
      (data) => setSettings(data),
      () => undefined,
    );

    void onUsageUpdated((data) => setSnapshot(data)).match(
      (fn) => {
        unlistenUsage = fn;
      },
      () => undefined,
    );

    void getCurrentWindow()
      .onMoved(({ payload: pos }) => {
        handlePositionChanged(pos.x, pos.y);
      })
      .then((fn) => {
        unlistenMoved = fn;
      })
      .catch(() => undefined);

    const handleWindowClick = () => setContextMenuOpen(false);
    window.addEventListener("click", handleWindowClick);

    requestAnimationFrame(() => syncWindowSize());

    onCleanup(() => {
      if (saveTimer) clearTimeout(saveTimer);
      unlistenUsage?.();
      unlistenMoved?.();
      window.removeEventListener("click", handleWindowClick);
    });
  });

  const circleItems = createMemo<DisplayCircleItem[]>(() => {
    const data = snapshot();
    const config = settings();
    if (!data || data.providers.length === 0) return [];

    const selectedKeys = new Set(config?.overlay_items ?? []);
    const items: DisplayCircleItem[] = [];

    if (selectedKeys.size > 0) {
      for (const provider of data.providers) {
        if (provider.windows.length === 0) continue;

        const matchingWindows = provider.windows.filter((win) =>
          selectedKeys.has(`${provider.provider_id}:${win.label}`),
        );
        const count = matchingWindows.length;

        // Check each specific window
        for (const win of matchingWindows) {
          const key = `${provider.provider_id}:${win.label}`;
          const remaining = remainingPercent(win);
          const reset = formatReset(win.resets_at);
          const tooltip = `${provider.display_name} (${win.label}): ${remaining.toFixed(0)}% remaining${
            reset ? ` • ${reset}` : ""
          }`;
          items.push({
            id: key,
            label: `${provider.display_name} - ${win.label}`,
            shortLabel: getShortLabel(provider.provider_id, win.label, count),
            monogram: getMonogram(provider.provider_id),
            remaining,
            tooltip,
          });
        }

        // Only add provider-level summary if NO specific window of this provider was selected
        if (count === 0 && selectedKeys.has(provider.provider_id)) {
          const remaining = minRemainingPercent(provider.windows) ?? 0;
          items.push({
            id: provider.provider_id,
            label: provider.display_name,
            shortLabel: getShortLabel(provider.provider_id, undefined, 1),
            monogram: getMonogram(provider.provider_id),
            remaining,
            tooltip: `${provider.display_name}: ${remaining.toFixed(0)}% remaining`,
          });
        }
      }
    } else {
      // Default: 1 circle per active provider (showing lowest remaining window)
      for (const provider of data.providers) {
        if (provider.windows.length === 0) continue;
        const remaining = minRemainingPercent(provider.windows) ?? 0;
        const lowestWin = [...provider.windows].sort(
          (a, b) => remainingPercent(a) - remainingPercent(b),
        )[0];
        const reset = lowestWin ? formatReset(lowestWin.resets_at) : "";
        items.push({
          id: provider.provider_id,
          label: provider.display_name,
          shortLabel: getShortLabel(provider.provider_id),
          monogram: getMonogram(provider.provider_id),
          remaining,
          tooltip: `${provider.display_name}: ${remaining.toFixed(0)}% remaining${
            reset ? ` • ${reset}` : ""
          }`,
        });
      }
    }

    return items;
  });

  const showLabels = () => settings()?.overlay_show_labels ?? true;
  const labelPos = () => settings()?.overlay_label_position ?? "beside";

  // Dynamically sync window size whenever circles or label settings change
  createEffect(() => {
    circleItems();
    showLabels();
    labelPos();
    requestAnimationFrame(() => syncWindowSize());
  });

  const handleContextMenu = (e: MouseEvent) => {
    e.preventDefault();
    setMenuPos({ x: e.clientX, y: e.clientY });
    setContextMenuOpen(true);
  };

  const handleOpenUsage = () => {
    setContextMenuOpen(false);
    void toggleUsageWindow();
  };

  const handleOpenSettings = () => {
    setContextMenuOpen(false);
    void showSettingsWindow();
  };

  const handleHideWidget = () => {
    setContextMenuOpen(false);
    void hideOverlayWindow();
  };

  return (
    <div
      class="inline-flex m-0 p-0 select-none items-center justify-start overflow-hidden bg-transparent"
      onContextMenu={handleContextMenu}
    >
      {/* Floating Pill Container - Borderless */}
      <div
        ref={pillRef}
        class={cn(
          "group inline-flex items-center rounded-full bg-bg/95 shadow-xl backdrop-blur-md px-2.5 transition-colors",
          labelPos() === "below" && showLabels() ? "h-11 gap-2 py-1" : "h-8 gap-2.5",
        )}
        data-tauri-drag-region
      >
        {/* Drag handle dots */}
        <div
          class="flex cursor-grab flex-col gap-0.5 pr-0.5 text-muted/40 active:cursor-grabbing group-hover:text-muted/70"
          data-tauri-drag-region
          title="Drag to reposition (Double-click to open details)"
          onDblClick={handleOpenUsage}
        >
          <div class="size-1 rounded-full bg-current" />
          <div class="size-1 rounded-full bg-current" />
          <div class="size-1 rounded-full bg-current" />
        </div>

        {/* Circles */}
        <Show
          when={circleItems().length > 0}
          fallback={
            <div
              class="flex cursor-pointer items-center px-1 text-xs text-muted"
              onClick={handleOpenUsage}
            >
              No data
            </div>
          }
        >
          <div class="flex items-center gap-2" onDblClick={handleOpenUsage}>
            <For each={circleItems()}>
              {(item) => (
                <div
                  class={cn(
                    "flex items-center",
                    labelPos() === "below" && showLabels()
                      ? "flex-col gap-0.5 px-0.5"
                      : "gap-1.5 px-0.5",
                  )}
                >
                  <CircleDiagram
                    id={item.id}
                    label={item.label}
                    monogram={item.monogram}
                    remaining={item.remaining}
                    tooltip={item.tooltip}
                    showPercentage={settings()?.overlay_show_percentage ?? true}
                    size={24}
                  />
                  <Show when={showLabels()}>
                    <span
                      class={cn(
                        "font-medium tracking-tight select-none",
                        labelPos() === "below"
                          ? "text-[9px] text-muted/90 max-w-[56px] truncate text-center leading-none"
                          : "text-[11px] text-fg/90 whitespace-nowrap leading-none",
                      )}
                    >
                      {item.shortLabel}
                    </span>
                  </Show>
                </div>
              )}
            </For>
          </div>
        </Show>
      </div>

      {/* Context Menu */}
      <Show when={contextMenuOpen()}>
        <div
          class="fixed z-50 min-w-32 rounded-lg bg-surface/95 py-1 text-xs shadow-2xl backdrop-blur-lg"
          style={{
            top: `${Math.min(window.innerHeight - 80, menuPos().y)}px`,
            left: `${Math.min(window.innerWidth - 130, menuPos().x)}px`,
          }}
        >
          <button
            type="button"
            class="flex w-full items-center px-3 py-1.5 text-left text-fg hover:bg-white/10"
            onClick={handleOpenUsage}
          >
            Open AI Usage
          </button>
          <button
            type="button"
            class="flex w-full items-center px-3 py-1.5 text-left text-fg hover:bg-white/10"
            onClick={handleOpenSettings}
          >
            Settings
          </button>
          <div class="my-1 border-t border-border/40" />
          <button
            type="button"
            class="flex w-full items-center px-3 py-1.5 text-left text-critical hover:bg-critical/20"
            onClick={handleHideWidget}
          >
            Hide Widget
          </button>
        </div>
      </Show>
    </div>
  );
}
