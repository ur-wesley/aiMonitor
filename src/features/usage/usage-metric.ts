import type { UsageWindowMetric } from "~/types/usage";

export function remainingPercent(window: UsageWindowMetric): number {
  const value = Math.min(100, Math.max(0, window.value));
  return window.is_remaining_percent ? value : 100 - value;
}

export function minRemainingPercent(windows: readonly UsageWindowMetric[]): number | null {
  if (windows.length === 0) return null;
  return Math.min(...windows.map(remainingPercent));
}

export function remainingStatusClass(remaining: number): string {
  if (remaining <= 5) return "bg-critical";
  if (remaining <= 20) return "bg-near-limit";
  return "bg-ok";
}
