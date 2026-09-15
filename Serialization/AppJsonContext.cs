using System.Text.Json.Serialization;
using aiMonitor.Models;
using aiMonitor.Models.ExternalApi;

namespace aiMonitor.Serialization;

[JsonSourceGenerationOptions(WriteIndented = false)]
[JsonSerializable(typeof(StoredSettingsDto))]
[JsonSerializable(typeof(YasbExportDto))]
[JsonSerializable(typeof(ProviderUsage))]
[JsonSerializable(typeof(UsageWindowMetric))]
[JsonSerializable(typeof(List<UsageWindowMetric>))]
[JsonSerializable(typeof(CursorUsageSummaryResponse))]
[JsonSerializable(typeof(OpenCodeGoUsageResponse))]
[JsonSerializable(typeof(AntigravityQuotaResponse))]
[JsonSerializable(typeof(LoadCodeAssistResponse))]
[JsonSerializable(typeof(LoadCodeAssistRequest))]
[JsonSerializable(typeof(StoredOAuthToken))]
[JsonSerializable(typeof(AntigravityAuthStatus))]
[JsonSerializable(typeof(OAuthTokenResponse))]
[JsonSerializable(typeof(Dictionary<string, string>))]
[JsonSerializable(typeof(HealthResponse))]
internal partial class AppJsonContext : JsonSerializerContext;

public sealed class StoredSettingsDto
{
    public int RefreshIntervalSeconds { get; set; } = 300;
    public bool LocalApiEnabled { get; set; } = true;
    public int LocalApiPort { get; set; } = 6736;
    public string? EncryptedOpenCodeGoApiKey { get; set; }
    public string? CursorStateDbPath { get; set; }
    public string? AntigravityStateDbPath { get; set; }

    public bool StartWithWindows { get; set; }

    public bool LowUsageNotificationsEnabled { get; set; } = true;
}

public sealed record HealthResponse(string Status);
