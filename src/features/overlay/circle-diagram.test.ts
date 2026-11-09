import { describe, expect, it } from "vitest";

import { getMonogram, getShortLabel } from "~/features/overlay/overlay-helpers";
import {
  circleDashOffset,
  remainingStatusClass,
  remainingStrokeClass,
} from "~/features/usage/usage-metric";

describe("CircleDiagram metrics integration", () => {
  it("determines correct stroke color class based on remaining threshold", () => {
    // Healthy
    expect(remainingStrokeClass(90)).toBe("text-ok");
    expect(remainingStatusClass(90)).toBe("bg-ok");

    // Near limit
    expect(remainingStrokeClass(15)).toBe("text-near-limit");
    expect(remainingStatusClass(15)).toBe("bg-near-limit");

    // Critical
    expect(remainingStrokeClass(3)).toBe("text-critical");
    expect(remainingStatusClass(3)).toBe("bg-critical");
  });

  it("calculates accurate circle circumference dash offsets", () => {
    const circumference = 78.5398;

    // 100% full remaining -> 0 offset
    expect(circleDashOffset(100, circumference)).toBeCloseTo(0, 2);

    // 0% remaining -> full offset (hidden progress)
    expect(circleDashOffset(0, circumference)).toBeCloseTo(circumference, 2);

    // 50% remaining -> half offset
    expect(circleDashOffset(50, circumference)).toBeCloseTo(circumference / 2, 2);

    // 25% remaining -> 75% offset
    expect(circleDashOffset(25, circumference)).toBeCloseTo(circumference * 0.75, 2);
  });
});

describe("overlay-helpers", () => {
  it("formats antigravity label as agy and strips claude", () => {
    expect(getShortLabel("antigravity")).toBe("agy");
    expect(getShortLabel("antigravity", "Claude Weekly", 1)).toBe("agy");
    expect(getShortLabel("antigravity", "Claude Weekly", 2)).toBe("agy Weekly");
    expect(getShortLabel("antigravity", "Claude", 2)).toBe("agy");
    expect(getMonogram("antigravity")).toBe("AGY");
  });

  it("formats cursor labels and avoids redundant Total", () => {
    expect(getShortLabel("cursor")).toBe("Cursor");
    expect(getShortLabel("cursor", "Total", 1)).toBe("Cursor");
    expect(getShortLabel("cursor", "Auto", 1)).toBe("Cursor");
    expect(getShortLabel("cursor", "Auto", 2)).toBe("Cursor Auto");
    expect(getMonogram("cursor")).toBe("C");
  });

  it("formats opencode labels", () => {
    expect(getShortLabel("opencode_go")).toBe("OpenCode");
    expect(getShortLabel("opencode_go", "Monthly", 1)).toBe("OpenCode");
    expect(getMonogram("opencode_go")).toBe("O");
  });
});
