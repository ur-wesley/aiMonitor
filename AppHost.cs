using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;
using Microsoft.Extensions.Options;
using aiMonitor.Configuration;
using aiMonitor.Hosting;
using aiMonitor.Platform;
using aiMonitor.Services;
using aiMonitor.Services.Auth;
using aiMonitor.Services.Providers;
using aiMonitor.ViewModels;

namespace aiMonitor;

public static class AppHost
{
    public static IHost Build(string[] args)
    {
        var settingsStore = new SettingsStore();
        var settings = settingsStore.Load();

        var builder = Host.CreateApplicationBuilder(args);
        builder.Services.AddSingleton(settingsStore);
        builder.Services.AddSingleton(settings);
        builder.Services.AddSingleton<IOptions<AppSettings>>(sp => new OptionsWrapper<AppSettings>(sp.GetRequiredService<AppSettings>()));
        builder.Services.AddSingleton<IOptionsMonitor<AppSettings>>(sp => new StaticOptionsMonitor(sp.GetRequiredService<AppSettings>()));

        builder.Services.AddHttpClient("cursor");
        builder.Services.AddHttpClient("opencode");
        builder.Services.AddHttpClient("antigravity");

        builder.Services.AddSingleton<OAuthTokenRefresher>();
        builder.Services.AddSingleton<IUsageStore, UsageStore>();
        builder.Services.AddSingleton<IUsageProvider, CursorProvider>();
        builder.Services.AddSingleton<IUsageProvider, OpenCodeGoProvider>();
        builder.Services.AddSingleton<IUsageProvider, AntigravityProvider>();
        builder.Services.AddSingleton<UsageRefreshBackgroundService>();
        builder.Services.AddHostedService(sp => sp.GetRequiredService<UsageRefreshBackgroundService>());
        builder.Services.AddHostedService<LocalApiHostedService>();
        builder.Services.AddSingleton<WindowsToast>();
        builder.Services.AddHostedService<LowUsageNotificationService>();

        builder.Services.AddSingleton<ApplicationViewModel>();
        builder.Services.AddSingleton<UsageViewModel>();
        builder.Services.AddSingleton<SettingsViewModel>();

        return builder.Build();
    }

    private sealed class StaticOptionsMonitor(AppSettings settings) : IOptionsMonitor<AppSettings>
    {
        public AppSettings CurrentValue => settings;

        public AppSettings Get(string? name) => settings;

        public IDisposable? OnChange(Action<AppSettings, string?> listener) => null;
    }
}
