using System.Text.Json.Serialization;

namespace UsageTray.Models.ExternalApi;

public sealed class AntigravityQuotaResponse
{
    [JsonPropertyName("quotaGroups")]
    public List<AntigravityQuotaGroup>? QuotaGroups { get; set; }

    [JsonPropertyName("quota_groups")]
    public List<AntigravityQuotaGroup>? QuotaGroupsSnake { get; set; }

    public IEnumerable<AntigravityQuotaGroup> Groups =>
        QuotaGroups ?? QuotaGroupsSnake ?? [];
}

public sealed class AntigravityQuotaGroup
{
    [JsonPropertyName("displayName")]
    public string? DisplayName { get; set; }

    [JsonPropertyName("display_name")]
    public string? DisplayNameSnake { get; set; }

    [JsonPropertyName("quotaBuckets")]
    public List<AntigravityQuotaBucket>? QuotaBuckets { get; set; }

    [JsonPropertyName("quota_buckets")]
    public List<AntigravityQuotaBucket>? QuotaBucketsSnake { get; set; }

    public string Name => DisplayName ?? DisplayNameSnake ?? "Unknown";

    public IEnumerable<AntigravityQuotaBucket> Buckets =>
        QuotaBuckets ?? QuotaBucketsSnake ?? [];
}

public sealed class AntigravityQuotaBucket
{
    [JsonPropertyName("remainingFraction")]
    public double? RemainingFraction { get; set; }

    [JsonPropertyName("remaining_fraction")]
    public double? RemainingFractionSnake { get; set; }

    [JsonPropertyName("resetTime")]
    public string? ResetTime { get; set; }

    [JsonPropertyName("reset_time")]
    public string? ResetTimeSnake { get; set; }

    [JsonPropertyName("modelId")]
    public string? ModelId { get; set; }

    [JsonPropertyName("model_id")]
    public string? ModelIdSnake { get; set; }

    public double? Fraction => RemainingFraction ?? RemainingFractionSnake;

    public string? Reset => ResetTime ?? ResetTimeSnake;
}

public sealed class OAuthTokenResponse
{
    [JsonPropertyName("access_token")]
    public string? AccessToken { get; set; }

    [JsonPropertyName("expires_in")]
    public int? ExpiresIn { get; set; }

    [JsonPropertyName("refresh_token")]
    public string? RefreshToken { get; set; }
}

public sealed class StoredOAuthToken
{
    [JsonPropertyName("access_token")]
    public string? AccessToken { get; set; }

    [JsonPropertyName("refresh_token")]
    public string? RefreshToken { get; set; }

    [JsonPropertyName("expiry")]
    public string? Expiry { get; set; }

    [JsonPropertyName("expires_at")]
    public long? ExpiresAt { get; set; }
}
