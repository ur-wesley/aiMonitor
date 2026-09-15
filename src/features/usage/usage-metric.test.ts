import { describe, expect, it } from "vitest";

import { minRemainingPercent, remainingPercent } from "~/features/usage/usage-metric";
import type { UsageWindowMetric } from "~/types/usage";

function metric(value: number, isRemaining: boolean): UsageWindowMetric {
  return {
    label: "test",
    value,
    is_remaining_percent: isRemaining,
    resets_at: null,
  };
}

describe("remainingPercent", () => {
  it("returns value when already remaining", () => {
    expect(remainingPercent(metric(25, true))).toBe(25);
  });

  it("inverts used percent to remaining", () => {
    expect(remainingPercent(metric(75, false))).toBe(25);
  });
});

describe("minRemainingPercent", () => {
  it("returns lowest remaining across windows", () => {
    const min = minRemainingPercent([metric(80, false), metric(40, true)]);
    expect(min).toBe(20);
  });
});
