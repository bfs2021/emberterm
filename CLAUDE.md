# EmberTerm — Claude Context

## Overview

EmberTerm is a lightweight Windows terminal for daily use: Tauri 2 (Rust) + Svelte 5 + xterm.js, built on `portable-pty`. Many tabs with different shells (Git Bash, WSL, pwsh, cmd), split panes, and the previous session restored on reopen. It replaces Tabby, which failed to launch after updates and forgot sessions.

**Non-goals:** AI-agent protocols, scrollback restore, background shells that outlive the window, native GPU rendering, macOS/Linux packaging (macOS only builds in CI), cmd folder tracking.

**Stack:**
- Rust `1.85`: `tauri 2`, `portable-pty 0.9`, `serde`, `anyhow`/`thiserror`, `notify`, `tracing`, `sysinfo`, `fontdb`; Tauri plugins: opener, clipboard-manager (read), window-state, updater (no `zip` feature, see Cargo.toml), process
- Frontend: Svelte 5 (runes), TypeScript, Vite 6, xterm.js 5 (fit, web-links, search addons)
- IPC: Tauri commands (frontend → Rust), a Tauri Channel for PTY output (raw bytes), and Tauri events (Rust → frontend)

---

## Architecture

### Packages

The project is a **pnpm + Cargo workspace**:

| Package | Path | Purpose |
|---|---|---|
| `emberterm` (Rust) | `packages/desktop/` | Tauri 2 Rust backend (PTY, shell hooks, config/profile/session files, IPC) |
| `@emberterm/app` | `packages/app/` | Svelte 5 webview frontend |
| `@emberterm/ui` | `packages/ui/` | Shared Svelte component library (Btn, Badge, Kbd, SplitContainer) |

`scripts/version.mjs` keeps one version across Cargo.toml and the package.json files.

### File Map

```
packages/desktop/src/
  main.rs                ← Tauri entry, plugins, AppState registration
  state.rs               ← AppState (sessions, config, profiles, startup warnings), lock_or_recover
  ipc.rs                 ← all #[tauri::command] handlers
  fonts.rs               ← font_list: installed font families (fontdb), for the font picker
  fsutil.rs              ← write_atomic, quarantine, load_user_file, StartupWarning
  profile.rs             ← Profile TOML + loading (broken files are quarantined)
  config.rs              ← Config TOML + loading; save_to refuses to overwrite a broken file
  session_store.rs       ← session.json (tabs, splits, folders) load/save
  watcher.rs             ← notify watcher: config.toml / profiles hot-reload, app:warning
  terminal/
    pty.rs               ← spawn_session(): reader thread, DEC 2026 coalescing, Channel send, exit wait thread
    session.rs           ← Session struct (PTY master, writer, pid)
    shell_hooks.rs       ← per-shell hook injection: OSC 7 folder + OSC 133 busy marks (Bash/WSL/pwsh), folder restore
    busy.rs              ← process-tree "is a program running" check (cmd, pwsh)
    which.rs             ← PATH lookup, cached default shell
    events.rs            ← SessionExitPayload

packages/app/src/
  main.ts                ← Svelte mount
  App.svelte             ← root: startup, tab lifecycle, restore, autosave, shortcuts, close confirmation
  ipc/client.ts, events.ts   ← typed invoke() / listen() wrappers
  stores/
    app.ts               ← tabs, activeCwd, toasts, split/rename updaters
    sessionSnapshot.ts   ← tabs → SavedSession (session.json shape)
    terminalConfigStore.ts, gitStatus.ts
    paneBusy.ts          ← per-pane "command running" from OSC 133, not reactive
  session/
    autosave.ts          ← debounced save with flush/suspend
    profiles.ts          ← pickProfile: requested id → default → first
    spawnPane.ts         ← starts a PTY session, wires output + background-tab indicators
    confirmClose.ts      ← whether closing a tab needs a confirmation
    closeWindow.ts       ← quit after the last tab closes
  terminal/
    Grid.svelte          ← xterm.js Terminal per session; OSC 7/133/52, title, bell, find, context menu
    TabPane.svelte       ← one tab: one Grid or a split of two
    FindBar.svelte, TermMenu.svelte   ← scrollback search, right-click menu
    termKeys.ts          ← copy / paste / find key chords
    outputSink.ts        ← per-session byte sink (buffers until Grid attaches)
  utils/
    osc.ts               ← parseOsc7, parseOsc52, parseOsc133
    shortcuts.ts         ← window-level shortcuts: single source for the key handler and Keymap page
    tabLabel.ts, fontFamily.ts, clock.ts, bell.ts, warnings.ts
    updater.ts           ← check → download → install → relaunch state machine
  chrome/
    TitleBar.svelte, TabStrip.svelte, ProfileMenu.svelte, StatusBar.svelte, Toasts.svelte, ConfirmDialog.svelte
  palette/CommandPalette.svelte ← Ctrl+Shift+P: tabs, profiles, splits, themes
  settings/              ← SettingsPanel, Appearance/Terminal/Keymap/About, ThemeEditor, ProfileEditor
  welcome/WelcomeScreen.svelte  ← first-launch onboarding
  theme/                 ← tokens.ts, vscode-import.ts, store.ts, font/padding/theme config, themes/ (7 built-in)

packages/ui/src/lib/     ← Btn, Badge, Kbd, SplitContainer (barrel: index.ts)
```

### Data Flow

```
Shell (Git Bash / WSL / pwsh)
    │ bytes (incl. OSC 7 folder and OSC 133 busy marks from the injected hooks)
    ▼
portable-pty ──► reader thread ──► DEC 2026 coalescing ──► Channel (raw bytes) ──► outputSink ──► xterm.js
                 wait thread ──► terminal:exit (first of reader/wait wins)                  ├─ OSC 7   → pane cwd
                                                                                            ├─ OSC 133 → paneBusy
                                                                                            ├─ title   → tab title
                                                                                            └─ OSC 52  → clipboard
tabs store ──(500 ms debounce)──► session_save ──► session.json
startup    ──► session_load ──► session_create(profile, cwd, channel) per pane
```

Closing a tab asks first when `paneBusy` says a command is running; shells without markers (cmd, pwsh) fall back to `session_busy`, the process-tree check. Closing the last tab, or a shell exiting in it, closes the window.

---

## Coding Standards

### Rust

**Error handling**
- Return `Result<T, E>` — never panic in non-test code
- Library/module errors: define with `thiserror`; command/binary errors: use `anyhow::Result`
- Use `?` to propagate; `.expect("msg")` only for invariants that are genuinely impossible
- Log-and-discard errors only at the top-most boundary (e.g. PTY reader thread)

**Logging**
- `tracing::{info!, warn!, error!, debug!}` — never `println!` or `eprintln!` in production
- `#[instrument]` on Tauri command handlers and long-running tasks
- Log levels: `info` for session lifecycle; `debug` for per-chunk PTY data; `error` for unrecoverable failures

**Concurrency**
- PTY reader threads are `std::thread` (blocking `read()` — not tokio tasks)
- `std::sync::Mutex` for AppState accessed from sync contexts
- `tokio::sync::Mutex` only when `.await` is needed inside the lock
- IPC payloads must be `Send + Sync + Clone + serde::Serialize`

**IPC types**
- All Tauri event payload structs: `#[derive(serde::Serialize, Clone)]`
- All Tauri command return types: `#[derive(serde::Serialize)]`
- Struct field names in `snake_case` — Tauri/serde serializes to camelCase automatically
- Use the `SessionId = String` type alias — never raw `String` where a session ID is meant

**Module structure**
- Imports: std → external crates → crate-internal; blank line between groups
- No wildcard imports except `use super::*` in `#[cfg(test)]` modules
- One public `mod` per file; `pub(crate)` over `pub` where the symbol doesn't cross the crate boundary

**Tests**
- Unit tests live in a **separate `*_test.rs` file** alongside the source file:
  - Place tests in `src/session_store_test.rs`, `src/terminal/shell_hooks_test.rs`, etc.
  - Reference it from the source file with `#[cfg(test)] #[path = "shell_hooks_test.rs"] mod tests;`
  - The test file starts with `use super::*;` to access the parent module's items
  - This keeps each source file under the 300-line limit and makes tests easy to locate
- Integration tests in `packages/desktop/tests/`
- No `#[ignore]` without a comment explaining why and a tracking issue

### TypeScript / Svelte

**Svelte 5 runes — always use these, never Svelte 4 APIs inside components**
- State: `let x = $state(initial)` — not `writable()`
- Derived: `let y = $derived(expr)` — not `$: y = expr`
- Props: `let { foo, bar = defaultVal }: Props = $props()` — typed interface, always
- Effects: `$effect(() => { ... })` for side effects with reactive dependencies
- Never mutate props; pass callbacks or emit events instead

**IPC discipline**
- Only use typed wrappers from `ipc/client.ts` — never raw `invoke<any>(...)`
- Only use typed subscriptions from `ipc/events.ts` — never raw `listen(...)`
- Store `UnlistenFn` from every `listen()` call and call it in `onDestroy` or `$effect` cleanup
- Catch IPC errors and expose them as component state — never silently swallow

**Component conventions**
- Props typed with a `interface Props { ... }` block above `<script>`
- Inline handlers limited to single-expression lambdas; extract anything longer to a named function
- All colors and spacing via `var(--token)` CSS variables — no hardcoded hex values
- `<style>` block is scoped per component; global styles only in `app.css` or `:root` token declarations

**Design tokens — use names from `theme/tokens.ts` which match `references/design.html`**
- Elevation: `--bg` (base) → `--bg-elev` (raised) → `--bg-elev-2` (modal/overlay)
- Block backgrounds: `--bg-block`
- Diff colors: `--diff-add-bg`, `--diff-add-line`, `--diff-del-bg`, `--diff-del-line`
- Semantic: `--green`, `--red`, `--yellow`, `--blue` (status indicators)
- Accent variants: `--accent-soft` (hover), `--accent-glow` (focus ring)
- Do not reference ANSI palette hex values in component CSS

**Stores**
- Stores live in `stores/` (tab state in `stores/app.ts`) — no per-component stores
- Mutations only via exported updater functions; never `.update()` from outside the module
- Derived stores computed at the top of `stores/app.ts` with a one-line comment

### General

- Comments explain WHY, not WHAT — well-named code explains itself
- No TODO/FIXME in committed code — open a GitHub issue instead
- No dead code without `// used in Phase N` annotation
- **Max 300 lines per file.** If a file exceeds 300 lines, split it: extract a cohesive sub-module (Rust) or a focused child component / utility module (Svelte/TS). Tests in `mod tests` / `*.test.ts` are excluded from the line count.
- Keep functions under ~40 lines; split on responsibility, not arbitrary size
- No feature flags or backwards-compatibility shims — change the code directly

---

## IPC Quick Reference

### Tauri Commands (frontend → Rust)

| Function | Returns | Notes |
|---|---|---|
| `sessionCreate({ profileId?, cwd?, onOutput })` | `Promise<string>` | output arrives as raw bytes on a Channel |
| `sessionWrite(id, bytes)` | `Promise<void>` | |
| `sessionResize(id, cols, rows)` | `Promise<void>` | |
| `sessionClose(id)` | `Promise<void>` | |
| `sessionBusy(id)` | `Promise<boolean>` | child process running under the shell (fallback when no OSC 133) |
| `sessionLoad()` | `Promise<{ session, warning }>` | reads `session.json` |
| `sessionSave(session)` | `Promise<void>` | atomic write |
| `startupWarnings()` | `Promise<StartupWarning[]>` | drained on read |
| `profileList()` / `profileUpsert(p)` / `profileDelete(id)` | | |
| `configGet()` / `configUpdate(patch)` / `configIsFirstRun()` | | `configUpdate` refuses to overwrite a broken file |
| `gitStatus(cwd)` | `Promise<GitStatus>` | branch, dirty, insertions, deletions |
| `detectBuiltinProfiles()` | `Promise<BuiltinProfileSpec[]>` | |
| `fontList()` | `Promise<{ name, monospaced }[]>` | installed font families, cached per run |

Plugins are called through their own JS packages (not `ipc/client.ts`): clipboard read for paste, updater and process for About → Check for updates, opener for external links. Each needs a permission in `capabilities/default.json`.

### Tauri Events (Rust → frontend)

| Event | Payload | Trigger |
|---|---|---|
| `terminal:exit` | `session_id, code` | shell process ended (also on reader panic) |
| `profiles:updated` | `Profile[]` | profiles dir changed |
| `config:updated` | `Config` | config.toml changed and parses |
| `app:warning` | `StartupWarning` | a watched file failed to parse |

Folder, busy state, title, bell and clipboard come from xterm.js handlers (`utils/osc.ts`), not from Rust.

---

## Key Conventions

- Rust structs are the source of truth for IPC types — TypeScript types mirror them by hand
- Themes use VS Code-compatible JSON — do not invent a new format
- Config, profiles and `session.json` live in `<config_dir>/ember/` (`%APPDATA%\ember` on Windows); window size and position in `%APPDATA%\dev.emberterm.app`
- `SessionId` is a `String` — never expose PTY handles or OS pids to the frontend
- The `--bg-elev` CSS variable naming convention is canonical — not `--bg-elevated`
- A user-editable file that fails to parse is moved aside (`*.broken-<ts>`) at startup and reported as a toast; while the app runs it is left alone and saves are refused
- Svelte reactivity trap: props that resolve through the `$tabs` store re-run any effect that reads them; read them with `untrack` or through a `$derived`
- xterm swallows Tab and Ctrl+V; terminal-scoped keys are handled in `Grid.svelte`'s custom key handler, window-level ones in `utils/shortcuts.ts`
- Git Bash orphans child processes (MSYS), so the process tree cannot show a running command there; that is why bash/WSL announce it with OSC 133

---

## Versioning and Releases

One version everywhere: `packages/desktop/Cargo.toml` is the source (`tauri.conf.json` has no `version`), and the package.json files must match. `pnpm version:set X.Y.Z` writes them all and refreshes `Cargo.lock`; `pnpm version:check` verifies (CI runs it). To release: bump in a PR, merge, push tag `vX.Y.Z`. `.github/workflows/release.yml` checks the tag against the files, builds the signed Windows installer and publishes `latest.json` for the in-app updater. The signing key is the repo secret `TAURI_SIGNING_PRIVATE_KEY` (local copy `~/.tauri/emberterm.key`; the public key is in `tauri.conf.json`). See CONTRIBUTING.md.

---

## Dev Commands

Quick bash reference:

```bash
# Install all JS deps (pnpm workspace — run from repo root)
pnpm install

# Type-check (fast, no build)
cargo check --manifest-path packages/desktop/Cargo.toml
pnpm turbo check

# Tests
cargo test --manifest-path packages/desktop/Cargo.toml
pnpm turbo test
pnpm --filter @emberterm/app test       # app tests only

# Lint
cargo clippy --manifest-path packages/desktop/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path packages/desktop/Cargo.toml -- --check
pnpm turbo check

# Full build (JS packages first, then Rust)
pnpm turbo build
cargo build --manifest-path packages/desktop/Cargo.toml

# Dev mode
pnpm dev      # cargo tauri dev: the desktop app with hot reload
pnpm dev:ui   # Vite dev server only (no Tauri backend)

# Versions
pnpm version:check
pnpm version:set 0.2.0
```

Custom slash commands (defined in `.claude/commands/`):
- `/check` — type-check Rust + TS/Svelte without building
- `/test [filter]` — run Rust + Vitest test suites
- `/build` — full frontend + Rust build
- `/lint` — clippy + fmt check + svelte-check
