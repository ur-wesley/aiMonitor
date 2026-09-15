using System.Security.Cryptography;
using System.Text;
using System.Text.Json;
using aiMonitor.Serialization;

namespace aiMonitor.Configuration;

public sealed class SettingsStore
{
    public AppSettings Load()
    {
        var path = PathResolver.SettingsFilePath();
        if (!File.Exists(path))
            return new AppSettings();

        try
        {
            var json = File.ReadAllText(path);
            var stored = JsonSerializer.Deserialize(json, AppJsonContext.Default.StoredSettingsDto);
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
                StartWithWindows = stored.StartWithWindows,
                LowUsageNotificationsEnabled = stored.LowUsageNotificationsEnabled,
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

        var stored = new StoredSettingsDto
        {
            RefreshIntervalSeconds = settings.RefreshIntervalSeconds,
            LocalApiEnabled = settings.LocalApiEnabled,
            LocalApiPort = settings.LocalApiPort,
            EncryptedOpenCodeGoApiKey = Encrypt(settings.OpenCodeGoApiKey),
            CursorStateDbPath = settings.CursorStateDbPath,
            AntigravityStateDbPath = settings.AntigravityStateDbPath,
            StartWithWindows = settings.StartWithWindows,
            LowUsageNotificationsEnabled = settings.LowUsageNotificationsEnabled,
        };

        File.WriteAllText(path, JsonSerializer.Serialize(stored, AppJsonContext.Default.StoredSettingsDto));
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
}
