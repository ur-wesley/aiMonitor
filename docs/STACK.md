# Stack

## Platform

desktop (Windows-only v1)

## Prescribed

| Layer | Choice |
| --- | --- |
| Desktop | Tauri v2 + Rust |
| Frontend | SolidJS, Tailwind 4, solid-ui |
| Icons | Iconify MDI (`i-mdi-<icon>`) |
| Logic | `@ur-wesley/ts-prelude/<subpath>` |
| Tooling | Bun, Oxc (oxlint + oxfmt), lefthook |

Skills: tauri-solid-desktop, solidjs-ui, ts-styleguide

## Extras

- axum — localhost HTTP API
- reqwest — provider HTTP
- rusqlite — read Cursor/Antigravity state.vscdb
- windows crate — DPAPI, CredRead, registry autostart, WinRT toasts

## Commands

```bash
mise install
bun install
bun tauri dev
bun tauri build
bun run lint
bun run typecheck
bun run test
bun run release
```

Never `npm` / `pnpm`.
