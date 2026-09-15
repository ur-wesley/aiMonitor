using System.Runtime.InteropServices;
using System.Text;

namespace aiMonitor.Services.Auth;

public static partial class WindowsCredentialReader
{
    private const string ServiceName = "gemini";
    private const string AccountName = "antigravity";

    public static string? ReadAntigravityToken()
    {
        if (!OperatingSystem.IsWindows())
            return null;

        if (!CredRead($"{ServiceName}:{AccountName}", CredentialType.Generic, 0, out var credentialPtr))
            return null;

        try
        {
            var credential = Marshal.PtrToStructure<Credential>(credentialPtr);
            if (credential.CredentialBlobSize == 0 || credential.CredentialBlob == IntPtr.Zero)
                return null;

            var bytes = new byte[credential.CredentialBlobSize];
            Marshal.Copy(credential.CredentialBlob, bytes, 0, (int)credential.CredentialBlobSize);
            var raw = Encoding.UTF8.GetString(bytes);

            if (raw.StartsWith("go-keyring-base64:", StringComparison.Ordinal))
                raw = Encoding.UTF8.GetString(Convert.FromBase64String(raw["go-keyring-base64:".Length..]));

            return raw;
        }
        finally
        {
            CredFree(credentialPtr);
        }
    }

    [LibraryImport("advapi32", EntryPoint = "CredReadW", SetLastError = true, StringMarshalling = StringMarshalling.Utf16)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool CredRead(string target, CredentialType type, int reservedFlag, out IntPtr credential);

    [LibraryImport("advapi32")]
    private static partial void CredFree(IntPtr credential);

    private enum CredentialType : uint
    {
        Generic = 1,
    }

    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    private struct Credential
    {
        public uint Flags;
        public CredentialType Type;
        public IntPtr TargetName;
        public IntPtr Comment;
        public long LastWritten;
        public uint CredentialBlobSize;
        public IntPtr CredentialBlob;
        public uint Persist;
        public uint AttributeCount;
        public IntPtr Attributes;
        public IntPtr TargetAlias;
        public IntPtr UserName;
    }
}
