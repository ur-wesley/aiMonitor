using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using aiMonitor.Configuration;
using aiMonitor.Platform;

namespace aiMonitor.ViewModels;

public partial class SettingsViewModel : ObservableObject
{
    private readonly SettingsStore _settingsStore;
    private readonly AppSettings _appSettings;

    public SettingsViewModel(SettingsStore settingsStore, AppSettings appSettings)
    {
        _settingsStore = settingsStore;
        _appSettings = appSettings;
        RefreshIntervalSeconds = appSettings.RefreshIntervalSeconds;
        LocalApiEnabled = appSettings.LocalApiEnabled;
        LocalApiPort = appSettings.LocalApiPort;
        OpenCodeGoApiKey = appSettings.OpenCodeGoApiKey ?? string.Empty;
        CursorStateDbPath = appSettings.CursorStateDbPath ?? string.Empty;
        AntigravityStateDbPath = appSettings.AntigravityStateDbPath ?? string.Empty;
        StartWithWindows = AutoStart.IsEnabled();
        LowUsageNotificationsEnabled = appSettings.LowUsageNotificationsEnabled;
    }

    [ObservableProperty]
    private int _refreshIntervalSeconds = 300;

    [ObservableProperty]
    private bool _localApiEnabled = true;

    [ObservableProperty]
    private int _localApiPort = 6736;

    [ObservableProperty]
    private string _openCodeGoApiKey = string.Empty;

    [ObservableProperty]
    private string _cursorStateDbPath = string.Empty;

    [ObservableProperty]
    private string _antigravityStateDbPath = string.Empty;

    [ObservableProperty]
    private bool _startWithWindows;

    [ObservableProperty]
    private bool _lowUsageNotificationsEnabled = true;

    [ObservableProperty]
    private string _statusMessage = string.Empty;

    [RelayCommand]
    private void Save()
    {
        _appSettings.RefreshIntervalSeconds = Math.Max(60, RefreshIntervalSeconds);
        _appSettings.LocalApiEnabled = LocalApiEnabled;
        _appSettings.LocalApiPort = LocalApiPort;
        _appSettings.OpenCodeGoApiKey = string.IsNullOrWhiteSpace(OpenCodeGoApiKey) ? null : OpenCodeGoApiKey.Trim();
        _appSettings.CursorStateDbPath = string.IsNullOrWhiteSpace(CursorStateDbPath) ? null : CursorStateDbPath.Trim();
        _appSettings.AntigravityStateDbPath = string.IsNullOrWhiteSpace(AntigravityStateDbPath) ? null : AntigravityStateDbPath.Trim();
        _appSettings.StartWithWindows = StartWithWindows;
        _appSettings.LowUsageNotificationsEnabled = LowUsageNotificationsEnabled;

        _settingsStore.Save(_appSettings);

        if (!AutoStart.SetEnabled(StartWithWindows))
        {
            StatusMessage = "Settings saved, but autostart could not be updated.";
            return;
        }

        StatusMessage = "Saved. Restart aiMonitor for API port changes.";
    }
}
