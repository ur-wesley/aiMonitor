using System.Collections.ObjectModel;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using UsageTray.Models;
using UsageTray.Services;

namespace UsageTray.ViewModels;

public partial class UsageViewModel : ObservableObject
{
    private readonly IUsageStore _store;
    private readonly UsageRefreshBackgroundService _refreshService;

    public UsageViewModel(IUsageStore store, UsageRefreshBackgroundService refreshService)
    {
        _store = store;
        _refreshService = refreshService;
        _store.Changed += (_, _) => SyncFromStore();
        SyncFromStore();
    }

    public ObservableCollection<ProviderCardViewModel> Providers { get; } = [];

    [ObservableProperty]
    private string _lastUpdated = "—";

    [ObservableProperty]
    private bool _isRefreshing;

    [RelayCommand]
    private async Task RefreshAsync()
    {
        IsRefreshing = true;
        try
        {
            await _refreshService.RefreshOnceAsync(CancellationToken.None);
        }
        finally
        {
            IsRefreshing = false;
        }
    }

    private void SyncFromStore()
    {
        var snapshot = _store.Current;
        LastUpdated = snapshot.FetchedAt.ToLocalTime().ToString("g");

        Providers.Clear();
        foreach (var provider in snapshot.Providers)
            Providers.Add(ProviderCardViewModel.From(provider));
    }
}

public partial class ProviderCardViewModel : ObservableObject
{
    [ObservableProperty]
    private string _displayName = string.Empty;

    [ObservableProperty]
    private string? _error;

    public bool HasError => !string.IsNullOrWhiteSpace(Error);
    public ObservableCollection<UsageBarViewModel> Bars { get; } = [];

    public static ProviderCardViewModel From(ProviderUsage provider)
    {
        var card = new ProviderCardViewModel
        {
            DisplayName = provider.DisplayName,
            Error = provider.Error,
        };

        foreach (var window in provider.Windows)
        {
            var displayValue = window.IsRemainingPercent
                ? window.Value
                : window.Value;

            var max = 100.0;
            card.Bars.Add(new UsageBarViewModel
            {
                Label = window.Label,
                Value = displayValue,
                Maximum = max,
                IsRemaining = window.IsRemainingPercent,
                ResetText = FormatReset(window.ResetsAt),
                StatusClass = GetStatusClass(window),
            });
        }

        return card;
    }

    private static string GetStatusClass(UsageWindowMetric window)
    {
        if (window.IsRemainingPercent)
        {
            if (window.Value <= 5) return "critical";
            if (window.Value <= 20) return "near-limit";
            return "ok";
        }

        if (window.Value >= 95) return "critical";
        if (window.Value >= 80) return "near-limit";
        return "ok";
    }

    private static string FormatReset(DateTimeOffset? resetsAt)
    {
        if (resetsAt is null)
            return string.Empty;

        var delta = resetsAt.Value - DateTimeOffset.UtcNow;
        if (delta <= TimeSpan.Zero)
            return "resets soon";

        if (delta.TotalHours >= 24)
            return $"resets in {(int)delta.TotalDays}d {delta.Hours}h";

        if (delta.TotalHours >= 1)
            return $"resets in {(int)delta.TotalHours}h {delta.Minutes}m";

        return $"resets in {(int)delta.TotalMinutes}m";
    }
}

public partial class UsageBarViewModel : ObservableObject
{
    [ObservableProperty]
    private string _label = string.Empty;

    [ObservableProperty]
    private double _value;

    [ObservableProperty]
    private double _maximum = 100;

    [ObservableProperty]
    private bool _isRemaining;

    [ObservableProperty]
    private string _resetText = string.Empty;

    [ObservableProperty]
    private string _statusClass = "ok";

    public string DisplayText => IsRemaining ? $"{Value:0}% left" : $"{Value:0}% used";
}
