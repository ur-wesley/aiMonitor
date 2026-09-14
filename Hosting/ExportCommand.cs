using System.Text.Json;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;
using Microsoft.Extensions.Options;
using UsageTray.Configuration;
using UsageTray.Services;
using UsageTray.Services.Auth;
using UsageTray.Services.Providers;

namespace UsageTray.Hosting;

public static class ExportCommand
{
    public static async Task<int> RunAsync(string[] args)
    {
        var builder = Host.CreateApplicationBuilder(args);
        ConfigureServices(builder.Services);

        using var host = builder.Build();
        var refresh = host.Services.GetRequiredService<UsageRefreshBackgroundService>();
        await refresh.RefreshOnceAsync(CancellationToken.None).ConfigureAwait(false);

        var store = host.Services.GetRequiredService<IUsageStore>();
        var dto = YasbExportMapper.ToDto(store.Current);
        var json = JsonSerializer.Serialize(dto, new JsonSerializerOptions { WriteIndented = false });
        Console.WriteLine(json);
        return 0;
    }

    private static void ConfigureServices(IServiceCollection services)
    {
        var settingsStore = new SettingsStore();
        var settings = settingsStore.Load();
        services.AddSingleton(settingsStore);
        services.AddSingleton<IOptions<AppSettings>>(new OptionsWrapper<AppSettings>(settings));
        services.AddSingleton<IOptionsMonitor<AppSettings>>(new StaticOptionsMonitor(settings));

        services.AddHttpClient("cursor");
        services.AddHttpClient("opencode");
        services.AddHttpClient("antigravity");

        services.AddSingleton<OAuthTokenRefresher>();
        services.AddSingleton<IUsageStore, UsageStore>();
        services.AddSingleton<IUsageProvider, CursorProvider>();
        services.AddSingleton<IUsageProvider, OpenCodeGoProvider>();
        services.AddSingleton<IUsageProvider, AntigravityProvider>();
        services.AddSingleton<UsageRefreshBackgroundService>();
    }

    private sealed class StaticOptionsMonitor(AppSettings settings) : IOptionsMonitor<AppSettings>
    {
        public AppSettings CurrentValue => settings;

        public AppSettings Get(string? name) => settings;

        public IDisposable? OnChange(Action<AppSettings, string?> listener) => null;
    }
}
