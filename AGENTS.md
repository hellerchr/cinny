# AGENTS.md

This repo is a **fork of [cinnyapp/cinny](https://github.com/cinnyapp/cinny)** (upstream `dev`)
with a Tauri desktop app, Discord-style voice features, and a desktop release pipeline.

**Read [FORK.md](./FORK.md) first** — it documents every difference vs upstream, how each
feature is implemented, and which files carry merge-conflict risk when syncing upstream.

## Project Layout

- Upstream web app: `src/` (React + TypeScript + matrix-js-sdk, Vite)
- Desktop shell: `src-tauri/` (Tauri v2, Rust) — fork-only, self-contained
- Release pipeline: `.github/workflows/release-desktop.yml` — fork-only
- Fork documentation: `FORK.md`

## Commands

- `npm run start` — Vite dev server (web)
- `npm run build` — production web build (`dist/`)
- `npm run tauri:dev` / `npm run tauri:build` — desktop dev / release build
- `npm run lint` (`check:eslint` + `check:prettier`), `npm run typecheck`
  - ⚠️ `typecheck` is broken upstream (matrix-js-sdk type resolution, ~790 errors on a
    clean checkout). Verify changes with `eslint`, `prettier`, and `npm run build` instead.

## Releasing Desktop Builds

Push a tag matching `v*-desktop*` (e.g. `v4.12.3-desktop.5`) → CI builds and attaches
macOS DMG, Windows NSIS `.exe`, and Linux `.flatpak` to the GitHub release.

## macOS Permissions (critical for local desktop work)

The desktop app needs two macOS privacy grants:

1. **Microphone/Camera** — declared in `src-tauri/Info.plist` (prompted automatically).
2. **Input Monitoring** — required for global push-to-talk with modifier-only keys
   (`MetaLeft` etc.), see `src-tauri/src/ptt.rs`.

Because local builds are **ad-hoc signed, macOS revokes the Input Monitoring grant on
every rebuild** (TCC keys on the binary hash). After each local `tauri:build`, you MUST:

```sh
tccutil reset ListenEvent app.cinny.desktop
# then relaunch the app, re-grant Input Monitoring when prompted, restart the app
```

Without this reset, the stale (broken) grant entry silently blocks global hotkeys.
