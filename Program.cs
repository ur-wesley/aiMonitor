using Avalonia;
using Microsoft.Extensions.DependencyInjection;
using aiMonitor.Hosting;
using aiMonitor.Platform;
using aiMonitor.ViewModels;

namespace aiMonitor;

internal static class Program
{
    [STAThread]
    public static async Task<int> Main(string[] args)
    {
        if (args.Length > 0 && string.Equals(args[0], "export", StringComparison.OrdinalIgnoreCase))
            return await ExportCommand.RunAsync(args).ConfigureAwait(false);

        var host = AppHost.Build(args);
        await host.StartAsync().ConfigureAwait(false);

        var appVm = host.Services.GetRequiredService<ApplicationViewModel>();
        var toast = host.Services.GetRequiredService<WindowsToast>();
        toast.Initialize(appVm.ShowUsageWindow);

        try
        {
            BuildAvaloniaApp(host.Services).StartWithClassicDesktopLifetime(args);
        }
        finally
        {
            await host.StopAsync().ConfigureAwait(false);
        }

        return 0;
    }

    public static AppBuilder BuildAvaloniaApp(IServiceProvider services) =>
        AppBuilder.Configure(() => new App(services))
            .UsePlatformDetect()
#if DEBUG
            .WithDeveloperTools()
#endif
            .WithInterFont()
            .LogToTrace();
}
