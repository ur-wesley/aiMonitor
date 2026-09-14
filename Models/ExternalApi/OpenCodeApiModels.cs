using System.Text.Json.Serialization;

namespace aiMonitor.Models.ExternalApi;

public sealed class OpenCodeGoUsageResponse
{
    [JsonPropertyName("usage")]
    public OpenCodeGoUsageBuckets? Usage { get; set; }
}

public sealed class OpenCodeGoUsageBuckets
{
    [JsonPropertyName("rolling")]
    public OpenCodeGoUsageWindow? Rolling { get; set; }

    [JsonPropertyName("weekly")]
    public OpenCodeGoUsageWindow? Weekly { get; set; }

    [JsonPropertyName("monthly")]
    public OpenCodeGoUsageWindow? Monthly { get; set; }
}

public sealed class OpenCodeGoUsageWindow
{
    [JsonPropertyName("status")]
    public string? Status { get; set; }

    [JsonPropertyName("percent")]
    public double? Percent { get; set; }

    [JsonPropertyName("resetsAt")]
    public string? ResetsAt { get; set; }
}
