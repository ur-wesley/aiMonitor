using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Interactivity;
using UsageTray.ViewModels;

namespace UsageTray.Views;

public partial class UsageWindow : Window
{
    public UsageWindow()
    {
        InitializeComponent();
        Deactivated += (_, _) => Hide();
        SettingsButton.Click += OnSettingsClick;
        QuitButton.Click += OnQuitClick;
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
