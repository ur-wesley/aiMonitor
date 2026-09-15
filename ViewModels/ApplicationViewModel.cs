using System.Windows.Input;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Threading;
using Avalonia.Controls.ApplicationLifetimes;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using aiMonitor.Platform;
using aiMonitor.Services;
using aiMonitor.Views;

namespace aiMonitor.ViewModels;

public partial class ApplicationViewModel : ObservableObject
{
    private readonly UsageViewModel _usageViewModel;
    private readonly SettingsViewModel _settingsViewModel;
    private readonly UsageRefreshBackgroundService _refreshService;
    private readonly IUsageStore _store;
    private UsageWindow? _usageWindow;
    private SettingsWindow? _settingsWindow;

    public ApplicationViewModel(
        UsageViewModel usageViewModel,
        SettingsViewModel settingsViewModel,
        UsageRefreshBackgroundService refreshService,
        IUsageStore store)
    {
        _usageViewModel = usageViewModel;
        _settingsViewModel = settingsViewModel;
        _refreshService = refreshService;
        _store = store;
        _store.Changed += (_, _) => UpdateTrayState();
    }

    public ICommand ToggleUsageWindowCommand => new RelayCommand(ToggleUsageWindow);
    public ICommand RefreshCommand => new AsyncRelayCommand(RefreshAsync);
    public ICommand OpenSettingsCommand => new RelayCommand(OpenSettings);
    public ICommand ExitCommand => new RelayCommand(Exit);

    [ObservableProperty]
    private string _trayToolTip = "AI Usage";

    private void ToggleUsageWindow()
    {
        if (_usageWindow is { IsVisible: true })
        {
            _usageWindow.Hide();
            return;
        }

        ShowUsageWindow();
    }

    public void ShowUsageWindow()
    {
        if (!Dispatcher.UIThread.CheckAccess())
        {
            Dispatcher.UIThread.Post(ShowUsageWindow);
            return;
        }

        _usageWindow ??= new UsageWindow { DataContext = _usageViewModel };
        _usageWindow.PrepareToShow();
        _usageWindow.Show();
        WindowPlacement.PositionNearCursor(_usageWindow);
        _usageWindow.Activate();
        _usageWindow.EnableHideOnDeactivate();
    }

    private void OpenSettings()
    {
        _settingsWindow ??= new SettingsWindow { DataContext = _settingsViewModel };
        _settingsWindow.Show();
        _settingsWindow.Activate();
    }

    private async Task RefreshAsync()
    {
        await _refreshService.RefreshOnceAsync(CancellationToken.None);
        UpdateTrayState();
    }

    private void Exit()
    {
        if (Application.Current?.ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
            desktop.Shutdown();
    }

    private void UpdateTrayState()
    {
        var maxUsed = YasbExportMapper.ToDto(_store.Current).MaxUsed;
        TrayToolTip = maxUsed >= 95
            ? $"AI Usage — critical ({maxUsed:0}% used)"
            : maxUsed >= 80
                ? $"AI Usage — warning ({maxUsed:0}% used)"
                : "AI Usage";
    }

}
