using System.Windows.Input;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.ApplicationLifetimes;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using UsageTray.Services;
using UsageTray.Views;

namespace UsageTray.ViewModels;

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

        _usageWindow ??= new UsageWindow { DataContext = _usageViewModel };
        PositionNearTray(_usageWindow);
        _usageWindow.Show();
        _usageWindow.Activate();
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

    private static void PositionNearTray(Window window)
    {
        if (Application.Current?.ApplicationLifetime is not IClassicDesktopStyleApplicationLifetime desktop)
            return;

        var screen = desktop.MainWindow?.Screens.Primary ?? window.Screens.Primary;
        if (screen is null)
            return;

        var workArea = screen.WorkingArea;
        window.Position = new PixelPoint(
            workArea.X + workArea.Width - (int)window.Width - 8,
            workArea.Y + workArea.Height - (int)window.Height - 48);
    }
}
