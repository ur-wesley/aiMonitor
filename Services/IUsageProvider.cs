using aiMonitor.Models;

namespace aiMonitor.Services;

public interface IUsageProvider
{
    string ProviderId { get; }

    string DisplayName { get; }

    Task<ProviderUsage> FetchAsync(CancellationToken ct);
}
