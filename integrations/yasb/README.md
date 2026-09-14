# YASB integration

UsageTray exposes a localhost JSON API for [YASB](https://github.com/amnweb/yasb) custom widgets.

## Setup

1. Build or download `UsageTray.exe` and keep it running in the system tray.
2. Confirm the API responds:
   ```powershell
   curl.exe -s http://127.0.0.1:6736/api/v1/usage
   ```
3. Copy widget blocks from [`config-snippet.yaml`](config-snippet.yaml) into `%USERPROFILE%\.config\yasb\config.yaml`.
4. Add `ai_usage` (or per-provider widgets) to a bar's `widgets` list.
5. Optional: append rules from [`styles.css`](styles.css) to your YASB `styles.css`.

## Without the tray app

Use the CLI export mode as `run_cmd` instead:

```yaml
exec_options:
  run_cmd: "UsageTray.exe export --format json"
  run_interval: 300000
  return_format: "json"
```

This fetches all providers on every poll and is slower than the localhost API.

## API

| Endpoint | Description |
|---|---|
| `GET /api/v1/usage` | Full YASB payload |
| `GET /api/v1/usage/cursor` | Cursor provider only |
| `GET /api/v1/usage/opencode_go` | OpenCode Go provider only |
| `GET /api/v1/usage/antigravity` | Antigravity provider only |
| `GET /health` | Health check |

Default port: `6736` (change in UsageTray Settings).
