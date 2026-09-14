namespace UsageTray.Configuration;

public sealed class AppSettings
{
    public int RefreshIntervalSeconds { get; set; } = 300;

    public bool LocalApiEnabled { get; set; } = true;

    public int LocalApiPort { get; set; } = 6736;

    public string? OpenCodeGoApiKey { get; set; }

    public string? CursorStateDbPath { get; set; }

    public string? AntigravityStateDbPath { get; set; }
}
