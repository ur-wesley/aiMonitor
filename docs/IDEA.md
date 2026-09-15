# aiMonitor

## Idea

Windows system-tray companion that shows AI coding subscription quotas (Cursor, OpenCode Go, Antigravity) and exposes the same data to YASB status-bar widgets via localhost JSON.

## Problem

Developers on multiple AI coding tools lose track of quota windows until they hit limits mid-session. There is no single glanceable view across providers.

## Audience

Windows developers using Cursor, OpenCode Go, and/or Antigravity who want tray-level quota awareness and optional YASB integration.

## Scope

MVP: tray icon, usage popup, settings, background refresh, low-quota toasts, localhost API, CLI export, YASB widget pack.

Later: macOS/Linux, usage history charts, additional providers.

## Features

- **Tray companion** — lives in system tray; left-click toggles usage popup; menu for refresh, settings, quit
- **Multi-provider usage** — Cursor (total/auto/API), OpenCode Go (5h/weekly/monthly), Antigravity (Gemini + Claude/GPT windows)
- **Credential pickup** — reads local auth from Cursor DB, OpenCode auth.json, Antigravity OAuth/credentials; no login UI
- **Background refresh** — polls on configurable interval; manual refresh from tray or popup
- **Low-quota alerts** — Windows toast when any window drops to ~10% remaining
- **Settings** — refresh interval, autostart, notifications, localhost API port, optional path/key overrides
- **YASB API** — `GET /api/v1/usage` on localhost with stable JSON contract
- **CLI export** — `aiMonitor export --format json` for headless YASB polling

## Non-goals

- macOS/Linux in v1
- Usage history or charts
- Direct OpenAI/Anthropic billing APIs
- User accounts or cloud sync
- Installer-first distribution (single exe is primary)

## Constraints

- Windows-only v1
- Local-first; credentials never leave the machine except to provider APIs
- Must preserve `%APPDATA%\aiMonitor\settings.json` and YASB JSON field names
- Single `aiMonitor-v*-win-x64.exe` release artifact

## Success

User sees accurate quota for all signed-in providers in tray popup; YASB widget reads localhost API; refresh never duplicates cards; settings migrate from prior .NET build.
