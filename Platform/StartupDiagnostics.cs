namespace aiMonitor.Platform;

internal static class StartupDiagnostics
{
    public static void RegisterUnhandledExceptionLogger()
    {
        AppDomain.CurrentDomain.UnhandledException += (_, e) =>
        {
            try
            {
                var dir = Path.Combine(
                    Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
                    "aiMonitor");
                Directory.CreateDirectory(dir);
                var path = Path.Combine(dir, "crash.log");
                File.AppendAllText(path, $"{DateTimeOffset.UtcNow:O} {e.ExceptionObject}{Environment.NewLine}");
            }
            catch
            {
            }
        };
    }
}
