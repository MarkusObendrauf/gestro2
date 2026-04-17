# Gestro

OS-wide mouse gesture launcher. Hold right-click, drag in a direction, release to fire a keyboard shortcut. Tray-only app with a settings window.

## Stack

- **Frontend:** Svelte 5 (runes/`$state`/`$props`/`$effect`), Vite, TypeScript
- **Backend:** Rust, Tauri v2
- **Input capture:** `rdev` crate (`unstable_grab` feature) — global input grab on a dedicated thread (uses evdev on Linux, works on Wayland)
- **Input simulation:** `rdev::simulate` for right-click replay, `enigo` for keyboard shortcuts
- **Config:** JSON via `directories` crate (`~/.config/gestro/config.json` on Linux)
- **Build deps (Fedora):** `libevdev-devel`, `libxdo-devel`

## Architecture

```
┌─────────────────────────────────────────────────────┐
│  main.rs  →  lib.rs::run()                          │
│    Tauri Builder                                     │
│    ├── AppState { config: Mutex, config_tx: Sender } │
│    ├── IPC: get_config, save_config                  │
│    ├── System tray (Settings / Quit)                 │
│    └── setup → grabber::spawn()                      │
└─────────────────────────────────────────────────────┘
         │                          │
         ▼                          ▼
┌─────────────────┐     ┌────────────────────────┐
│  Grab Thread     │     │  Settings Window (UI)  │
│  (grabber.rs)    │     │  Svelte SPA            │
│                  │     │                        │
│  rdev::grab()    │     │  App.svelte            │
│  callback:       │     │  ├── GestureWheel      │
│  - RightPress    │     │  ├── ShortcutCapture   │
│    → suppress,   │     │  ├── SettingsPanel     │
│      start track │     │  └── StatusBar         │
│  - MouseMove     │     └────────────────────────┘
│    → update pos  │
│  - RightRelease  │
│    → finish()    │
│    → spawn       │
│      action      │
│      thread      │
└─────────────────┘
```

### Gesture lifecycle (grabber.rs + gesture.rs)

1. **RightPress** → suppressed (swallowed), `GestureTracker::start()` begins tracking
2. **MouseMove** while tracking → first move sets `origin`, subsequent moves update `current`
3. **RightRelease** → `GestureTracker::finish()` computes displacement:
   - Distance < `threshold` (default 50px) → `PassThrough` → `simulator::replay_right_click()`
   - Distance >= threshold → `Direction::classify(dx, dy)` (8-way compass via atan2)
     - Bound direction → `Fire(shortcut)` → `simulator::fire_shortcut()`
     - Unbound direction → `Unbound` → replay right-click

### Simulation guard (simulator.rs)

`SIMULATING: AtomicU32` counter prevents the grab callback from re-capturing synthetic right-click events. Set to 2 before simulating press+release; the grab callback decrements as it processes each one.

### Config hot-reload

`save_config` IPC → sends new config over `crossbeam_channel` → grab thread picks it up on next event via `try_recv` → `GestureTracker::update_config()`.

## File map

### Rust (`src-tauri/src/`)

| File | Purpose |
|------|---------|
| `main.rs` | Entry point, calls `gestro::run()` |
| `lib.rs` | Tauri setup, IPC commands (`get_config`, `save_config`), tray, `AppState` |
| `grabber.rs` | Spawns grab thread, `rdev::grab` callback routing events to `GestureTracker`, retry with backoff |
| `gesture.rs` | `GestureTracker` (start/update/finish), `GestureResult` enum, threshold logic |
| `direction.rs` | `Direction` enum (8-way), `classify(dx, dy)` via atan2, labels |
| `config.rs` | `GestroConfig`, `Shortcut`, `Modifier` — load/save JSON config |
| `simulator.rs` | `replay_right_click()`, `fire_shortcut()`, `SIMULATING` flag, key mapping |

### Frontend (`src/`)

| File | Purpose |
|------|---------|
| `main.ts` | Mounts Svelte `App` |
| `lib/App.svelte` | Root: loads config, listens for `grab-error`, orchestrates child components |
| `lib/GestureWheel.svelte` | SVG radial widget showing 8 direction wedges with bindings |
| `lib/DirectionSlot.svelte` | Single direction display (used elsewhere, not in wheel) |
| `lib/ShortcutCapture.svelte` | Modal: captures keyboard shortcut for a direction |
| `lib/SettingsPanel.svelte` | Threshold slider, launch-at-login toggle |
| `lib/StatusBar.svelte` | Running/error status indicator |
| `lib/types.ts` | `Direction`, `Shortcut`, `GestroConfig` TS types, `formatShortcut()` |
| `app.css` | Global styles, CSS variables (dark theme) |

### Config

| File | Purpose |
|------|---------|
| `src-tauri/Cargo.toml` | Rust dependencies |
| `src-tauri/tauri.conf.json` | Tauri config — no default windows (tray-only), bundle settings |
| `package.json` | npm scripts: `dev`, `build`, `check`, `tauri` |
| `vite.config.ts` | Vite + Svelte plugin, port 1420 |

## Key design decisions

- **No default windows** in `tauri.conf.json` — app is tray-only, settings window created on demand
- **ExitRequested handler** keeps app alive when all windows close (unless explicit `app.exit()`)
- **Grab thread** is separate from the main/Tauri thread; actions (shortcut firing) spawn their own threads to avoid blocking the grab callback
- **Origin set on first MouseMove** after start, not on ButtonPress (ButtonPress doesn't carry coordinates on all platforms)
- **Config channel** is `crossbeam_channel::unbounded` — non-blocking `try_recv` in grab callback

## Dev commands

```sh
npm run tauri dev     # dev mode (Vite HMR + Tauri)
npm run tauri build   # production build
npm run check         # svelte-check typecheck
cargo test --manifest-path src-tauri/Cargo.toml  # Rust tests
```

## Permissions

On Linux, the user must be in the `input` group for `rdev::grab` to work. If grab fails, the backend emits a `grab-error` Tauri event that the frontend displays. The grab thread retries up to 5 times with exponential backoff (2s, 4s, 8s, 16s, 32s).
