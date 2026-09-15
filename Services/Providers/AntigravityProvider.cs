using System.Net.Http.Headers;
using System.Net.Http.Json;
using System.Text.Json;
using aiMonitor.Configuration;
using aiMonitor.Models;
using aiMonitor.Models.ExternalApi;
using aiMonitor.Serialization;
using aiMonitor.Services.Auth;

namespace aiMonitor.Services.Providers;

public sealed class AntigravityProvider(
    IHttpClientFactory httpClientFactory,
    OAuthTokenRefresher tokenRefresher) : IUsageProvider
{
    private const string UserAgent = "antigravity";

    private static readonly string[] BaseUrls =
    [
        "https://daily-cloudcode-pa.googleapis.com",
        "https://daily-cloudcode-pa.sandbox.googleapis.com",
        "https://cloudcode-pa.googleapis.com",
    ];

    public string ProviderId => "antigravity";

    public string DisplayName => "Antigravity";

    public async Task<ProviderUsage> FetchAsync(CancellationToken ct)
    {
        var token = await tokenRefresher.ResolveAccessTokenAsync(ct).ConfigureAwait(false);
        if (string.IsNullOrWhiteSpace(token))
            return Failed("Not signed in to Antigravity");

        var client = httpClientFactory.CreateClient("antigravity");
        var projectId = await LoadProjectIdAsync(client, token, ct).ConfigureAwait(false);

        AntigravityQuotaResponse? body = null;
        foreach (var baseUrl in BaseUrls)
        {
            using var request = CreateQuotaRequest(baseUrl, token, projectId);
            using var response = await client.SendAsync(request, ct).ConfigureAwait(false);
            if (!response.IsSuccessStatusCode)
                continue;

            await using var stream = await response.Content.ReadAsStreamAsync(ct).ConfigureAwait(false);
            body = await JsonSerializer.DeserializeAsync(stream, AppJsonContext.Default.AntigravityQuotaResponse, ct)
                .ConfigureAwait(false);

            if (body is not null && body.AllGroups.Any())
                break;
        }

        if (body is null || !body.AllGroups.Any())
            return Failed("Could not fetch Antigravity quota");

        var windows = new List<UsageWindowMetric>();
        foreach (var group in body.AllGroups)
        {
            var isGemini = group.Name.Contains("gemini", StringComparison.OrdinalIgnoreCase);
            var prefix = isGemini ? "Gemini" : "Claude/GPT";
            var buckets = group.AllBuckets.ToList();

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

    private static HttpRequestMessage CreateQuotaRequest(string baseUrl, string token, string? projectId)
    {
        var projectBody = string.IsNullOrWhiteSpace(projectId)
            ? new Dictionary<string, string>()
            : new Dictionary<string, string> { ["project"] = projectId };

        var request = new HttpRequestMessage(HttpMethod.Post, $"{baseUrl}/v1internal:retrieveUserQuotaSummary")
        {
            Content = JsonContent.Create(projectBody, AppJsonContext.Default.DictionaryStringString),
        };
        request.Headers.Authorization = new AuthenticationHeaderValue("Bearer", token);
        request.Headers.TryAddWithoutValidation("User-Agent", UserAgent);
        return request;
    }

    private static async Task<string?> LoadProjectIdAsync(HttpClient client, string token, CancellationToken ct)
    {
        var payload = new LoadCodeAssistRequest
        {
            Metadata = new LoadCodeAssistMetadata { IdeType = "ANTIGRAVITY" },
        };

        foreach (var baseUrl in BaseUrls)
        {
            using var request = new HttpRequestMessage(HttpMethod.Post, $"{baseUrl}/v1internal:loadCodeAssist")
            {
                Content = JsonContent.Create(payload, AppJsonContext.Default.LoadCodeAssistRequest),
            };
            request.Headers.Authorization = new AuthenticationHeaderValue("Bearer", token);
            request.Headers.TryAddWithoutValidation("User-Agent", UserAgent);

            using var response = await client.SendAsync(request, ct).ConfigureAwait(false);
            if (!response.IsSuccessStatusCode)
                continue;

            await using var stream = await response.Content.ReadAsStreamAsync(ct).ConfigureAwait(false);
            var body = await JsonSerializer.DeserializeAsync(stream, AppJsonContext.Default.LoadCodeAssistResponse, ct)
                .ConfigureAwait(false);

            if (!string.IsNullOrWhiteSpace(body?.CloudAiCompanionProject))
                return body.CloudAiCompanionProject;
        }

        return null;
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
