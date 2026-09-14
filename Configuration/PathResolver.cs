namespace aiMonitor.Configuration;

public static class PathResolver
{
    public static string CursorStateDb(AppSettings settings)
    {
        if (!string.IsNullOrWhiteSpace(settings.CursorStateDbPath))
            return Environment.ExpandEnvironmentVariables(settings.CursorStateDbPath);

        var appData = Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData);
        return Path.Combine(appData, "Cursor", "User", "globalStorage", "state.vscdb");
    }

    public static IEnumerable<string> OpenCodeAuthPaths()
    {
        var profile = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);
        yield return Path.Combine(profile, ".local", "share", "opencode", "auth.json");

        var localAppData = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);
        yield return Path.Combine(localAppData, "opencode", "auth.json");
    }

    public static string AntigravityOAuthTokenFile()
    {
        var profile = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);
        return Path.Combine(profile, ".gemini", "antigravity-cli", "antigravity-oauth-token");
    }

    public static IEnumerable<string> AntigravityStateDbCandidates(AppSettings settings)
    {
        if (!string.IsNullOrWhiteSpace(settings.AntigravityStateDbPath))
        {
            yield return Environment.ExpandEnvironmentVariables(settings.AntigravityStateDbPath);
            yield break;
        }

        var appData = Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData);
        foreach (var folder in new[] { "Antigravity IDE", "Antigravity", "antigravity" })
        {
            var path = Path.Combine(appData, folder, "User", "globalStorage", "state.vscdb");
            if (File.Exists(path))
                yield return path;
        }

        foreach (var folder in new[] { "Antigravity IDE", "Antigravity" })
        {
            yield return Path.Combine(appData, folder, "User", "globalStorage", "state.vscdb");
        }
    }

    public static string SettingsFilePath()
    {
        var appData = Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData);
        return Path.Combine(appData, "aiMonitor", "settings.json");
    }
}
