using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Interactivity;
using Avalonia.Threading;
using aiMonitor.ViewModels;

namespace aiMonitor.Views;

public partial class UsageWindow : Window
{
    private bool _hideOnDeactivate;

    public UsageWindow()
    {
        InitializeComponent();
        Deactivated += OnDeactivated;
        SettingsButton.Click += OnSettingsClick;
        QuitButton.Click += OnQuitClick;
    }

    public void PrepareToShow() => _hideOnDeactivate = false;

    public void EnableHideOnDeactivate()
    {
        Dispatcher.UIThread.Post(() => _hideOnDeactivate = true, DispatcherPriority.Background);
    }

    private void OnDeactivated(object? sender, EventArgs e)
    {
        if (!_hideOnDeactivate)
            return;

        Dispatcher.UIThread.Post(() =>
        {
            if (_hideOnDeactivate && IsVisible)
                Hide();
        }, DispatcherPriority.Background);
    }

    private void OnSettingsClick(object? sender, RoutedEventArgs e)
    {
        if (Application.Current?.ApplicationLifetime is not IClassicDesktopStyleApplicationLifetime desktop)
            return;

        var appVm = desktop.MainWindow?.DataContext as ApplicationViewModel;
        appVm?.OpenSettingsCommand.Execute(null);
    }

    private void OnQuitClick(object? sender, RoutedEventArgs e)
    {
        if (Application.Current?.ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
            desktop.Shutdown();
    }
}
