using aiMonitor.Models;

namespace aiMonitor.Services;

public static class YasbExportMapper
{
    public static YasbExportDto ToDto(UsageSnapshot snapshot)
    {
        var dto = new YasbExportDto
        {
            FetchedAt = snapshot.FetchedAt.ToLocalTime().ToString("g"),
        };

        var maxUsed = 0.0;
        var minRemaining = 100.0;

        foreach (var provider in snapshot.Providers)
        {
            switch (provider.ProviderId)
            {
                case "cursor":
                    MapCursor(provider, dto.Cursor);
                    if (dto.Cursor.Total is double total)
                        maxUsed = Math.Max(maxUsed, total);
                    break;
                case "opencode_go":
                    MapOpenCode(provider, dto.OpenCodeGo);
                    foreach (var window in provider.Windows.Where(w => !w.IsRemainingPercent))
                        maxUsed = Math.Max(maxUsed, window.Value);
                    break;
                case "antigravity":
                    MapAntigravity(provider, dto.Antigravity);
                    foreach (var window in provider.Windows.Where(w => w.IsRemainingPercent))
                        minRemaining = Math.Min(minRemaining, window.Value);
                    break;
            }
        }

        dto.MaxUsed = maxUsed;
        dto.Status = snapshot.Providers.Any(p => p.Error is not null) ? "partial" : "ok";
        dto.StatusClass = GetStatusClass(maxUsed, minRemaining);
        dto.Label = BuildLabel(dto);
        dto.LabelAlt = BuildLabelAlt(dto);
        dto.Tooltip = BuildTooltip(dto);
        RoundDisplayNumbers(dto);
        return dto;
    }

    private static void MapCursor(ProviderUsage provider, YasbCursorDto dto)
    {
        dto.Error = provider.Error;
        if (provider.Error is not null)
            return;

        dto.Total = GetValue(provider, "Total");
        dto.Auto = GetValue(provider, "Auto");
        dto.Api = GetValue(provider, "API");
        dto.ResetsAt = GetReset(provider)?.ToLocalTime().ToString("g");
    }

    private static void MapOpenCode(ProviderUsage provider, YasbOpenCodeGoDto dto)
    {
        dto.Error = provider.Error;
        if (provider.Error is not null)
            return;

        dto.Rolling = GetValue(provider, "5h");
        dto.Weekly = GetValue(provider, "Weekly");
        dto.Monthly = GetValue(provider, "Monthly");
    }

    private static void MapAntigravity(ProviderUsage provider, YasbAntigravityDto dto)
    {
        dto.Error = provider.Error;
        if (provider.Error is not null)
            return;

        dto.Gemini5h = GetValue(provider, "Gemini 5h");
        dto.GeminiWeekly = GetValue(provider, "Gemini Weekly");
        dto.Claude5h = GetValue(provider, "Claude/GPT 5h");
        dto.ClaudeWeekly = GetValue(provider, "Claude/GPT Weekly");
    }

    private static double? GetValue(ProviderUsage provider, string label)
    {
        var window = provider.Windows.FirstOrDefault(w =>
            string.Equals(w.Label, label, StringComparison.OrdinalIgnoreCase));
        return window?.Value;
    }

    private static DateTimeOffset? GetReset(ProviderUsage provider)
    {
        return provider.Windows.Select(w => w.ResetsAt).FirstOrDefault(r => r is not null);
    }

    private static string GetStatusClass(double maxUsed, double minRemaining)
    {
        if (maxUsed >= 95 || minRemaining <= 5)
            return "critical";

        if (maxUsed >= 80 || minRemaining <= 20)
            return "near-limit";

        return "ok";
    }

    private static void RoundDisplayNumbers(YasbExportDto dto)
    {
        dto.Cursor.Total = Round(dto.Cursor.Total);
        dto.Cursor.Auto = Round(dto.Cursor.Auto);
        dto.Cursor.Api = Round(dto.Cursor.Api);
        dto.OpenCodeGo.Rolling = Round(dto.OpenCodeGo.Rolling);
        dto.OpenCodeGo.Weekly = Round(dto.OpenCodeGo.Weekly);
        dto.OpenCodeGo.Monthly = Round(dto.OpenCodeGo.Monthly);
        dto.Antigravity.Gemini5h = Round(dto.Antigravity.Gemini5h);
        dto.Antigravity.GeminiWeekly = Round(dto.Antigravity.GeminiWeekly);
        dto.Antigravity.Claude5h = Round(dto.Antigravity.Claude5h);
        dto.Antigravity.ClaudeWeekly = Round(dto.Antigravity.ClaudeWeekly);
    }

    private static double? Round(double? value) =>
        value is null ? null : Math.Round(value.Value);

    private static string BuildLabel(YasbExportDto dto) =>
        $"C {FormatUsed(dto.Cursor.Total, dto.Cursor.Error)}" +
        $" · G {FormatUsed(dto.OpenCodeGo.Rolling, dto.OpenCodeGo.Error)}" +
        $" · AG {FormatRemaining(dto.Antigravity.Gemini5h, dto.Antigravity.Error)}";

    private static string BuildLabelAlt(YasbExportDto dto)
    {
        var cursor = dto.Cursor.Error is not null
            ? "Cursor —"
            : $"Cursor {FormatUsed(dto.Cursor.Total)} / auto {FormatUsed(dto.Cursor.Auto)}";

        var go = dto.OpenCodeGo.Error is not null
            ? "Go —"
            : $"Go 5h {FormatUsed(dto.OpenCodeGo.Rolling)} / wk {FormatUsed(dto.OpenCodeGo.Weekly)}";

        var ag = dto.Antigravity.Error is not null
            ? "AG —"
            : $"AG G5h {FormatRemaining(dto.Antigravity.Gemini5h)} / C5h {FormatRemaining(dto.Antigravity.Claude5h)}";

        return $"{cursor} · {go} · {ag}";
    }

    private static string BuildTooltip(YasbExportDto dto)
    {
        var lines = new List<string>();

        if (dto.Cursor.Error is not null)
            lines.Add($"Cursor: {dto.Cursor.Error}");
        else
        {
            lines.Add($"Cursor total: {FormatUsed(dto.Cursor.Total)}");
            lines.Add($"Cursor auto: {FormatUsed(dto.Cursor.Auto)} · API: {FormatUsed(dto.Cursor.Api)}");
            if (!string.IsNullOrWhiteSpace(dto.Cursor.ResetsAt))
                lines.Add($"Cursor resets: {dto.Cursor.ResetsAt}");
        }

        if (dto.OpenCodeGo.Error is not null)
            lines.Add($"OpenCode Go: {dto.OpenCodeGo.Error}");
        else
        {
            lines.Add($"OpenCode Go 5h: {FormatUsed(dto.OpenCodeGo.Rolling)}");
            lines.Add($"OpenCode Go weekly: {FormatUsed(dto.OpenCodeGo.Weekly)} · monthly: {FormatUsed(dto.OpenCodeGo.Monthly)}");
        }

        if (dto.Antigravity.Error is not null)
            lines.Add($"Antigravity: {dto.Antigravity.Error}");
        else
        {
            lines.Add($"Antigravity Gemini 5h: {FormatRemaining(dto.Antigravity.Gemini5h)} left");
            lines.Add($"Antigravity Gemini weekly: {FormatRemaining(dto.Antigravity.GeminiWeekly)} left");
            lines.Add($"Antigravity Claude/GPT 5h: {FormatRemaining(dto.Antigravity.Claude5h)} left");
            lines.Add($"Antigravity Claude/GPT weekly: {FormatRemaining(dto.Antigravity.ClaudeWeekly)} left");
        }

        lines.Add($"Updated: {dto.FetchedAt}");
        return string.Join('\n', lines);
    }

    private static string FormatUsed(double? value, string? error = null) =>
        error is not null ? "—" : value is null ? "—" : $"{Math.Round(value.Value):0}%";

    private static string FormatRemaining(double? value, string? error = null) =>
        error is not null ? "—" : value is null ? "—" : $"{Math.Round(value.Value):0}%";
}
