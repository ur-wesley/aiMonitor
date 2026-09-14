# aiMonitor

Small Windows system-tray app that shows AI coding subscription usage for **Cursor**, **OpenCode Go**, and **Antigravity**.

Also exposes a localhost JSON API for [YASB](https://github.com/amnweb/yasb) custom widgets.

**Repository:** [cursor.com/codebase/ur-wesley/aiMonitor](https://cursor.com/codebase/ur-wesley/aiMonitor) (private — change visibility in repo settings)

## Requirements

- Windows 10/11
- WSL (for cloning — Origin CLI is not available in PowerShell)
- [.NET 10 SDK](https://dotnet.microsoft.com/download) for building

## Quick setup (clone to `C:\Arbeit\aiMonitor`)

### Option A — one command from PowerShell (after you have the repo once)

If you already have this folder, run:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\setup-windows.ps1
```

### Option B — WSL terminal (recommended first time)

```bash
# Install the Origin CLI
curl -fsSL https://downloads.cursor.com/origin/install.sh | sh

# Sign in (also sets up git credentials)
origin auth login

# Clone the repository to C:\Arbeit\aiMonitor
origin repo clone ur-wesley/aiMonitor /mnt/c/Arbeit/aiMonitor
```

If `origin` is not found after install:

```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

Or run the bundled script:

```bash
bash /mnt/c/Arbeit/aiMonitor/scripts/setup-wsl.sh
```

Origin CLI docs: https://cursor.com/docs/origin/cli

## Build

```powershell
cd C:\Arbeit\aiMonitor
dotnet publish -c Release -r win-x64 --self-contained -p:PublishReadyToRun=true
```

Output: `C:\Arbeit\aiMonitor\bin\Release\net10.0\win-x64\publish\aiMonitor.exe`

## Run

```powershell
C:\Arbeit\aiMonitor\bin\Release\net10.0\win-x64\publish\aiMonitor.exe
```

The app lives in the system tray. Left-click the icon to open the usage popup.

### CLI export (YASB fallback)

```powershell
aiMonitor.exe export --format json
```

Prints the same JSON as `GET http://127.0.0.1:6736/api/v1/usage`.

## Provider setup

| Provider | What you need |
|---|---|
| **Cursor** | Signed into Cursor on this PC (`%APPDATA%\Cursor\...`) |
| **OpenCode Go** | API key from `/connect` in OpenCode (`%USERPROFILE%\.local\share\opencode\auth.json`) |
| **Antigravity** | Signed into Antigravity CLI or IDE (Windows Credential Manager / `~\.gemini\antigravity-cli\`) |

Optional overrides: Settings → custom paths or OpenCode API key.

## YASB

See [integrations/yasb/README.md](integrations/yasb/README.md).

## Local API

- `http://127.0.0.1:6736/api/v1/usage` — full usage JSON
- Bound to localhost only; no tokens exposed

Settings are stored in `%APPDATA%\aiMonitor\settings.json` (OpenCode key encrypted with DPAPI).
