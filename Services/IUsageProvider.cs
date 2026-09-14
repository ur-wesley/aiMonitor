using UsageTray.Models;

namespace UsageTray.Services;

public interface IUsageProvider
{
    string ProviderId { get; }

    string DisplayName { get; }

    Task<ProviderUsage> FetchAsync(CancellationToken ct);
}
