using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;
using Microsoft.Extensions.Options;
using UsageTray.Configuration;
using UsageTray.Hosting;
using UsageTray.Services;
using UsageTray.Services.Auth;
using UsageTray.Services.Providers;
using UsageTray.ViewModels;

namespace UsageTray;

public static class AppHost
{
    public static IHost Build(string[] args)
    {
        var settingsStore = new SettingsStore();
        var settings = settingsStore.Load();

        var builder = Host.CreateApplicationBuilder(args);
        builder.Services.AddSingleton(settingsStore);
        builder.Services.AddSingleton<IOptions<AppSettings>>(new OptionsWrapper<AppSettings>(settings));
        builder.Services.AddSingleton<IOptionsMonitor<AppSettings>>(new StaticOptionsMonitor(settings));

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
