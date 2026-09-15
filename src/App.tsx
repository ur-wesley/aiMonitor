import { createSignal, onMount, Show } from "solid-js";
import { getCurrentWindow } from "@tauri-apps/api/window";

import { SettingsPanel } from "~/features/settings/settings-panel";
import { UsagePopup } from "~/features/usage/usage-popup";

async function readWindowLabel(): Promise<string> {
  const window = getCurrentWindow();
  const value = window.label;
  return typeof value === "string" ? value : await value;
}

export default function App() {
  const [label, setLabel] = createSignal<string | null>(null);

  onMount(() => {
    void readWindowLabel().then(setLabel);
  });

  return (
    <Show when={label()} keyed>
      {(windowLabel) => (
        <Show when={windowLabel === "usage"} fallback={
          <Show when={windowLabel === "settings"} fallback={<div />}>
            <SettingsPanel />
          </Show>
        }>
          <UsagePopup />
        </Show>
      )}
    </Show>
  );
}
