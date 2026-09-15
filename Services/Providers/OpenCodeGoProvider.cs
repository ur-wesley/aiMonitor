using System.Net.Http.Headers;
using System.Text.Json;
using System.Text.Json.Nodes;
using Microsoft.Extensions.Options;
using aiMonitor.Configuration;
using aiMonitor.Models;
using aiMonitor.Models.ExternalApi;
using aiMonitor.Serialization;

namespace aiMonitor.Services.Providers;

public sealed class OpenCodeGoProvider(
    IHttpClientFactory httpClientFactory,
    IOptionsMonitor<AppSettings> settings) : IUsageProvider
{
    public string ProviderId => "opencode_go";

    public string DisplayName => "OpenCode Go";

    public async Task<ProviderUsage> FetchAsync(CancellationToken ct)
    {
        var apiKey = settings.CurrentValue.OpenCodeGoApiKey ?? await ReadApiKeyAsync(ct).ConfigureAwait(false);
        if (string.IsNullOrWhiteSpace(apiKey))
            return Failed("No OpenCode Go API key found");

        var client = httpClientFactory.CreateClient("opencode");
        using var request = new HttpRequestMessage(HttpMethod.Get, "https://opencode.ai/zen/go/v1/usage");
        request.Headers.Authorization = new AuthenticationHeaderValue("Bearer", apiKey);

        using var response = await client.SendAsync(request, ct).ConfigureAwait(false);
        if (response.StatusCode == System.Net.HttpStatusCode.Forbidden)
            return Failed("No Go subscription on this key");

        if (!response.IsSuccessStatusCode)
            return Failed($"OpenCode API error ({(int)response.StatusCode})");

        await using var stream = await response.Content.ReadAsStreamAsync(ct).ConfigureAwait(false);
        var body = await JsonSerializer.DeserializeAsync(stream, AppJsonContext.Default.OpenCodeGoUsageResponse, ct)
            .ConfigureAwait(false);

        if (body?.Usage is null)
            return Failed("Empty OpenCode response");

        var windows = new List<UsageWindowMetric>();
        AddWindow(windows, "5h", body.Usage.Rolling);
        AddWindow(windows, "Weekly", body.Usage.Weekly);
        AddWindow(windows, "Monthly", body.Usage.Monthly);

        if (windows.Count == 0)
            return Failed("No quota windows in OpenCode response");

        return new ProviderUsage(ProviderId, DisplayName, windows, null, DateTimeOffset.UtcNow);
    }

    private static void AddWindow(List<UsageWindowMetric> windows, string label, OpenCodeGoUsageWindow? bucket)
    {
        if (bucket?.Percent is not double percent)
            return;

        DateTimeOffset? reset = DateTimeOffset.TryParse(bucket.ResetsAt, out var r) ? r : null;
        windows.Add(new UsageWindowMetric(label, percent, false, reset));
    }

    private static async Task<string?> ReadApiKeyAsync(CancellationToken ct)
    {
        foreach (var path in PathResolver.OpenCodeAuthPaths())
        {
            if (!File.Exists(path))
                continue;

            try
            {
                var json = await File.ReadAllTextAsync(path, ct).ConfigureAwait(false);
                var node = JsonNode.Parse(json);
                if (node is null)
                    continue;

                foreach (var key in new[] { "opencode-go", "opencode_go", "OpenCode Go", "go" })
                {
                    var apiKey = ExtractApiKey(node[key]);
                    if (!string.IsNullOrWhiteSpace(apiKey))
                        return apiKey;
                }

                var skKey = FindSkKey(node);
                if (!string.IsNullOrWhiteSpace(skKey))
                    return skKey;
            }
            catch
            {
                // try next path
            }
        }

        return null;
    }

    private static string? ExtractApiKey(JsonNode? node)
    {
        if (node is null)
            return null;

        if (node is JsonValue value)
        {
            var text = value.GetValue<string>();
            return string.IsNullOrWhiteSpace(text) ? null : text;
        }

        if (node is not JsonObject obj)
            return null;

        foreach (var propertyName in new[] { "key", "apiKey", "api_key" })
        {
            var direct = obj[propertyName]?.GetValue<string>();
            if (!string.IsNullOrWhiteSpace(direct))
                return direct;
        }

        foreach (var property in obj)
        {
            var nested = ExtractApiKey(property.Value);
            if (!string.IsNullOrWhiteSpace(nested))
                return nested;
        }

        return null;
    }

    private static string? FindSkKey(JsonNode? node)
    {
        if (node is null)
            return null;

        if (node is JsonValue value)
        {
            var text = value.GetValue<string>();
            if (!string.IsNullOrWhiteSpace(text) && text.StartsWith("sk-", StringComparison.Ordinal))
                return text;

            return null;
        }

        if (node is not JsonObject obj)
            return null;

        foreach (var property in obj)
        {
            var found = FindSkKey(property.Value);
            if (!string.IsNullOrWhiteSpace(found))
                return found;
        }

        return null;
    }

    private ProviderUsage Failed(string error) =>
        new(ProviderId, DisplayName, [], error, DateTimeOffset.UtcNow);
}
