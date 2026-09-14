namespace aiMonitor.Models;

public sealed record ProviderUsage(
    string ProviderId,
    string DisplayName,
    IReadOnlyList<UsageWindowMetric> Windows,
    string? Error,
    DateTimeOffset FetchedAt);
