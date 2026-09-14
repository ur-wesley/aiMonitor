using System.Net.Http.Json;
using System.Text.Json;
using aiMonitor.Configuration;
using aiMonitor.Models.ExternalApi;

namespace aiMonitor.Services.Auth;

public sealed class OAuthTokenRefresher(IHttpClientFactory httpClientFactory)
{
    private const string ClientId = "1071006060591-tmhssin2h21lcre235vtolojh4g403ep.apps.googleusercontent.com";
    private const string ClientSecret = "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";
    private const string TokenUrl = "https://oauth2.googleapis.com/token";

    public async Task<string?> ResolveAccessTokenAsync(CancellationToken ct)
    {
        var credential = WindowsCredentialReader.ReadAntigravityToken();
        if (!string.IsNullOrWhiteSpace(credential))
        {
            var token = await ResolveFromJsonAsync(credential, ct).ConfigureAwait(false);
            if (!string.IsNullOrWhiteSpace(token))
                return token;
        }

        foreach (var path in new[]
        {
            PathResolver.GeminiOAuthCredsFile(),
            PathResolver.AntigravityOAuthTokenFile(),
        })
        {
            if (!File.Exists(path))
                continue;

            var contents = await File.ReadAllTextAsync(path, ct).ConfigureAwait(false);
            if (string.IsNullOrWhiteSpace(contents))
                continue;

            var token = await ResolveFromJsonAsync(contents, ct).ConfigureAwait(false);
            if (!string.IsNullOrWhiteSpace(token))
                return token;
        }

        return await ReadVscdbApiKeyAsync(ct).ConfigureAwait(false);
    }

    private async Task<string?> ResolveFromJsonAsync(string tokenJson, CancellationToken ct)
    {
        var stored = ParseStoredToken(tokenJson);
        if (stored is null)
            return null;

        if (!string.IsNullOrEmpty(stored.AccessToken) && !IsExpired(stored))
            return stored.AccessToken;

        if (string.IsNullOrEmpty(stored.RefreshToken))
            return IsExpired(stored) ? null : stored.AccessToken;

        return await RefreshAsync(stored.RefreshToken, ct).ConfigureAwait(false);
    }

    private static StoredOAuthToken? ParseStoredToken(string tokenJson)
    {
        try
        {
            return JsonSerializer.Deserialize<StoredOAuthToken>(tokenJson)?.Normalize();
        }
        catch
        {
            return tokenJson.StartsWith("ya29.", StringComparison.Ordinal)
                ? new StoredOAuthToken { AccessToken = tokenJson }
                : null;
        }
    }

    private static bool IsExpired(StoredOAuthToken token)
    {
        if (token.ExpiryDate is long expiryMs && expiryMs > 0)
            return DateTimeOffset.FromUnixTimeMilliseconds(expiryMs) <= DateTimeOffset.UtcNow.AddMinutes(2);

        if (token.ExpiresAt is long unix && unix > 0)
            return DateTimeOffset.FromUnixTimeSeconds(unix) <= DateTimeOffset.UtcNow.AddMinutes(2);

        if (DateTimeOffset.TryParse(token.Expiry, out var expiry))
            return expiry <= DateTimeOffset.UtcNow.AddMinutes(2);

        return false;
    }

    private static async Task<string?> ReadVscdbApiKeyAsync(CancellationToken ct)
    {
        foreach (var dbPath in PathResolver.AntigravityStateDbCandidates(new AppSettings()))
        {
            var json = await SqliteTokenReader.ReadValueAsync(dbPath, "antigravityAuthStatus", ct).ConfigureAwait(false);
            if (string.IsNullOrWhiteSpace(json))
                continue;

            var status = JsonSerializer.Deserialize<AntigravityAuthStatus>(json);
            if (!string.IsNullOrWhiteSpace(status?.ApiKey))
                return status.ApiKey;
        }

        return null;
    }

    private async Task<string?> RefreshAsync(string refreshToken, CancellationToken ct)
    {
        var client = httpClientFactory.CreateClient("antigravity");
        using var response = await client.PostAsync(
            TokenUrl,
            new FormUrlEncodedContent(new Dictionary<string, string>
            {
                ["client_id"] = ClientId,
                ["client_secret"] = ClientSecret,
                ["refresh_token"] = refreshToken,
                ["grant_type"] = "refresh_token",
            }),
            ct).ConfigureAwait(false);

        if (!response.IsSuccessStatusCode)
            return null;

        var body = await response.Content.ReadFromJsonAsync<OAuthTokenResponse>(cancellationToken: ct)
            .ConfigureAwait(false);

        return body?.AccessToken;
    }
}
