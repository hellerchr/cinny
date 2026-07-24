# Fork Changes (vs `cinnyapp/cinny` upstream `dev`)

This fork adds a Tauri desktop app, Discord-style voice features, and a desktop release
pipeline. All changes are designed to be **minimal and additive** so upstream merges stay
easy. 11 commits on top of upstream `dev`.

Run the full diff with: `git diff upstream/dev...dev`

---

## 1. Tauri Desktop App

**New directory `src-tauri/`** (self-contained, never conflicts with upstream):

- `tauri.conf.json` — wraps the existing Vite build (`dist/`), window defaults, CSP disabled.
- `Cargo.toml` / `build.rs` / `src/main.rs` — Tauri v2 shell with two plugins registered:
  `global-shortcut` and our custom `ptt` module.
- `Info.plist` — **required on macOS**: `NSMicrophoneUsageDescription` and
  `NSCameraUsageDescription`. Without these, WKWebView silently blocks `getUserMedia`
  and joining any call hangs forever on the "connecting" spinner (Element Call iframe
  never gets mic access).
- `capabilities/default.json` — Tauri v2 permissions: `core:default`,
  `core:event:allow-listen/unlisten` (frontend listens for native PTT key events),
  `global-shortcut:allow-register/unregister`.
- `icons/` — generated via `npx tauri icon`.
- `flatpak/` — Flatpak manifest + `.desktop` launcher for the Linux release. Uses the
  `org.gnome.Platform` runtime (the only one shipping WebKitGTK, which Tauri needs).

**`package.json`** — added `tauri` / `tauri:dev` / `tauri:build` scripts and devDeps
(`@tauri-apps/cli`, `@tauri-apps/api`, `@tauri-apps/plugin-global-shortcut`).

## 2. Expanded Voice Channel Member List

Discord-style: instead of the red `N Live` badge, each voice channel in the room
sidebar lists who's inside (avatar + display name), live-updating.

- **`src/app/features/room-nav/RoomNavItem.tsx`** — the only file touched.
  - Removed the `{callMembers.length} Live` badge.
  - Added `RoomNavCallMembers` / `CallMemberUser` components rendered *below* the
    `NavItem`, reusing the existing `useCallSession`/`useCallMembers` hooks (MatrixRTC
    `session.memberships`, deduplicated by `userId`).
  - Root wrapped in a fragment (`<>`), which is why the diff shows ~120 re-indented
    lines; the functional change is ~60 lines.
  - Works with the existing list virtualization (`measureElement` handles the
    dynamic height).

## 3. Push to Talk (PTT)

Settings → General → **Voice**: enable/disable + key capture (stores `KeyboardEvent.code`).

- **`src/app/state/settings.ts`** — two new persisted settings: `pushToTalk` (bool,
  default false), `pushToTalkKey` (string, default `'Space'`).
- **`src/app/hooks/usePushToTalk.ts`** (new, all logic in one file) — mounted from
  `CallUtils` in **`src/app/components/CallEmbedProvider.tsx`** (+3 lines), active only
  while in a joined call. Mic starts muted; hold key = unmute, release = mute.
  Mic is driven through the existing `CallControl.toggleMicrophone()` (widget action
  `io.element.device_mute` to the Element Call iframe).
- **`src/app/features/settings/general/General.tsx`** — `Voice` settings section
  (Switch + key-capture button).

### Global hotkey — what it took (macOS)

Three layers, two of them hard-won:

1. **Focused window** (browser + desktop fallback): plain `keydown`/`keyup`/`blur`
   listeners in the hook. Ignores typing targets.
2. **Non-modifier keys, global** (`Space`, `KeyV`, …):
   `tauri-plugin-global-shortcut` (JS `register(key, cb)` with Pressed/Released).
   Uses macOS Carbon hotkeys — **no permission needed**. Verified `KeyboardEvent.code`
   strings (`Space`, `KeyV`) parse correctly in the `global-hotkey` crate.
3. **Modifier-only keys, global** (`MetaLeft`, `ShiftLeft`, …): Carbon hotkeys
   **cannot** register modifier-only keys, so these go through a native listener:
   - **`src-tauri/src/ptt.rs`** — `set_ptt_key` command + event-tap thread emitting
     `ptt-key-press` / `ptt-key-release` to the frontend.
   - First attempt used the `rdev` crate — it **crashed the app** (`EXC_BREAKPOINT`):
     rdev resolves key names via `TSMGetInputSourceProperty`, which asserts main-queue
     when called from the event-tap thread. Replaced with a raw **`CGEventTap`** on
     `FlagsChanged` events comparing virtual keycodes + `CGEventFlags` only
     (`core-graphics`/`core-foundation` crates, macOS-only; `rdev` remains for
     Windows/Linux).
   - **macOS Input Monitoring permission** is required for the tap to receive
     background events. `CGRequestListenEventAccess()` is called at startup to trigger
     the system prompt; user must grant it and restart the app.
   - ⚠️ **Unsigned/ad-hoc builds lose this grant on every rebuild** (TCC keys on the
     binary cdhash). Re-grant after each local build. Signed releases keep it.

## 4. Fullscreen Call View

A true breakout window is not feasible (the Element Call iframe is bound to the main
window's widget API), so the call can cover the entire app window instead.

- **`src/app/state/callEmbed.ts`** — new `callFullscreenAtom`, auto-reset when the
  call ends.
- **`src/app/components/CallEmbedProvider.tsx`** — container goes
  `100vw/100vh` + `config.zIndex.Max` when fullscreen; `Esc` exits; fullscreen forces
  visibility even when viewing another room.
- **`src/app/hooks/useCallEmbed.ts`** — `useCallEmbedPlacementSync` skips syncing
  while fullscreen and re-syncs on exit (also now syncs on mount).
- **`src/app/features/call-status/CallControl.tsx`** — `FullscreenButton`
  (monitor icon) in the call control bar.

## 5. Release Pipeline

**`.github/workflows/release-desktop.yml`** (new; upstream workflows untouched):

- Triggers: tags matching `v*-desktop*` or manual dispatch with a tag input.
- Matrix: `macos-latest` → DMG, `windows-latest` → NSIS `.exe`,
  `ubuntu-latest` → Flatpak only (`--bundles none`, then `flatpak-builder` +
  `build-bundle`; needs `flatpak`, `flatpak-builder`, GNOME Platform/Sdk 48).
- Uploads artifacts to the GitHub release via `softprops/action-gh-release`
  (requires `permissions: contents: write` — its absence was the first CI failure).

Upstream's `prod-deploy.yml` is **disabled on this fork** (fails without
Docker/Netlify production secrets); disabled via repo settings, not by editing the
upstream file, to keep merges clean.

---

## Merge-Maintenance Notes

- New files (no conflict risk): `src-tauri/`, `usePushToTalk.ts`,
  `release-desktop.yml`, this document.
- Modified upstream files (possible conflicts, all small): `RoomNavItem.tsx`
  (largest — re-indentation + member list), `General.tsx`, `CallControl.tsx`,
  `CallEmbedProvider.tsx`, `useCallEmbed.ts`, `callEmbed.ts`, `settings.ts`,
  `package.json` / `package-lock.json`.
