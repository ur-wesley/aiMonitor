using System.Security.Cryptography;
using System.Text;
using System.Text.Json;

namespace aiMonitor.Configuration;

public sealed class SettingsStore
{
    private static readonly JsonSerializerOptions JsonOptions = new() { WriteIndented = true };

    public AppSettings Load()
    {
        var path = PathResolver.SettingsFilePath();
        if (!File.Exists(path))
            return new AppSettings();

        try
        {
            var json = File.ReadAllText(path);
            var stored = JsonSerializer.Deserialize<StoredSettings>(json, JsonOptions);
            if (stored is null)
                return new AppSettings();

            return new AppSettings
            {
                RefreshIntervalSeconds = stored.RefreshIntervalSeconds,
                LocalApiEnabled = stored.LocalApiEnabled,
                LocalApiPort = stored.LocalApiPort,
                OpenCodeGoApiKey = Decrypt(stored.EncryptedOpenCodeGoApiKey),
                CursorStateDbPath = stored.CursorStateDbPath,
                AntigravityStateDbPath = stored.AntigravityStateDbPath,
            };
        }
        catch
        {
            return new AppSettings();
        }
    }

    public void Save(AppSettings settings)
    {
        var path = PathResolver.SettingsFilePath();
        Directory.CreateDirectory(Path.GetDirectoryName(path)!);

        var stored = new StoredSettings
        {
            RefreshIntervalSeconds = settings.RefreshIntervalSeconds,
            LocalApiEnabled = settings.LocalApiEnabled,
            LocalApiPort = settings.LocalApiPort,
            EncryptedOpenCodeGoApiKey = Encrypt(settings.OpenCodeGoApiKey),
            CursorStateDbPath = settings.CursorStateDbPath,
            AntigravityStateDbPath = settings.AntigravityStateDbPath,
        };

        File.WriteAllText(path, JsonSerializer.Serialize(stored, JsonOptions));
    }

    private static string? Encrypt(string? plain)
    {
        if (string.IsNullOrEmpty(plain))
            return null;

        if (!OperatingSystem.IsWindows())
            return Convert.ToBase64String(Encoding.UTF8.GetBytes(plain));

        var bytes = Encoding.UTF8.GetBytes(plain);
        var protectedBytes = ProtectedData.Protect(bytes, null, DataProtectionScope.CurrentUser);
        return Convert.ToBase64String(protectedBytes);
    }

    private static string? Decrypt(string? encrypted)
    {
        if (string.IsNullOrEmpty(encrypted))
            return null;

        var bytes = Convert.FromBase64String(encrypted);

        if (!OperatingSystem.IsWindows())
            return Encoding.UTF8.GetString(bytes);

        var plain = ProtectedData.Unprotect(bytes, null, DataProtectionScope.CurrentUser);
        return Encoding.UTF8.GetString(plain);
    }

    private sealed class StoredSettings
    {
        public int RefreshIntervalSeconds { get; set; } = 300;
        public bool LocalApiEnabled { get; set; } = true;
        public int LocalApiPort { get; set; } = 6736;
        public string? EncryptedOpenCodeGoApiKey { get; set; }
        public string? CursorStateDbPath { get; set; }
        public string? AntigravityStateDbPath { get; set; }
    }
}
