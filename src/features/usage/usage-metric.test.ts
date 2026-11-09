import { describe, expect, it } from "vitest";

import {
  circleDashOffset,
  minRemainingPercent,
  remainingPercent,
  remainingStatusClass,
  remainingStrokeClass,
} from "~/features/usage/usage-metric";
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

describe("remainingStrokeClass and remainingStatusClass", () => {
  it("returns text-critical and bg-critical for <= 5", () => {
    expect(remainingStrokeClass(5)).toBe("text-critical");
    expect(remainingStrokeClass(2)).toBe("text-critical");
    expect(remainingStatusClass(5)).toBe("bg-critical");
    expect(remainingStatusClass(2)).toBe("bg-critical");
  });

  it("returns text-near-limit and bg-near-limit for 6..20", () => {
    expect(remainingStrokeClass(15)).toBe("text-near-limit");
    expect(remainingStrokeClass(20)).toBe("text-near-limit");
    expect(remainingStatusClass(15)).toBe("bg-near-limit");
    expect(remainingStatusClass(20)).toBe("bg-near-limit");
  });

  it("returns text-ok and bg-ok for > 20", () => {
    expect(remainingStrokeClass(21)).toBe("text-ok");
    expect(remainingStrokeClass(100)).toBe("text-ok");
    expect(remainingStatusClass(21)).toBe("bg-ok");
    expect(remainingStatusClass(100)).toBe("bg-ok");
  });
});

describe("circleDashOffset", () => {
  it("calculates correct dash offset for full and empty", () => {
    const circumference = 100;
    expect(circleDashOffset(100, circumference)).toBe(0);
    expect(circleDashOffset(0, circumference)).toBe(100);
    expect(circleDashOffset(50, circumference)).toBe(50);
  });

  it("clamps out of range percentages", () => {
    const circumference = 100;
    expect(circleDashOffset(120, circumference)).toBe(0);
    expect(circleDashOffset(-10, circumference)).toBe(100);
  });
});
