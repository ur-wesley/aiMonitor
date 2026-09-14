using System.Runtime.InteropServices;
using Avalonia;
using Avalonia.Controls;

namespace aiMonitor.Platform;

internal static class WindowPlacement
{
    private const int Margin = 8;

    public static void PositionNearCursor(Window window)
    {
        var screens = window.Screens;
        var cursor = GetCursorPosition(screens);
        var screen = screens.ScreenFromPoint(cursor) ?? screens.Primary;
        if (screen is null)
            return;

        var workArea = screen.WorkingArea;
        var maxHeightDip = (workArea.Height - Margin * 2) / screen.Scaling;
        window.MaxHeight = maxHeightDip;

        var frameSize = window.FrameSize ?? window.Bounds.Size;
        if (frameSize.Height <= 0)
            frameSize = new Size(window.Width, window.ClientSize.Height);

        var size = PixelSize.FromSize(frameSize, screen.Scaling);

        var x = cursor.X;
        var y = cursor.Y + Margin;

        if (x + size.Width > workArea.Right - Margin)
            x = cursor.X - size.Width;

        if (y + size.Height > workArea.Bottom - Margin)
            y = cursor.Y - size.Height - Margin;

        x = Math.Clamp(x, workArea.X + Margin, Math.Max(workArea.X + Margin, workArea.Right - size.Width - Margin));
        y = Math.Clamp(y, workArea.Y + Margin, Math.Max(workArea.Y + Margin, workArea.Bottom - size.Height - Margin));

        window.Position = new PixelPoint(x, y);
    }

    private static PixelPoint GetCursorPosition(Screens screens)
    {
        if (OperatingSystem.IsWindows() && GetCursorPos(out var point))
            return new PixelPoint(point.X, point.Y);

        var primary = screens.Primary;
        if (primary is null)
            return new PixelPoint(Margin, Margin);

        var workArea = primary.WorkingArea;
        return new PixelPoint(workArea.Right - Margin, workArea.Bottom - Margin);
    }

    [DllImport("user32.dll")]
    private static extern bool GetCursorPos(out Point point);

    [StructLayout(LayoutKind.Sequential)]
    private struct Point
    {
        public int X;
        public int Y;
    }
}
