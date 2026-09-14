using System.Text.Json;
using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Hosting;
using Microsoft.AspNetCore.Http;
using Microsoft.Extensions.Hosting;
using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Options;
using UsageTray.Configuration;
using UsageTray.Models;
using UsageTray.Services;

namespace UsageTray.Hosting;

public sealed class LocalApiHostedService(
    IUsageStore store,
    IOptionsMonitor<AppSettings> settings,
    ILogger<LocalApiHostedService> logger) : IHostedService
{
    private WebApplication? _app;
    private Task? _runTask;

    public Task StartAsync(CancellationToken cancellationToken)
    {
        if (!settings.CurrentValue.LocalApiEnabled)
            return Task.CompletedTask;

        var port = settings.CurrentValue.LocalApiPort;
        var builder = WebApplication.CreateSlimBuilder();
        builder.WebHost.UseUrls($"http://127.0.0.1:{port}");

        _app = builder.Build();
        MapRoutes(_app);

        _runTask = _app.RunAsync(cancellationToken);
        logger.LogInformation("Local API listening on http://127.0.0.1:{Port}", port);
        return Task.CompletedTask;
    }

    public async Task StopAsync(CancellationToken cancellationToken)
    {
        if (_app is null)
            return;

        await _app.StopAsync(cancellationToken).ConfigureAwait(false);
        if (_runTask is not null)
            await _runTask.ConfigureAwait(false);
    }

    private void MapRoutes(WebApplication app)
    {
        app.MapGet("/api/v1/usage", () => Results.Json(YasbExportMapper.ToDto(store.Current)));

        app.MapGet("/api/v1/usage/{providerId}", (string providerId) =>
        {
            var provider = store.Current.Providers.FirstOrDefault(p =>
                string.Equals(p.ProviderId, providerId, StringComparison.OrdinalIgnoreCase));

            if (provider is null)
                return Results.NotFound();

            return Results.Json(provider);
        });

        app.MapGet("/health", () => Results.Ok(new { status = "ok" }));
    }
}
