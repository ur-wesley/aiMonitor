import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ResultAsync } from "@ur-wesley/ts-prelude/result";

import type { AppSettings, CredentialHealth, UsageSnapshot } from "~/types/usage";

function mapInvokeError(error: unknown): { code: string; message: string } {
  return {
    code: "invoke",
    message: error instanceof Error ? error.message : String(error),
  };
}

export function getUsageSnapshot(): ResultAsync<UsageSnapshot, { code: string; message: string }> {
  return ResultAsync.fromPromise(invoke<UsageSnapshot>("get_usage_snapshot"), mapInvokeError);
}

export function refreshUsage(): ResultAsync<UsageSnapshot, { code: string; message: string }> {
  return ResultAsync.fromPromise(invoke<UsageSnapshot>("refresh_usage"), mapInvokeError);
}

export function getSettings(): ResultAsync<AppSettings, { code: string; message: string }> {
  return ResultAsync.fromPromise(invoke<AppSettings>("get_settings"), mapInvokeError);
}

export function saveSettings(
  settings: AppSettings,
): ResultAsync<string, { code: string; message: string }> {
  return ResultAsync.fromPromise(invoke<string>("save_app_settings", { settings }), mapInvokeError);
}

export function checkCredentials(): ResultAsync<CredentialHealth, { code: string; message: string }> {
  return ResultAsync.fromPromise(invoke<CredentialHealth>("check_credentials"), mapInvokeError);
}

export function hideUsageWindow(): ResultAsync<void, { code: string; message: string }> {
  return ResultAsync.fromPromise(invoke<void>("hide_usage_window"), mapInvokeError);
}

export function placeUsageWindow(): ResultAsync<void, { code: string; message: string }> {
  return ResultAsync.fromPromise(invoke<void>("place_usage_window"), mapInvokeError);
}

export function showSettingsWindow(): ResultAsync<void, { code: string; message: string }> {
  return ResultAsync.fromPromise(invoke<void>("show_settings_window"), mapInvokeError);
}

export function quitApp(): ResultAsync<void, { code: string; message: string }> {
  return ResultAsync.fromPromise(invoke<void>("quit_app"), mapInvokeError);
}

export function onUsageUpdated(
  handler: (snapshot: UsageSnapshot) => void,
): ResultAsync<() => void, { code: string; message: string }> {
  return ResultAsync.fromPromise(
    listen<UsageSnapshot>("usage-updated", (event) => handler(event.payload)),
    mapInvokeError,
  ).map((unlisten) => () => {
    void unlisten();
  });
}
