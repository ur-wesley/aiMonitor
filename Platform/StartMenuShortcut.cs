using System.Runtime.InteropServices;

namespace aiMonitor.Platform;

internal static partial class StartMenuShortcut
{
    private const uint ClsctxInprocServer = 1;
    private const ushort VtLpwstr = 31;

    private static readonly Guid ShellLinkClsid = new("00021401-0000-0000-C000-000000000046");
    private static readonly Guid IShellLinkWGuid = new("000214F9-0000-0000-C000-000000000046");
    private static readonly Guid IPropertyStoreGuid = new("886D8EEB-8CF2-4446-8D02-CDBA1DBDCF99");
    private static readonly Guid IPersistFileGuid = new("0000010B-0000-0000-C000-000000000046");

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
        Marshal.ThrowExceptionForHR(CoCreateInstance(ref clsid, IntPtr.Zero, ClsctxInprocServer, ref iid, out var shellLink));

        try
        {
            CallSetDescription(shellLink, description);
            CallSetPath(shellLink, targetPath);

            if (QueryInterface(shellLink, IPropertyStoreGuid, out var propertyStore) == 0)
            {
                try
                {
                    var key = AppUserModelIdKey;
                    CallPropertyStoreSetValue(propertyStore, ref key, appUserModelId);
                    CallPropertyStoreCommit(propertyStore);
                }
                finally
                {
                    Release(propertyStore);
                }
            }

            if (QueryInterface(shellLink, IPersistFileGuid, out var persistFile) == 0)
            {
                try
                {
                    CallPersistFileSave(persistFile, shortcutPath, true);
                }
                finally
                {
                    Release(persistFile);
                }
            }
        }
        finally
        {
            Release(shellLink);
        }
    }

    private static int QueryInterface(IntPtr comObject, Guid iid, out IntPtr result)
    {
        var vtable = Marshal.ReadIntPtr(comObject);
        var queryInterface = Marshal.GetDelegateForFunctionPointer<QueryInterfaceDelegate>(Marshal.ReadIntPtr(vtable));
        return queryInterface(comObject, ref iid, out result);
    }

    private static void Release(IntPtr comObject)
    {
        var vtable = Marshal.ReadIntPtr(comObject);
        var release = Marshal.GetDelegateForFunctionPointer<ReleaseDelegate>(Marshal.ReadIntPtr(vtable, 2 * IntPtr.Size));
        release(comObject);
    }

    private static void CallSetPath(IntPtr shellLink, string path)
    {
        var vtable = Marshal.ReadIntPtr(shellLink);
        var setPath = Marshal.GetDelegateForFunctionPointer<SetPathDelegate>(Marshal.ReadIntPtr(vtable, 20 * IntPtr.Size));
        Marshal.ThrowExceptionForHR(setPath(shellLink, path, 0));
    }

    private static void CallSetDescription(IntPtr shellLink, string description)
    {
        var vtable = Marshal.ReadIntPtr(shellLink);
        var setDescription = Marshal.GetDelegateForFunctionPointer<SetDescriptionDelegate>(Marshal.ReadIntPtr(vtable, 7 * IntPtr.Size));
        Marshal.ThrowExceptionForHR(setDescription(shellLink, description));
    }

    private static void CallPropertyStoreSetValue(IntPtr propertyStore, ref PropertyKey key, string value)
    {
        var variant = default(PropVariant);
        variant.Vt = VtLpwstr;
        variant.Ptr = Marshal.StringToCoTaskMemUni(value);

        try
        {
            var vtable = Marshal.ReadIntPtr(propertyStore);
            var setValue = Marshal.GetDelegateForFunctionPointer<PropertyStoreSetValueDelegate>(
                Marshal.ReadIntPtr(vtable, 6 * IntPtr.Size));
            Marshal.ThrowExceptionForHR(setValue(propertyStore, ref key, ref variant));
        }
        finally
        {
            if (variant.Ptr != IntPtr.Zero)
                Marshal.FreeCoTaskMem(variant.Ptr);
        }
    }

    private static void CallPropertyStoreCommit(IntPtr propertyStore)
    {
        var vtable = Marshal.ReadIntPtr(propertyStore);
        var commit = Marshal.GetDelegateForFunctionPointer<PropertyStoreCommitDelegate>(
            Marshal.ReadIntPtr(vtable, 7 * IntPtr.Size));
        Marshal.ThrowExceptionForHR(commit(propertyStore));
    }

    private static void CallPersistFileSave(IntPtr persistFile, string path, bool remember)
    {
        var vtable = Marshal.ReadIntPtr(persistFile);
        var save = Marshal.GetDelegateForFunctionPointer<PersistFileSaveDelegate>(Marshal.ReadIntPtr(vtable, 6 * IntPtr.Size));
        Marshal.ThrowExceptionForHR(save(persistFile, path, remember));
    }

    [LibraryImport("ole32.dll")]
    private static partial int CoCreateInstance(
        ref Guid clsid,
        IntPtr outer,
        uint clsContext,
        ref Guid iid,
        out IntPtr ppv);

    [UnmanagedFunctionPointer(CallingConvention.StdCall)]
    private delegate int QueryInterfaceDelegate(IntPtr comObject, ref Guid iid, out IntPtr ppv);

    [UnmanagedFunctionPointer(CallingConvention.StdCall)]
    private delegate uint ReleaseDelegate(IntPtr comObject);

    [UnmanagedFunctionPointer(CallingConvention.StdCall)]
    private delegate int SetPathDelegate(IntPtr shellLink, [MarshalAs(UnmanagedType.LPWStr)] string path, int flags);

    [UnmanagedFunctionPointer(CallingConvention.StdCall)]
    private delegate int SetDescriptionDelegate(IntPtr shellLink, [MarshalAs(UnmanagedType.LPWStr)] string description);

    [UnmanagedFunctionPointer(CallingConvention.StdCall)]
    private delegate int PropertyStoreSetValueDelegate(IntPtr propertyStore, ref PropertyKey key, ref PropVariant value);

    [UnmanagedFunctionPointer(CallingConvention.StdCall)]
    private delegate int PropertyStoreCommitDelegate(IntPtr propertyStore);

    [UnmanagedFunctionPointer(CallingConvention.StdCall)]
    private delegate int PersistFileSaveDelegate(IntPtr persistFile, [MarshalAs(UnmanagedType.LPWStr)] string path, [MarshalAs(UnmanagedType.Bool)] bool remember);

    [StructLayout(LayoutKind.Sequential, Pack = 4)]
    private struct PropertyKey
    {
        public Guid FormatId;
        public uint PropertyId;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct PropVariant
    {
        public ushort Vt;
        public ushort Reserved1;
        public ushort Reserved2;
        public ushort Reserved3;
        public IntPtr Ptr;
        public int Data1;
        public int Data2;
    }
}
