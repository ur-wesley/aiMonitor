using Microsoft.Extensions.Hosting;
using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Options;
using aiMonitor.Configuration;
using aiMonitor.Models;

namespace aiMonitor.Services;

public sealed class UsageRefreshBackgroundService(
    IEnumerable<IUsageProvider> providers,
    IUsageStore store,
    IOptionsMonitor<AppSettings> settings,
    ILogger<UsageRefreshBackgroundService> logger) : BackgroundService
{
    protected override async Task ExecuteAsync(CancellationToken stoppingToken)
    {
        await RefreshOnceAsync(stoppingToken).ConfigureAwait(false);

        while (!stoppingToken.IsCancellationRequested)
        {
            var interval = Math.Max(60, settings.CurrentValue.RefreshIntervalSeconds);
            try
            {
                await Task.Delay(TimeSpan.FromSeconds(interval), stoppingToken).ConfigureAwait(false);
            }
            catch (OperationCanceledException)
            {
                break;
            }

            await RefreshOnceAsync(stoppingToken).ConfigureAwait(false);
        }
    }

    public async Task RefreshOnceAsync(CancellationToken ct)
    {
        var tasks = providers.Select(p => FetchSafeAsync(p, ct)).ToArray();
        var results = await Task.WhenAll(tasks).ConfigureAwait(false);

        store.Update(new UsageSnapshot(results.ToList(), DateTimeOffset.UtcNow));
        logger.LogInformation("Usage refreshed for {Count} providers", results.Length);
    }

    private static async Task<ProviderUsage> FetchSafeAsync(IUsageProvider provider, CancellationToken ct)
    {
        try
        {
            return await provider.FetchAsync(ct).ConfigureAwait(false);
        }
        catch (Exception ex)
        {
            return new ProviderUsage(
                provider.ProviderId,
                provider.DisplayName,
                [],
                ex.Message,
                DateTimeOffset.UtcNow);
        }
    }
}
