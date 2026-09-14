using System.Text.Json.Serialization;

namespace UsageTray.Models.ExternalApi;

public sealed class CursorUsageSummaryResponse
{
    [JsonPropertyName("individualUsage")]
    public CursorIndividualUsage? IndividualUsage { get; set; }

    [JsonPropertyName("autoModelSelectedDisplayMessage")]
    public string? AutoModelSelectedDisplayMessage { get; set; }

    [JsonPropertyName("namedModelSelectedDisplayMessage")]
    public string? NamedModelSelectedDisplayMessage { get; set; }
}

public sealed class CursorIndividualUsage
{
    [JsonPropertyName("plan")]
    public CursorPlanUsage? Plan { get; set; }

    [JsonPropertyName("billingCycleEnd")]
    public string? BillingCycleEnd { get; set; }
}

public sealed class CursorPlanUsage
{
    [JsonPropertyName("totalPercentUsed")]
    public double? TotalPercentUsed { get; set; }

    [JsonPropertyName("autoPercentUsed")]
    public double? AutoPercentUsed { get; set; }

    [JsonPropertyName("apiPercentUsed")]
    public double? ApiPercentUsed { get; set; }
}
