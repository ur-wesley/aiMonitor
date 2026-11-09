export function getMonogram(providerId: string): string {
  const id = providerId.toLowerCase();
  if (id.includes("cursor")) return "C";
  if (id.includes("antigravity")) return "AGY";
  if (id.includes("opencode")) return "O";
  return providerId.slice(0, 2).toUpperCase();
}

export function getShortLabel(
  providerId: string,
  windowLabel?: string,
  itemCountForProvider = 1,
): string {
  const id = providerId.toLowerCase();
  let base = "";
  if (id.includes("cursor")) base = "Cursor";
  else if (id.includes("antigravity")) base = "agy";
  else if (id.includes("opencode")) base = "OpenCode";
  else base = providerId;

  // When only 1 item for this provider is configured or no windowLabel is provided,
  // simply show the provider base label (e.g. "agy", "Cursor").
  if (!windowLabel || itemCountForProvider <= 1) {
    return base;
  }

  // Clean up window label - strip out "Claude", "requests", "total"
  const cleanWin = windowLabel
    .replace(/claude/i, "")
    .replace(/requests?/i, "")
    .trim();

  if (
    cleanWin.length === 0 ||
    cleanWin.toLowerCase() === "total" ||
    cleanWin.toLowerCase() === base.toLowerCase()
  ) {
    return base;
  }

  return `${base} ${cleanWin}`.trim();
}

export function formatReset(resetsAt: string | null): string {
  if (!resetsAt) return "";
  const reset = new Date(resetsAt);
  const delta = reset.getTime() - Date.now();
  if (delta <= 0) return "resets soon";
  const hours = Math.floor(delta / 3_600_000);
  const minutes = Math.floor((delta % 3_600_000) / 60_000);
  if (hours >= 24) return `resets in ${Math.floor(hours / 24)}d ${hours % 24}h`;
  if (hours >= 1) return `resets in ${hours}h ${minutes}m`;
  return `resets in ${minutes}m`;
}
