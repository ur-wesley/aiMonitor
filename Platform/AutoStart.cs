using Microsoft.Win32;

namespace aiMonitor.Platform;

internal static class AutoStart
{
    private const string RunKeyPath = @"Software\Microsoft\Windows\CurrentVersion\Run";
    private const string ValueName = "aiMonitor";

    public static bool IsEnabled()
    {
        if (!OperatingSystem.IsWindows())
            return false;

        try
        {
            using var key = Registry.CurrentUser.OpenSubKey(RunKeyPath, false);
            return key?.GetValue(ValueName) is string;
        }
        catch
        {
            return false;
        }
    }

    public static bool SetEnabled(bool enabled)
    {
        if (!OperatingSystem.IsWindows())
            return false;

        try
        {
            using var key = Registry.CurrentUser.OpenSubKey(RunKeyPath, true);
            if (key is null)
                return false;

            if (enabled)
            {
                var exePath = Environment.ProcessPath;
                if (string.IsNullOrWhiteSpace(exePath))
                    return false;

                key.SetValue(ValueName, $"\"{exePath}\"");
            }
            else
            {
                key.DeleteValue(ValueName, false);
            }

            return true;
        }
        catch
        {
            return false;
        }
    }
}
