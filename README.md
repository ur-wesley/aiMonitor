# UsageTray

Small Windows system-tray app that shows AI coding subscription usage for **Cursor**, **OpenCode Go**, and **Antigravity**.

Also exposes a localhost JSON API for [YASB](https://github.com/amnweb/yasb) custom widgets.

## Requirements

- Windows 10/11
- [.NET 10 SDK](https://dotnet.microsoft.com/download) for building

## Build

```powershell
dotnet publish -c Release -r win-x64 --self-contained -p:PublishReadyToRun=true
```

Output: `bin/Release/net10.0/win-x64/publish/UsageTray.exe`

## Run

```powershell
UsageTray.exe
```

The app lives in the system tray. Left-click the icon to open the usage popup.

### CLI export (YASB fallback)

```powershell
UsageTray.exe export --format json
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

Settings are stored in `%APPDATA%\UsageTray\settings.json` (OpenCode key encrypted with DPAPI).
