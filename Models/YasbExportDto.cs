using System.Text.Json.Serialization;

namespace aiMonitor.Models;

public sealed class YasbExportDto
{
    [JsonPropertyName("cursor")]
    public YasbCursorDto Cursor { get; set; } = new();

    [JsonPropertyName("opencode_go")]
    public YasbOpenCodeGoDto OpenCodeGo { get; set; } = new();

    [JsonPropertyName("antigravity")]
    public YasbAntigravityDto Antigravity { get; set; } = new();

    [JsonPropertyName("max_used")]
    public double MaxUsed { get; set; }

    [JsonPropertyName("status")]
    public string Status { get; set; } = "ok";

    [JsonPropertyName("fetched_at")]
    public string FetchedAt { get; set; } = string.Empty;

    [JsonPropertyName("label")]
    public string Label { get; set; } = "AI…";

    [JsonPropertyName("label_alt")]
    public string LabelAlt { get; set; } = string.Empty;

    [JsonPropertyName("tooltip")]
    public string Tooltip { get; set; } = string.Empty;

    [JsonPropertyName("status_class")]
    public string StatusClass { get; set; } = "ok";
}

public sealed class YasbCursorDto
{
    [JsonPropertyName("total")]
    public double? Total { get; set; }

    [JsonPropertyName("auto")]
    public double? Auto { get; set; }

    [JsonPropertyName("api")]
    public double? Api { get; set; }

    [JsonPropertyName("resets_at")]
    public string? ResetsAt { get; set; }

    [JsonPropertyName("error")]
    public string? Error { get; set; }
}

public sealed class YasbOpenCodeGoDto
{
    [JsonPropertyName("rolling")]
    public double? Rolling { get; set; }

    [JsonPropertyName("weekly")]
    public double? Weekly { get; set; }

    [JsonPropertyName("monthly")]
    public double? Monthly { get; set; }

    [JsonPropertyName("error")]
    public string? Error { get; set; }
}

public sealed class YasbAntigravityDto
{
    [JsonPropertyName("gemini_5h")]
    public double? Gemini5h { get; set; }

    [JsonPropertyName("gemini_weekly")]
    public double? GeminiWeekly { get; set; }

    [JsonPropertyName("claude_5h")]
    public double? Claude5h { get; set; }

    [JsonPropertyName("claude_weekly")]
    public double? ClaudeWeekly { get; set; }

    [JsonPropertyName("error")]
    public string? Error { get; set; }
}
