using System.Net.Http.Headers;
using System.Text.Json;
using System.Text.RegularExpressions;
using Microsoft.Extensions.Options;
using aiMonitor.Configuration;
using aiMonitor.Models;
using aiMonitor.Models.ExternalApi;
using aiMonitor.Serialization;
using aiMonitor.Services.Auth;

namespace aiMonitor.Services.Providers;

public sealed partial class CursorProvider(
    IHttpClientFactory httpClientFactory,
    IOptionsMonitor<AppSettings> settings) : IUsageProvider
{
    public string ProviderId => "cursor";

    public string DisplayName => "Cursor";

    public async Task<ProviderUsage> FetchAsync(CancellationToken ct)
    {
        var dbPath = PathResolver.CursorStateDb(settings.CurrentValue);
        var token = await SqliteTokenReader.ReadValueAsync(dbPath, "cursorAuth/accessToken", ct)
            .ConfigureAwait(false);

        if (string.IsNullOrWhiteSpace(token))
        {
            return Failed("Not signed in to Cursor");
        }

        var userId = ExtractUserId(token);
        if (userId is null)
        {
            return Failed("Could not read Cursor session");
        }

        var client = httpClientFactory.CreateClient("cursor");
        using var request = new HttpRequestMessage(HttpMethod.Get, "https://cursor.com/api/usage-summary");
        request.Headers.Add("Cookie", $"WorkosCursorSessionToken={userId}%3A%3A{token}");

        using var response = await client.SendAsync(request, ct).ConfigureAwait(false);
        if (response.StatusCode == System.Net.HttpStatusCode.Unauthorized)
            return Failed("Cursor session expired");

        if (!response.IsSuccessStatusCode)
            return Failed($"Cursor API error ({(int)response.StatusCode})");

        await using var stream = await response.Content.ReadAsStreamAsync(ct).ConfigureAwait(false);
        var body = await JsonSerializer.DeserializeAsync(stream, AppJsonContext.Default.CursorUsageSummaryResponse, ct)
            .ConfigureAwait(false);

        if (body is null)
            return Failed("Empty Cursor response");

        var plan = body.IndividualUsage?.Plan;
        double? total = plan?.TotalPercentUsed;
        double? auto = plan?.AutoPercentUsed;
        double? api = plan?.ApiPercentUsed;

        if (total is null)
        {
            auto = ParsePercent(body.AutoModelSelectedDisplayMessage);
            api = ParsePercent(body.NamedModelSelectedDisplayMessage);
            total = auto is not null || api is not null ? Math.Max(auto ?? 0, api ?? 0) : null;
        }

        DateTimeOffset? reset = null;
        if (DateTimeOffset.TryParse(body.BillingCycleEnd, out var resetAt))
            reset = resetAt;

        var windows = new List<UsageWindowMetric>();
        if (total is not null)
            windows.Add(new UsageWindowMetric("Total", total.Value, false, reset));
        if (auto is not null)
            windows.Add(new UsageWindowMetric("Auto", auto.Value, false, reset));
        if (api is not null)
            windows.Add(new UsageWindowMetric("API", api.Value, false, reset));

        if (windows.Count == 0)
            return Failed("No usage data in Cursor response");

        return new ProviderUsage(ProviderId, DisplayName, windows, null, DateTimeOffset.UtcNow);
    }

    private ProviderUsage Failed(string error) =>
        new(ProviderId, DisplayName, [], error, DateTimeOffset.UtcNow);

    private static string? ExtractUserId(string jwt)
    {
        var parts = jwt.Split('.');
        if (parts.Length < 2)
            return null;

        try
        {
            var payload = parts[1];
            payload = payload.PadRight(payload.Length + (4 - payload.Length % 4) % 4, '=');
            var json = System.Text.Encoding.UTF8.GetString(Convert.FromBase64String(payload.Replace('-', '+').Replace('_', '/')));
            using var doc = JsonDocument.Parse(json);
            if (doc.RootElement.TryGetProperty("sub", out var sub))
                return sub.GetString();
        }
        catch
        {
            return null;
        }

        return null;
    }

    private static double? ParsePercent(string? message)
    {
        if (string.IsNullOrWhiteSpace(message))
            return null;

        var match = PercentRegex().Match(message);
        return match.Success && double.TryParse(match.Groups[1].Value, out var value) ? value : null;
    }

    [GeneratedRegex(@"(\d+(?:\.\d+)?)\s*%")]
    private static partial Regex PercentRegex();
}
