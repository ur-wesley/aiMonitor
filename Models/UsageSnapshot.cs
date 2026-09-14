namespace aiMonitor.Models;

public sealed record UsageSnapshot(
    IReadOnlyList<ProviderUsage> Providers,
    DateTimeOffset FetchedAt)
{
    public static UsageSnapshot Empty => new([], DateTimeOffset.UtcNow);
}
