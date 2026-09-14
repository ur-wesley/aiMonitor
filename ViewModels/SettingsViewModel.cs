using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using aiMonitor.Configuration;

namespace aiMonitor.ViewModels;

public partial class SettingsViewModel : ObservableObject
{
    private readonly SettingsStore _settingsStore;

    public SettingsViewModel(SettingsStore settingsStore)
    {
        _settingsStore = settingsStore;
        var settings = settingsStore.Load();
        RefreshIntervalSeconds = settings.RefreshIntervalSeconds;
        LocalApiEnabled = settings.LocalApiEnabled;
        LocalApiPort = settings.LocalApiPort;
        OpenCodeGoApiKey = settings.OpenCodeGoApiKey ?? string.Empty;
        CursorStateDbPath = settings.CursorStateDbPath ?? string.Empty;
        AntigravityStateDbPath = settings.AntigravityStateDbPath ?? string.Empty;
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
    private string _statusMessage = string.Empty;

    [RelayCommand]
    private void Save()
    {
        var settings = new AppSettings
        {
            RefreshIntervalSeconds = Math.Max(60, RefreshIntervalSeconds),
            LocalApiEnabled = LocalApiEnabled,
            LocalApiPort = LocalApiPort,
            OpenCodeGoApiKey = string.IsNullOrWhiteSpace(OpenCodeGoApiKey) ? null : OpenCodeGoApiKey.Trim(),
            CursorStateDbPath = string.IsNullOrWhiteSpace(CursorStateDbPath) ? null : CursorStateDbPath.Trim(),
            AntigravityStateDbPath = string.IsNullOrWhiteSpace(AntigravityStateDbPath) ? null : AntigravityStateDbPath.Trim(),
        };

        _settingsStore.Save(settings);
        StatusMessage = "Saved. Restart aiMonitor for API port changes.";
    }
}
