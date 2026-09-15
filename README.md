# aiMonitor

Windows system-tray app for AI coding subscription usage: **Cursor**, **OpenCode Go**, and **Antigravity**.

Exposes localhost JSON for [YASB](https://github.com/amnweb/yasb) widgets.

## Requirements

- Windows 10/11
- [Bun](https://bun.sh)
- [Rust](https://rustup.rs) (see `mise.toml`)

## Setup

```powershell
mise install
bun install
```

## Development

```powershell
bun tauri dev
```

Tray icon → left-click toggles usage popup. Menu: Refresh, Settings, Quit.

## Build

```powershell
bun tauri build
```

Output: `src-tauri/target/release/aiMonitor.exe`

## Run

```powershell
.\src-tauri\target\release\aiMonitor.exe
```

## CLI export (YASB fallback)

```powershell
aiMonitor.exe export --format json
```

Same JSON as `GET http://127.0.0.1:6736/api/v1/usage`.

## Provider setup

| Provider | What you need |
|---|---|
| **Cursor** | Signed into Cursor (`%APPDATA%\Cursor\...`) |
| **OpenCode Go** | API key from `/connect` in OpenCode |
| **Antigravity** | Antigravity CLI or IDE credentials |

Optional overrides in Settings → custom paths or OpenCode API key.

## YASB

See [integrations/yasb/README.md](integrations/yasb/README.md).

## Local API

- `GET /health`
- `GET /api/v1/usage`
- `GET /api/v1/usage/{cursor|opencode_go|antigravity}`

Default port `6736`. Localhost only.

Settings: `%APPDATA%\aiMonitor\settings.json` (OpenCode key encrypted with DPAPI).

## Release

```powershell
bun run release
```

Tags `v*` trigger CI → `aiMonitor-v*-win-x64.exe` on GitHub Releases.
