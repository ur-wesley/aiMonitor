using System.Text.Json.Serialization;

namespace aiMonitor.Models.ExternalApi;

public sealed class AntigravityQuotaResponse
{
    [JsonPropertyName("groups")]
    public List<AntigravityQuotaGroup>? Groups { get; set; }

    [JsonPropertyName("quotaGroups")]
    public List<AntigravityQuotaGroup>? QuotaGroups { get; set; }

    [JsonPropertyName("quota_groups")]
    public List<AntigravityQuotaGroup>? QuotaGroupsSnake { get; set; }

    public IEnumerable<AntigravityQuotaGroup> AllGroups =>
        Groups ?? QuotaGroups ?? QuotaGroupsSnake ?? [];
}

public sealed class LoadCodeAssistResponse
{
    [JsonPropertyName("cloudaicompanionProject")]
    public string? CloudAiCompanionProject { get; set; }
}

public sealed class LoadCodeAssistRequest
{
    [JsonPropertyName("metadata")]
    public LoadCodeAssistMetadata Metadata { get; set; } = new();
}

public sealed class LoadCodeAssistMetadata
{
    [JsonPropertyName("ideType")]
    public string IdeType { get; set; } = "ANTIGRAVITY";
}

public sealed class AntigravityAuthStatus
{
    [JsonPropertyName("apiKey")]
    public string? ApiKey { get; set; }
}

public sealed class AntigravityQuotaGroup
{
    [JsonPropertyName("displayName")]
    public string? DisplayName { get; set; }

    [JsonPropertyName("display_name")]
    public string? DisplayNameSnake { get; set; }

    [JsonPropertyName("buckets")]
    public List<AntigravityQuotaBucket>? Buckets { get; set; }

    [JsonPropertyName("quotaBuckets")]
    public List<AntigravityQuotaBucket>? QuotaBuckets { get; set; }

    [JsonPropertyName("quota_buckets")]
    public List<AntigravityQuotaBucket>? QuotaBucketsSnake { get; set; }

    public string Name => DisplayName ?? DisplayNameSnake ?? "Unknown";

    public IEnumerable<AntigravityQuotaBucket> AllBuckets =>
        Buckets ?? QuotaBuckets ?? QuotaBucketsSnake ?? [];
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

    [JsonPropertyName("expiry_date")]
    public long? ExpiryDate { get; set; }

    [JsonPropertyName("token")]
    public StoredOAuthToken? Token { get; set; }

    public StoredOAuthToken Normalize()
    {
        if (Token is null)
            return this;

        return new StoredOAuthToken
        {
            AccessToken = Token.AccessToken ?? AccessToken,
            RefreshToken = Token.RefreshToken ?? RefreshToken,
            Expiry = Token.Expiry ?? Expiry,
            ExpiresAt = Token.ExpiresAt ?? ExpiresAt,
            ExpiryDate = Token.ExpiryDate ?? ExpiryDate,
        };
    }
}
