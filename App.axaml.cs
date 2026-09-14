using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Controls.Platform;
using Avalonia.Markup.Xaml;
using Avalonia.Platform;
using Microsoft.Extensions.DependencyInjection;
using aiMonitor.ViewModels;

namespace aiMonitor;

public partial class App : Application
{
    private readonly IServiceProvider _services;

    public App(IServiceProvider services)
    {
        _services = services;
    }

    public override void Initialize()
    {
        AvaloniaXamlLoader.Load(this);
    }

    public override void OnFrameworkInitializationCompleted()
    {
        if (ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
        {
            desktop.ShutdownMode = ShutdownMode.OnExplicitShutdown;

            var appVm = _services.GetRequiredService<ApplicationViewModel>();
            DataContext = appVm;

            desktop.MainWindow = new Window
            {
                ShowInTaskbar = false,
                Width = 0,
                Height = 0,
                Opacity = 0,
                DataContext = appVm,
            };

            SetupTrayIcon(appVm);
        }

        base.OnFrameworkInitializationCompleted();
    }

    private void SetupTrayIcon(ApplicationViewModel appVm)
    {
        var icon = new TrayIcon
        {
            Icon = LoadIcon(),
            ToolTipText = appVm.TrayToolTip,
            Command = appVm.ToggleUsageWindowCommand,
            Menu =
            [
                new NativeMenuItem("Refresh") { Command = appVm.RefreshCommand },
                new NativeMenuItem("Settings") { Command = appVm.OpenSettingsCommand },
                new NativeMenuItemSeparator(),
                new NativeMenuItem("Quit") { Command = appVm.ExitCommand },
            ],
        };

        TrayIcon.SetIcons(this, [icon]);
        appVm.PropertyChanged += (_, e) =>
        {
            if (e.PropertyName == nameof(ApplicationViewModel.TrayToolTip))
                icon.ToolTipText = appVm.TrayToolTip;
        };
    }

    private static WindowIcon LoadIcon()
    {
        var uri = new Uri("avares://aiMonitor/Assets/app.ico");
        using var stream = AssetLoader.Open(uri);
        return new WindowIcon(stream);
    }
}
