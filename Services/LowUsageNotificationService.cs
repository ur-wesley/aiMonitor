using Microsoft.Extensions.Hosting;
using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Options;
using aiMonitor.Configuration;
using aiMonitor.Models;
using aiMonitor.Platform;

namespace aiMonitor.Services;

public sealed class LowUsageNotificationService(
    IUsageStore store,
    IOptionsMonitor<AppSettings> settings,
    WindowsToast toast,
    ILogger<LowUsageNotificationService> logger) : IHostedService
{
    private const double ThresholdPercent = 10.0;

    private readonly HashSet<string> _notifiedKeys = [];

    public Task StartAsync(CancellationToken cancellationToken)
    {
        store.Changed += OnStoreChanged;
        return Task.CompletedTask;
    }

    public Task StopAsync(CancellationToken cancellationToken)
    {
        store.Changed -= OnStoreChanged;
        return Task.CompletedTask;
    }

    private void OnStoreChanged(object? sender, EventArgs e)
    {
        if (!settings.CurrentValue.LowUsageNotificationsEnabled)
            return;

        Evaluate(store.Current);
    }

    private void Evaluate(UsageSnapshot snapshot)
    {
        foreach (var provider in snapshot.Providers)
        {
            if (provider.Error is not null || provider.Windows.Count == 0)
                continue;

            var window = FindMostDepleted(provider.Windows);
            if (window is null)
                continue;

            var remaining = GetRemainingPercent(window);
            var key = BuildKey(provider.ProviderId, window);

            if (remaining > ThresholdPercent)
            {
                _notifiedKeys.Remove(key);
                continue;
            }

            if (_notifiedKeys.Contains(key))
                continue;

            var title = $"{provider.DisplayName} quota low";
            var body = $"{window.Label}: {remaining:0}% left";
            if (toast.TryShow(title, body))
            {
                _notifiedKeys.Add(key);
                logger.LogInformation("Low usage toast sent for {Provider} {Window}", provider.ProviderId, window.Label);
            }
        }
    }

    private static UsageWindowMetric? FindMostDepleted(IReadOnlyList<UsageWindowMetric> windows)
    {
        UsageWindowMetric? worst = null;
        var lowestRemaining = double.MaxValue;

        foreach (var window in windows)
        {
            var remaining = GetRemainingPercent(window);
            if (remaining >= lowestRemaining)
                continue;

            lowestRemaining = remaining;
            worst = window;
        }

        return worst;
    }

    private static double GetRemainingPercent(UsageWindowMetric window) =>
        window.IsRemainingPercent ? window.Value : 100.0 - window.Value;

    private static string BuildKey(string providerId, UsageWindowMetric window) =>
        $"{providerId}|{window.Label}|{window.ResetsAt?.UtcTicks ?? 0}";
}
