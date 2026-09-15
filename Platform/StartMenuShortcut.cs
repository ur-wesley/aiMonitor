using System.Runtime.InteropServices;

namespace aiMonitor.Platform;

internal static partial class StartMenuShortcut
{
    private const uint ClsctxInprocServer = 1;

    private static readonly Guid ShellLinkClsid = new("00021401-0000-0000-C000-000000000046");
    private static readonly Guid IShellLinkWGuid = new("000214F9-0000-0000-C000-000000000046");
    private static PropertyKey AppUserModelIdKey => new()
    {
        FormatId = new Guid(0x9F4C2855, 0x9F79, 0x4F39, 0xA8, 0xD0, 0xE1, 0xD4, 0x2D, 0xE1, 0xD5, 0xF3),
        PropertyId = 5,
    };

    public static void Ensure(string appUserModelId, string displayName)
    {
        if (!OperatingSystem.IsWindows())
            return;

        var shortcutPath = Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.StartMenu),
            "Programs",
            $"{displayName}.lnk");

        if (File.Exists(shortcutPath))
            return;

        var exePath = Environment.ProcessPath;
        if (string.IsNullOrWhiteSpace(exePath))
            return;

        Directory.CreateDirectory(Path.GetDirectoryName(shortcutPath)!);
        CreateShortcut(shortcutPath, exePath, displayName, appUserModelId);
    }

    private static void CreateShortcut(string shortcutPath, string targetPath, string description, string appUserModelId)
    {
        var clsid = ShellLinkClsid;
        var iid = IShellLinkWGuid;
        Marshal.ThrowExceptionForHR(CoCreateInstance(ref clsid, IntPtr.Zero, ClsctxInprocServer, ref iid, out var shellLinkPtr));

        var shellLink = (IShellLinkW)Marshal.GetObjectForIUnknown(shellLinkPtr)!;
        shellLink.SetPath(targetPath);
        shellLink.SetDescription(description);

        var propertyStore = (IPropertyStore)shellLink;
        var appUserModelIdKey = AppUserModelIdKey;
        propertyStore.SetValue(ref appUserModelIdKey, appUserModelId);
        propertyStore.Commit();

        var persistFile = (IPersistFile)shellLink;
        persistFile.Save(shortcutPath, true);
        Marshal.Release(shellLinkPtr);
    }

    [LibraryImport("ole32.dll")]
    private static partial int CoCreateInstance(
        ref Guid clsid,
        IntPtr outer,
        uint clsContext,
        ref Guid iid,
        out IntPtr ppv);

    [ComImport]
    [Guid("000214F9-0000-0000-C000-000000000046")]
    [InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
    private interface IShellLinkW
    {
        void SetPath([MarshalAs(UnmanagedType.LPWStr)] string path);
        void SetDescription([MarshalAs(UnmanagedType.LPWStr)] string description);
    }

    [ComImport]
    [Guid("886D8EEB-8CF2-4446-8D02-CDBA1DBDCF99")]
    [InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
    private interface IPropertyStore
    {
        void GetCount(out uint count);
        void GetAt(uint index, out PropertyKey key);
        void GetValue(ref PropertyKey key, out PropVariant value);
        void SetValue(ref PropertyKey key, [MarshalAs(UnmanagedType.LPWStr)] string value);
        void Commit();
    }

    [ComImport]
    [Guid("0000010B-0000-0000-C000-000000000046")]
    [InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
    private interface IPersistFile
    {
        void Save([MarshalAs(UnmanagedType.LPWStr)] string file, bool remember);
    }

    [StructLayout(LayoutKind.Sequential, Pack = 4)]
    private struct PropertyKey
    {
        public Guid FormatId;
        public uint PropertyId;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct PropVariant;
}
