using System.Runtime.InteropServices;
using System.Xml.Linq;
using Windows.Data.Xml.Dom;
using Windows.UI.Notifications;

namespace aiMonitor.Platform;

public sealed partial class WindowsToast
{
    public const string AppUserModelId = "urWesley.aiMonitor";

    private Action? _onActivated;

    public void Initialize(Action onActivated)
    {
        if (!OperatingSystem.IsWindows())
            return;

        _onActivated = onActivated;
        SetCurrentProcessExplicitAppUserModelID(AppUserModelId);
        try
        {
            StartMenuShortcut.Ensure(AppUserModelId, "aiMonitor");
        }
        catch
        {
        }
    }

    public bool TryShow(string title, string body)
    {
        if (!OperatingSystem.IsWindows())
            return false;

        try
        {
            var xml = BuildToastXml(title, body);
            var doc = new XmlDocument();
            doc.LoadXml(xml);

            var toast = new ToastNotification(doc);
            toast.Activated += (_, _) => _onActivated?.Invoke();

            ToastNotificationManager.CreateToastNotifier(AppUserModelId).Show(toast);
            return true;
        }
        catch
        {
            return false;
        }
    }

    private static string BuildToastXml(string title, string body)
    {
        var toast = new XElement("toast",
            new XElement("visual",
                new XElement("binding",
                    new XAttribute("template", "ToastGeneric"),
                    new XElement("text", title),
                    new XElement("text", body))));

        return toast.ToString(SaveOptions.DisableFormatting);
    }

    [LibraryImport("shell32.dll", StringMarshalling = StringMarshalling.Utf16)]
    private static partial int SetCurrentProcessExplicitAppUserModelID(string appId);
}
