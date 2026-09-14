using System.Net.Http.Json;
using System.Text.Json;
using aiMonitor.Models.ExternalApi;

namespace aiMonitor.Services.Auth;

public sealed class OAuthTokenRefresher(IHttpClientFactory httpClientFactory)
{
    private const string ClientId = "1071006060591-tmhssin2h21lcre235vtolojh4g403ep.apps.googleusercontent.com";
    private const string ClientSecret = "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";
    private const string TokenUrl = "https://oauth2.googleapis.com/token";

    public async Task<string?> ResolveAccessTokenAsync(CancellationToken ct)
    {
        var tokenJson = WindowsCredentialReader.ReadAntigravityToken();
        if (string.IsNullOrWhiteSpace(tokenJson))
            tokenJson = await ReadTokenFileAsync(ct).ConfigureAwait(false);

        if (string.IsNullOrWhiteSpace(tokenJson))
            return null;

        StoredOAuthToken? stored;
        try
        {
            stored = JsonSerializer.Deserialize<StoredOAuthToken>(tokenJson);
        }
        catch
        {
            return tokenJson.StartsWith("ya29.", StringComparison.Ordinal) ? tokenJson : null;
        }

        if (stored is null)
            return null;

        if (!string.IsNullOrEmpty(stored.AccessToken) && !IsExpired(stored))
            return stored.AccessToken;

        if (string.IsNullOrEmpty(stored.RefreshToken))
            return stored.AccessToken;

        return await RefreshAsync(stored.RefreshToken, ct).ConfigureAwait(false);
    }

    private static bool IsExpired(StoredOAuthToken token)
    {
        if (token.ExpiresAt is long unix && unix > 0)
            return DateTimeOffset.FromUnixTimeSeconds(unix) <= DateTimeOffset.UtcNow.AddMinutes(2);

        if (DateTimeOffset.TryParse(token.Expiry, out var expiry))
            return expiry <= DateTimeOffset.UtcNow.AddMinutes(2);

        return false;
    }

    private static async Task<string?> ReadTokenFileAsync(CancellationToken ct)
    {
        var path = Configuration.PathResolver.AntigravityOAuthTokenFile();
        if (!File.Exists(path))
            return null;

        return await File.ReadAllTextAsync(path, ct).ConfigureAwait(false);
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
