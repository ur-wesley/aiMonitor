using System.Net.Http.Headers;
using System.Net.Http.Json;
using System.Text.Json;
using aiMonitor.Models;
using aiMonitor.Models.ExternalApi;
using aiMonitor.Services.Auth;

namespace aiMonitor.Services.Providers;

public sealed class AntigravityProvider(
    IHttpClientFactory httpClientFactory,
    OAuthTokenRefresher tokenRefresher) : IUsageProvider
{
    private static readonly string[] QuotaUrls =
    [
        "https://cloudcode-pa.googleapis.com/v1internal:retrieveUserQuotaSummary",
        "https://daily-cloudcode-pa.googleapis.com/v1internal:retrieveUserQuotaSummary",
    ];

    public string ProviderId => "antigravity";

    public string DisplayName => "Antigravity";

    public async Task<ProviderUsage> FetchAsync(CancellationToken ct)
    {
        var token = await tokenRefresher.ResolveAccessTokenAsync(ct).ConfigureAwait(false);
        if (string.IsNullOrWhiteSpace(token))
            return Failed("Not signed in to Antigravity");

        var client = httpClientFactory.CreateClient("antigravity");
        AntigravityQuotaResponse? body = null;

        foreach (var url in QuotaUrls)
        {
            using var request = new HttpRequestMessage(HttpMethod.Post, url)
            {
                Content = JsonContent.Create(new { }),
            };
            request.Headers.Authorization = new AuthenticationHeaderValue("Bearer", token);

            using var response = await client.SendAsync(request, ct).ConfigureAwait(false);
            if (!response.IsSuccessStatusCode)
                continue;

            await using var stream = await response.Content.ReadAsStreamAsync(ct).ConfigureAwait(false);
            body = await JsonSerializer.DeserializeAsync<AntigravityQuotaResponse>(stream, cancellationToken: ct)
                .ConfigureAwait(false);

            if (body is not null && body.Groups.Any())
                break;
        }

        if (body is null || !body.Groups.Any())
            return Failed("Could not fetch Antigravity quota");

        var windows = new List<UsageWindowMetric>();
        foreach (var group in body.Groups)
        {
            var isGemini = group.Name.Contains("gemini", StringComparison.OrdinalIgnoreCase);
            var prefix = isGemini ? "Gemini" : "Claude/GPT";
            var buckets = group.Buckets.ToList();

            if (buckets.Count >= 2)
            {
                AddRemaining(windows, $"{prefix} 5h", buckets[0]);
                AddRemaining(windows, $"{prefix} Weekly", buckets[1]);
            }
            else if (buckets.Count == 1)
            {
                AddRemaining(windows, $"{prefix} 5h", buckets[0]);
            }
        }

        if (windows.Count == 0)
            return Failed("No quota buckets in Antigravity response");

        return new ProviderUsage(ProviderId, DisplayName, windows, null, DateTimeOffset.UtcNow);
    }

    private static void AddRemaining(List<UsageWindowMetric> windows, string label, AntigravityQuotaBucket bucket)
    {
        if (bucket.Fraction is not double fraction)
            return;

        var remaining = Math.Round(fraction * 100, 1);
        DateTimeOffset? reset = DateTimeOffset.TryParse(bucket.Reset, out var r) ? r : null;
        windows.Add(new UsageWindowMetric(label, remaining, true, reset));
    }

    private ProviderUsage Failed(string error) =>
        new(ProviderId, DisplayName, [], error, DateTimeOffset.UtcNow);
}
