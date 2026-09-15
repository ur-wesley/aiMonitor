using System.Net;
using System.Text;
using System.Text.Json;
using Microsoft.Extensions.Hosting;
using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Options;
using aiMonitor.Configuration;
using aiMonitor.Serialization;
using aiMonitor.Services;

namespace aiMonitor.Hosting;

public sealed class LocalApiHostedService(
    IUsageStore store,
    IOptionsMonitor<AppSettings> settings,
    ILogger<LocalApiHostedService> logger) : IHostedService
{
    private HttpListener? _listener;
    private CancellationTokenSource? _cts;
    private Task? _loop;

    public Task StartAsync(CancellationToken cancellationToken)
    {
        if (!settings.CurrentValue.LocalApiEnabled)
            return Task.CompletedTask;

        var port = settings.CurrentValue.LocalApiPort;
        var prefix = $"http://127.0.0.1:{port}/";
        _listener = new HttpListener();
        _listener.Prefixes.Add(prefix);
        _listener.Start();

        _cts = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken);
        _loop = Task.Run(() => AcceptLoopAsync(_cts.Token), CancellationToken.None);
        logger.LogInformation("Local API listening on {Prefix}", prefix);
        return Task.CompletedTask;
    }

    public async Task StopAsync(CancellationToken cancellationToken)
    {
        if (_listener is null)
            return;

        _cts?.Cancel();
        _listener.Stop();
        _listener.Close();

        if (_loop is not null)
        {
            try
            {
                await _loop.WaitAsync(cancellationToken).ConfigureAwait(false);
            }
            catch (OperationCanceledException)
            {
            }
        }
    }

    private async Task AcceptLoopAsync(CancellationToken ct)
    {
        while (!ct.IsCancellationRequested && _listener is { IsListening: true })
        {
            HttpListenerContext context;
            try
            {
                context = await _listener.GetContextAsync().WaitAsync(ct).ConfigureAwait(false);
            }
            catch (OperationCanceledException)
            {
                break;
            }
            catch (HttpListenerException)
            {
                break;
            }
            catch (ObjectDisposedException)
            {
                break;
            }

            _ = Task.Run(() => HandleAsync(context), CancellationToken.None);
        }
    }

    private async Task HandleAsync(HttpListenerContext context)
    {
        try
        {
            var request = context.Request;
            var response = context.Response;
            if (!HttpMethods.IsGet(request.HttpMethod))
            {
                response.StatusCode = (int)HttpStatusCode.MethodNotAllowed;
                response.Close();
                return;
            }

            var path = request.Url?.AbsolutePath.TrimEnd('/') ?? string.Empty;
            if (path.Equals("/health", StringComparison.OrdinalIgnoreCase))
            {
                await WriteJsonAsync(response, HttpStatusCode.OK, new HealthResponse("ok"), AppJsonContext.Default.HealthResponse)
                    .ConfigureAwait(false);
                return;
            }

            if (path.Equals("/api/v1/usage", StringComparison.OrdinalIgnoreCase))
            {
                await WriteJsonAsync(
                        response,
                        HttpStatusCode.OK,
                        YasbExportMapper.ToDto(store.Current),
                        AppJsonContext.Default.YasbExportDto)
                    .ConfigureAwait(false);
                return;
            }

            const string usagePrefix = "/api/v1/usage/";
            if (path.StartsWith(usagePrefix, StringComparison.OrdinalIgnoreCase))
            {
                var providerId = path[usagePrefix.Length..];
                var provider = store.Current.Providers.FirstOrDefault(p =>
                    string.Equals(p.ProviderId, providerId, StringComparison.OrdinalIgnoreCase));

                if (provider is null)
                {
                    response.StatusCode = (int)HttpStatusCode.NotFound;
                    response.Close();
                    return;
                }

                await WriteJsonAsync(response, HttpStatusCode.OK, provider, AppJsonContext.Default.ProviderUsage)
                    .ConfigureAwait(false);
                return;
            }

            response.StatusCode = (int)HttpStatusCode.NotFound;
            response.Close();
        }
        catch (Exception ex)
        {
            logger.LogDebug(ex, "Local API request failed");
            try
            {
                context.Response.StatusCode = (int)HttpStatusCode.InternalServerError;
                context.Response.Close();
            }
            catch
            {
            }
        }
    }

    private static async Task WriteJsonAsync<T>(
        HttpListenerResponse response,
        HttpStatusCode status,
        T value,
        System.Text.Json.Serialization.Metadata.JsonTypeInfo<T> typeInfo)
    {
        var bytes = JsonSerializer.SerializeToUtf8Bytes(value, typeInfo);
        response.StatusCode = (int)status;
        response.ContentType = "application/json; charset=utf-8";
        response.ContentEncoding = Encoding.UTF8;
        response.ContentLength64 = bytes.Length;
        await response.OutputStream.WriteAsync(bytes).ConfigureAwait(false);
        response.Close();
    }

    private static class HttpMethods
    {
        public static bool IsGet(string? method) =>
            string.Equals(method, "GET", StringComparison.OrdinalIgnoreCase);
    }
}
