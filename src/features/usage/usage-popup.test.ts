import { describe, expect, it } from "vitest";

import type { ProviderUsage, UsageSnapshot } from "~/types/usage";

function mergeSnapshot(
  current: UsageSnapshot | null,
  next: UsageSnapshot,
): UsageSnapshot {
  return {
    fetched_at: next.fetched_at,
    providers: next.providers,
  };
}

describe("usage snapshot merge", () => {
  it("replaces providers instead of appending", () => {
    const provider: ProviderUsage = {
      provider_id: "cursor",
      display_name: "Cursor",
      windows: [],
      error: null,
      fetched_at: "2026-01-01T00:00:00Z",
    };
    const first: UsageSnapshot = {
      fetched_at: "2026-01-01T00:00:00Z",
      providers: [provider],
    };
    const second: UsageSnapshot = {
      fetched_at: "2026-01-02T00:00:00Z",
      providers: [provider],
    };

    const mergedOnce = mergeSnapshot(null, first);
    const mergedTwice = mergeSnapshot(mergedOnce, second);

    expect(mergedTwice.providers).toHaveLength(1);
    expect(mergedTwice.fetched_at).toBe(second.fetched_at);
  });
});
