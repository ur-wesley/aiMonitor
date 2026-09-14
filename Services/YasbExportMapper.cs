using UsageTray.Models;

namespace UsageTray.Services;

public static class YasbExportMapper
{
    public static YasbExportDto ToDto(UsageSnapshot snapshot)
    {
        var dto = new YasbExportDto
        {
            FetchedAt = snapshot.FetchedAt.ToString("O"),
        };

        var maxUsed = 0.0;

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
                    foreach (var w in provider.Windows.Where(w => !w.IsRemainingPercent))
                    {
                        maxUsed = Math.Max(maxUsed, w.Value);
                    }
                    break;
                case "antigravity":
                    MapAntigravity(provider, dto.Antigravity);
                    break;
            }
        }

        dto.MaxUsed = maxUsed;
        dto.Status = snapshot.Providers.Any(p => p.Error is not null) ? "partial" : "ok";
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
}
