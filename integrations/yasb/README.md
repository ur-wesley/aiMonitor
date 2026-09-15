# YASB integration

aiMonitor exposes localhost JSON for [YASB](https://github.com/amnweb/yasb) custom widgets.

## Setup

1. Build or download `aiMonitor.exe` and keep it running in the system tray.
2. Confirm API responds:
   ```powershell
   curl.exe -s http://127.0.0.1:6736/api/v1/usage
   ```
3. Copy the `ai_usage` widget from [`config-snippet.yaml`](config-snippet.yaml) into `%USERPROFILE%\.config\yasb\config.yaml`.
4. Add `ai_usage` to a bar's `widgets` list.
5. Append rules from [`styles.css`](styles.css) to your YASB `styles.css`.

Update `run_cmd` and `on_right` paths in the snippet to match where you installed aiMonitor.

Left-click toggles compact/detailed view; right-click launches aiMonitor.

## Without the tray app

```yaml
exec_options:
  run_cmd: "C:\\path\\to\\aiMonitor.exe export --format json"
  run_interval: 300000
  return_format: "json"
```

## API

| Endpoint | Description |
|---|---|
| `GET /api/v1/usage` | Full YASB payload |
| `GET /api/v1/usage/cursor` | Cursor only |
| `GET /api/v1/usage/opencode_go` | OpenCode Go only |
| `GET /api/v1/usage/antigravity` | Antigravity only |
| `GET /health` | Health check |

Default port: `6736` (Settings).
