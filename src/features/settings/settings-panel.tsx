import { createSignal, For, onMount, Show } from "solid-js";

import { WindowFrame } from "~/components/ui/window-frame";
import { Button } from "~/components/ui/button";
import {
  checkCredentials,
  getSettings,
  getUsageSnapshot,
  resetOverlayPosition,
  saveSettings,
} from "~/services/usage";
import type { AppSettings, CredentialHealth, UsageSnapshot } from "~/types/usage";

export function SettingsPanel() {
  const [settings, setSettings] = createSignal<AppSettings | null>(null);
  const [health, setHealth] = createSignal<CredentialHealth | null>(null);
  const [snapshot, setSnapshot] = createSignal<UsageSnapshot | null>(null);
  const [status, setStatus] = createSignal("");
  const [saving, setSaving] = createSignal(false);

  onMount(() => {
    void getSettings().match(
      (data) => setSettings(data),
      (err) => setStatus(err.message),
    );
    void checkCredentials().match(
      (data) => setHealth(data),
      () => undefined,
    );
    void getUsageSnapshot().match(
      (data) => setSnapshot(data),
      () => undefined,
    );
  });

  const update = <K extends keyof AppSettings>(key: K, value: AppSettings[K]) => {
    const current = settings();
    if (!current) return;
    setSettings({ ...current, [key]: value });
  };

  const toggleOverlayItem = (key: string, providerId?: string) => {
    const current = settings();
    if (!current) return;
    const items = new Set(current.overlay_items);
    if (providerId) {
      items.delete(providerId);
    }
    if (items.has(key)) {
      items.delete(key);
    } else {
      items.add(key);
    }
    update("overlay_items", Array.from(items));
  };

  const handleResetPosition = () => {
    void resetOverlayPosition().match(
      () => setStatus("Widget position reset to top-right."),
      (err) => setStatus(err.message),
    );
  };

  const handleSave = () => {
    const current = settings();
    if (!current) return;
    setSaving(true);
    void saveSettings({
      ...current,
      refresh_interval_seconds: Math.max(60, current.refresh_interval_seconds),
    }).match(
      (message) => {
        setStatus(message);
        setSaving(false);
        void checkCredentials().match(
          (data) => setHealth(data),
          () => undefined,
        );
      },
      (err) => {
        setStatus(err.message);
        setSaving(false);
      },
    );
  };

  return (
    <WindowFrame
      title="aiMonitor Settings"
      footer={
        <div class="flex items-center justify-between gap-3">
          <p class="min-w-0 truncate text-sm text-muted">{status()}</p>
          <Button disabled={saving()} onClick={handleSave}>
            Save
          </Button>
        </div>
      }
    >
      <Show when={settings()} fallback={<p class="text-muted">Loading…</p>}>
        {(current) => (
          <div class="space-y-6">
            <section class="space-y-3 rounded-lg border border-border bg-surface/80 p-3">
              <h2 class="font-medium">General</h2>
              <label class="block space-y-1 text-sm">
                <span>Refresh interval (seconds)</span>
                <input
                  type="number"
                  min={60}
                  max={3600}
                  class="w-full rounded-md border border-border bg-bg/80 px-3 py-2"
                  value={current().refresh_interval_seconds}
                  onInput={(e) => update("refresh_interval_seconds", Number(e.currentTarget.value))}
                />
              </label>
              <label class="flex items-center gap-2 text-sm">
                <input
                  type="checkbox"
                  checked={current().start_with_windows}
                  onChange={(e) => update("start_with_windows", e.currentTarget.checked)}
                />
                Start with Windows
              </label>
              <label class="flex items-center gap-2 text-sm">
                <input
                  type="checkbox"
                  checked={current().low_usage_notifications_enabled}
                  onChange={(e) =>
                    update("low_usage_notifications_enabled", e.currentTarget.checked)
                  }
                />
                Notify when a window is low (~10%)
              </label>
            </section>

            <section class="space-y-3 rounded-lg border border-border bg-surface/80 p-3">
              <div class="flex items-center justify-between">
                <h2 class="font-medium">Mini Widget (Always on Top)</h2>
                <Button
                  variant="outline"
                  size="sm"
                  class="h-7 text-xs"
                  onClick={handleResetPosition}
                >
                  Reset Position
                </Button>
              </div>

              <label class="flex items-center gap-2 text-sm">
                <input
                  type="checkbox"
                  checked={current().overlay_enabled}
                  onChange={(e) => update("overlay_enabled", e.currentTarget.checked)}
                />
                Enable Mini Floating Widget
              </label>

              <label class="flex items-center gap-2 text-sm">
                <input
                  type="checkbox"
                  checked={current().overlay_always_on_top}
                  onChange={(e) => update("overlay_always_on_top", e.currentTarget.checked)}
                />
                Keep Always on Top
              </label>

              <label class="flex items-center gap-2 text-sm">
                <input
                  type="checkbox"
                  checked={current().overlay_show_percentage}
                  onChange={(e) => update("overlay_show_percentage", e.currentTarget.checked)}
                />
                Show percentage in circle center (vs provider monogram)
              </label>

              <label class="flex items-center gap-2 text-sm">
                <input
                  type="checkbox"
                  checked={current().overlay_show_labels}
                  onChange={(e) => update("overlay_show_labels", e.currentTarget.checked)}
                />
                Show labels (e.g. Cursor, Antigravity)
              </label>

              <Show when={current().overlay_show_labels}>
                <div class="flex items-center gap-4 pl-6 text-sm">
                  <span class="text-xs text-muted">Position:</span>
                  <label class="flex items-center gap-1.5 text-xs">
                    <input
                      type="radio"
                      name="overlay_label_position"
                      value="beside"
                      checked={current().overlay_label_position === "beside"}
                      onChange={() => update("overlay_label_position", "beside")}
                    />
                    Beside circle
                  </label>
                  <label class="flex items-center gap-1.5 text-xs">
                    <input
                      type="radio"
                      name="overlay_label_position"
                      value="below"
                      checked={current().overlay_label_position === "below"}
                      onChange={() => update("overlay_label_position", "below")}
                    />
                    Below circle
                  </label>
                </div>
              </Show>

              <div class="mt-2 space-y-2">
                <span class="text-xs font-semibold text-muted">Items to display in widget:</span>
                <p class="text-[11px] text-muted">
                  Select specific items below, or uncheck all to show 1 circle per active provider.
                </p>

                <div class="space-y-2 max-h-40 overflow-y-auto rounded-md border border-border bg-bg/50 p-2">
                  <Show
                    when={snapshot() && snapshot()!.providers.length > 0}
                    fallback={<p class="text-xs text-muted">No providers detected yet.</p>}
                  >
                    <For each={snapshot()!.providers}>
                      {(provider) => (
                        <div class="space-y-1 border-b border-border/30 pb-2 last:border-b-0">
                          <div class="text-xs font-semibold text-fg">
                            {provider.provider_id === "antigravity"
                              ? "Antigravity (agy)"
                              : provider.display_name}
                          </div>
                          <div class="pl-2 space-y-1">
                            <For each={provider.windows}>
                              {(win) => {
                                const key = `${provider.provider_id}:${win.label}`;
                                return (
                                  <label class="flex items-center gap-2 text-xs text-muted hover:text-fg cursor-pointer">
                                    <input
                                      type="checkbox"
                                      checked={current().overlay_items.includes(key)}
                                      onChange={() => toggleOverlayItem(key, provider.provider_id)}
                                    />
                                    <span>{win.label}</span>
                                  </label>
                                );
                              }}
                            </For>
                          </div>
                        </div>
                      )}
                    </For>
                  </Show>
                </div>
              </div>
            </section>

            <section class="space-y-3 rounded-lg border border-border bg-surface/80 p-3">
              <h2 class="font-medium">Local API (YASB)</h2>
              <label class="flex items-center gap-2 text-sm">
                <input
                  type="checkbox"
                  checked={current().local_api_enabled}
                  onChange={(e) => update("local_api_enabled", e.currentTarget.checked)}
                />
                Enable localhost API
              </label>
              <label class="block space-y-1 text-sm">
                <span>API port</span>
                <input
                  type="number"
                  min={1024}
                  max={65535}
                  class="w-full rounded-md border border-border bg-bg/80 px-3 py-2"
                  value={current().local_api_port}
                  onInput={(e) => update("local_api_port", Number(e.currentTarget.value))}
                />
              </label>
            </section>

            <section class="space-y-3 rounded-lg border border-border bg-surface/80 p-3">
              <h2 class="font-medium">Providers</h2>
              <Show when={health()}>
                {(h) => (
                  <ul class="space-y-1 text-sm text-muted">
                    <li>Cursor: {h().cursor}</li>
                    <li>OpenCode Go: {h().opencode_go}</li>
                    <li>Antigravity: {h().antigravity}</li>
                  </ul>
                )}
              </Show>
              <label class="block space-y-1 text-sm">
                <span>OpenCode Go API key (optional override)</span>
                <input
                  type="password"
                  class="w-full rounded-md border border-border bg-bg/80 px-3 py-2"
                  value={current().opencode_go_api_key ?? ""}
                  onInput={(e) => update("opencode_go_api_key", e.currentTarget.value || null)}
                />
              </label>
              <label class="block space-y-1 text-sm">
                <span>Cursor state.vscdb path (optional)</span>
                <input
                  type="text"
                  class="w-full rounded-md border border-border bg-bg/80 px-3 py-2"
                  value={current().cursor_state_db_path ?? ""}
                  onInput={(e) => update("cursor_state_db_path", e.currentTarget.value || null)}
                />
              </label>
              <label class="block space-y-1 text-sm">
                <span>Antigravity state.vscdb path (optional)</span>
                <input
                  type="text"
                  class="w-full rounded-md border border-border bg-bg/80 px-3 py-2"
                  value={current().antigravity_state_db_path ?? ""}
                  onInput={(e) =>
                    update("antigravity_state_db_path", e.currentTarget.value || null)
                  }
                />
              </label>
            </section>
          </div>
        )}
      </Show>
    </WindowFrame>
  );
}
