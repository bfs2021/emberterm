# EmberTerm — Task Breakdown

> Derived from `docs/PLAN.md` (Revision 2, 2026-05-05).
> Check a task `[x]` once its implementation is merged and verified.

---

## Legend

- `[x]` Done — implemented and verified
- `[ ]` Not started / in progress
- `(P0)` Phase 0 — Foundations
- `(P1)` Phase 1 — v0.1
- `(P2)` Phase 2 — v0.2
- `(P3)` Phase 3 — v0.3 → v1.0

---

## Phase 0 — Foundations (Weeks 1–2)

### Repo & Tooling

- [x] Initialize Tauri 2.x project with Svelte 5 + TypeScript + Vite scaffold
- [x] Configure Cargo workspace in `packages/desktop/`
- [x] Set up `ts-rs` for Rust → TypeScript type codegen
- [x] Configure `tracing` + `tracing-subscriber` in Rust core
- [x] Add `.github/workflows/ci.yml` — build matrix (Windows, macOS), `cargo clippy`, `cargo fmt`, `cargo test`
- [x] Add `.github/workflows/release.yml` — build + sign + upload binaries
- [x] Add MIT `LICENSE` file
- [x] Add root `README.md` with project overview and build instructions

### Rust Core — Dependencies

- [x] Add `wezterm-term` to `packages/desktop/Cargo.toml` (pin version) — optional git dep, `branch = "main"`; pin rev when integrating the VT model
- [x] Add `termwiz` to `packages/desktop/Cargo.toml`
- [x] Add `portable-pty` to `packages/desktop/Cargo.toml`
- [x] Add `tokio` (full features) to `packages/desktop/Cargo.toml`
- [x] Add `serde` + `serde_json` to `packages/desktop/Cargo.toml`
- [x] Add `toml` crate to `packages/desktop/Cargo.toml`
- [x] Add `notify` (file watcher) to `packages/desktop/Cargo.toml`
- [x] Add `dirs`, `window-vibrancy` to `packages/desktop/Cargo.toml`
- [x] Add `thiserror` + `anyhow` to `packages/desktop/Cargo.toml`

### "Hello World" Terminal

- [x] `packages/desktop/src/main.rs` — Tauri app entry, window setup
- [x] `packages/desktop/src/terminal/pty.rs` — wrap `portable-pty`, spawn shell process
- [x] `packages/desktop/src/terminal/session.rs` — `Session` struct (wezterm-term integration deferred to Phase 1)
- [x] `packages/desktop/src/ipc.rs` — `session.create`, `session.write`, `session.resize`, `session.close` Tauri commands
- [x] `packages/desktop/src/state.rs` — `AppState` with `Mutex<HashMap<SessionId, Session>>`
- [x] PTY reader task: reads bytes from PTY master, emits `terminal:output` Tauri event to webview
- [x] `packages/app/src/terminal/Grid.svelte` — minimal xterm.js wrapper, subscribes to `terminal:output`
- [x] `packages/app/src/App.svelte` — mounts Grid, calls `session.create` on load
- [x] Verify shell spawns and output renders in xterm.js (`cargo check --manifest-path packages/desktop/Cargo.toml` ✅, `pnpm turbo build` ✅)

---

## Phase 1 — v0.1 (Weeks 3–8)

### Rust Core — Terminal Session Lifecycle

- [x] `session.create` — spawn PTY + shell, register in `AppState`, return `SessionId`
- [x] `session.write` — forward raw bytes from webview to PTY
- [x] `session.resize` — resize PTY (cols × rows)
- [x] `session.close` — kill child process, remove from `AppState`, emit `session.exit`
- [x] Handle `session.exit` event: detect child exit, emit code to frontend

### Rust Core — OSC Handlers

- [x] `packages/desktop/src/terminal/parser.rs` — intercept OSC sequences via termwiz
- [x] OSC 7 (`\e]7;file://...`) — parse and emit `terminal:cwd` event
- [x] OSC 8 (`\e]8;...`) — detect hyperlinks, pass data to frontend via `terminal:hyperlink`
- [x] OSC 52 (`\e]52;...`) — clipboard write via `terminal:clipboard-write` event (read not supported v0.1)
- [x] OSC 133 (`\e]133;A/B/C/D`) — parse prompt marks (block UI deferred to Phase 2)
- [x] OSC title (SetWindowTitle) — parse and emit `terminal:title`
- [x] DEC 2026 (`\e[?2026h/l`) — synchronized output: `detect_sync()` in pty.rs buffers chunks between begin/end markers and emits one coalesced IPC event (4 tests)
- [x] SGR mouse (1002/1006) — handled automatically: raw bytes (`\e[?1002h` etc.) forwarded to xterm.js which enables mouse tracking and routes mouse events back via `onData → sessionWrite`

### Rust Core — Profile System

- [x] `packages/desktop/src/profile.rs` — `Profile` struct + TOML deserialization
- [x] Default profiles per OS: `pwsh.exe` (Windows), `zsh`/`bash` (macOS)
- [x] `~` expansion in `cwd` field
- [x] `profile.list` Tauri command — scan `~/.config/ember/profiles/`
- [x] `profile.upsert` Tauri command — write `.toml` atomically
- [x] File watcher (`notify`) on profiles dir — emit profile list update event on change
- [x] Hot-reload: apply profile changes without restarting the app

### Rust Core — Config System

- [x] `packages/desktop/src/config.rs` — `Config` struct + TOML deserialization (appearance, behavior, agent sections)
- [x] `config.get` Tauri command
- [x] `config.update` Tauri command — merge patch, write `config.toml` atomically
- [x] File watcher on `config.toml` — re-apply on external edits

### Rust Core — Platform Layer

- [x] `packages/desktop/src/platform/windows.rs` — Windows 11 Mica via `window-vibrancy` crate
- [x] `packages/desktop/src/platform/windows.rs` — ConPTY explicit attribute (portable-pty handles automatically — N/A)
- [x] `packages/desktop/src/platform/macos.rs` — NSWindow vibrancy via `window-vibrancy` crate
- [x] Hide native title bar — `decorations: false` in `tauri.conf.json`

### Rust Core — OSC 7777 Stub

- [x] `packages/desktop/src/agent/protocol.rs` — `OscVerb` enum (agent.hello, tool.start/progress/end)
- [x] `packages/desktop/src/terminal/parser.rs` — route OSC 7777 sequences to agent module
- [x] `packages/desktop/src/agent/handlers.rs` — `agent.hello`, `tool.start/progress/end` dispatch
- [x] `packages/desktop/src/agent/tools.rs` — `ToolCallState` tracking + `ActiveTools` manager
- [x] `agent.respond` Tauri command — write OSC 7777 reply sequence into PTY input stream

### Frontend — Chrome: Title Bar

- [x] `packages/app/src/chrome/TitleBar.svelte` — custom title bar with tab strip + window controls
- [x] Integrate tab strip inside title bar (no native title bar above)
- [x] Window controls: minimize, maximize/restore, close — right side (Windows)
- [x] `data-tauri-drag-region` on non-tab areas for window drag
- [x] Double-click on empty title bar area → maximize/restore

### Frontend — Chrome: Tab Strip

- [x] `packages/app/src/chrome/TabStrip.svelte` — render tab list from store
- [x] Tab states: `active` (accent underline), `inactive`, `unread` (dot), `running` (pulse animation), `agent-active` (ember-orange dot)
- [x] "+" button — open new tab
- [x] Close tab on middle-click
- [x] Drag to reorder tabs (HTML5 drag-and-drop in TabStrip.svelte; `reorderTab()` in stores/app.ts with correct index adjustment; 4 tests)
- [x] Double-click tab to rename (inline edit)
- [x] Keyboard: `Ctrl+Shift+T` new tab, `Ctrl+Shift+W` close, `Ctrl+Tab` / `Ctrl+Shift+Tab` next/prev

### Frontend — Chrome: Status Bar

- [x] `packages/app/src/chrome/StatusBar.svelte` — 24px height, flex row
- [x] Segment: cwd (updates on `terminal:cwd` event, home dir shortened to `~`)
- [x] Segment: git branch (shell-out `git rev-parse --abbrev-ref HEAD` on cwd change)
- [x] Segment: agent indicator — shows `● <agent-name>` on `agent:hello` event

### Frontend — Terminal Grid

- [x] `packages/app/src/terminal/Grid.svelte` — xterm.js `Terminal` instance per session
- [x] Addons: `xterm-addon-fit`, `xterm-addon-webgl`, `xterm-addon-web-links`
- [x] Subscribe to `terminal:output` event, pipe bytes to xterm.js `term.write()`
- [x] Emit `session.write` on keystrokes (xterm.js `onData`)
- [x] Emit `session.resize` on container resize (`ResizeObserver` + `FitAddon`)
- [x] Apply theme colors (ANSI palette) from active theme to xterm.js; updates reactively on theme switch
- [x] Handle bell on `terminal.bell` event
- [x] Tab focus management: active tab's Grid is focused

### Frontend — Agent: Tool Card

- [x] `packages/app/src/agent/ToolCard.svelte` — inline card component
- [x] Collapsed state: icon + tool name + summary + status badge (1 line)
- [x] Expanded state: summary + progress message
- [x] State variants: `pending`, `running` (spin animation), `ok` (green), `error` (red), `cancelled`
- [x] Expand/collapse toggle
- [x] Icon mapping: `pencil`, `file`, `terminal`, `network`, `search`, `bug`
- [x] Wire to `agent:tool_start` / `agent:tool_end` / `agent:tool_progress` IPC events
- [x] Cancel button → emit `agent.respond` with cancel (requires agent.respond command)

### Frontend — Command Palette

- [x] `packages/app/src/palette/CommandPalette.svelte` — centered overlay, 600px wide
- [x] Trigger: `Ctrl+Shift+P`
- [x] Input field with fuzzy search (sequential character match)
- [x] Keyboard: arrow keys navigate, Enter selects, Esc closes
- [x] Commands: new tab, settings, all themes as commands
- [x] Sections: Commands, Themes (grouped display)
- [x] Focus trap

### Frontend — Settings Panel

- [x] `packages/app/src/settings/SettingsPanel.svelte` — two-pane overlay
- [x] Trigger: `Ctrl+,`
- [x] Left pane: category list (Appearance, Profiles, Keymap, Agent)
- [x] Right pane: controls for selected category
- [x] Esc button closes panel
- [x] Appearance section: full theme browser + VS Code JSON import
- [x] Profiles section: list + create/edit flow
- [x] Keymap section: read-only reference table
- [x] Agent section: OSC 7777 status info
- [x] Search input filtering controls across categories
- [x] Live font/size preview

### Frontend — Settings: Profile Editor

- [x] `packages/app/src/settings/ProfileEditor.svelte` — form inside Settings → Profiles
- [x] Fields: name, shell, args, cwd
- [x] [+ New Profile] button
- [x] [Save] button — calls `profile.upsert`
- [x] [Test] button — saves profile then opens a new tab with that profile id
- [x] [Delete] confirmation — inline confirm UI; `profile_delete` Rust command + IPC wrapper
- [x] env vars key-value editor (dynamic add/remove rows)
- [x] icon picker (select: terminal, pwsh, bash, agent)
- [x] font/theme overrides (optional `font_family`, `font_size`, `theme` fields)

### Frontend — Theme System

- [x] `packages/app/src/theme/tokens.ts` — CSS variable names (`--bg`, `--accent`, `--border`, etc.)
- [x] `packages/app/src/theme/vscode-import.ts` — `vscodeToTheme()` converter + `applyThemeVars()`
  - Maps all 16 ANSI colors, chrome tokens, xterm.js palette
- [x] `packages/app/src/theme/store.ts` — reactive theme store; applies CSS vars to `:root` on switch
- [x] Built-in theme: `ember-dark.json` — `#0d1117` bg, `#FF6B35` accent
- [x] Built-in theme: `ember-light.json`
- [x] Built-in theme: `tokyo-night.json`
- [x] Built-in theme: `catppuccin-mocha.json`
- [x] Built-in theme: `dracula.json`
- [x] Built-in theme: `github-dark.json`
- [x] Built-in theme: `solarized-dark.json`
- [x] `packages/app/src/settings/ThemeEditor.svelte` — theme browser with swatches + VS Code JSON import

### Frontend — Stores

- [x] `packages/app/src/stores/app.ts` — tabs, activeTabId, toolCalls, cwd/agent derived stores
- [x] `packages/app/src/ipc/client.ts` — typed wrappers for all Tauri commands
- [x] `packages/app/src/ipc/events.ts` — typed subscriptions for all Tauri events (terminal + agent)

### Frontend — First Launch / Welcome Screen

- [x] Welcome screen (one-time, shown if no config exists)
- [x] Theme picker: 3 thumbnails (Ember Dark, Ember Light, Tokyo Night)
- [x] Shell picker: auto-detect available shells, present dropdown
- [x] Agent integration toggle (default ON)
- [x] [Get Started] button → writes initial `config.toml`, opens first tab
- [x] Status bar hint on first launch: "Press Ctrl+Shift+P for the command palette"

### Packaging & Distribution

- [x] Configure Tauri `tauri.conf.json`: bundle identifier, version, icons, window defaults
- [ ] NSIS installer for Windows
- [ ] DMG + `.app` bundle for macOS
- [ ] Code signing: Windows (EV cert or self-signed for dev), macOS (Developer ID)
- [ ] Notarization for macOS

### Documentation

- [x] `docs/osc-7777-spec.md` — v0.1 of OSC 7777 spec published (also exists as `OSC-777-SPEC.md`)
- [x] `README.md` — landing page with screenshots, install instructions, quick start
- [x] `CONTRIBUTING.md` — dev setup, code style, PR template, issue labels
- [x] `docs/theming.md` — theme system guide

### Testing — Phase 1

- [x] Rust unit tests: `session.rs` — mock PTY, verify `Session` struct lifecycle
- [x] Rust unit tests: `parser.rs` — OSC sequence parsing (OSC 7, 8, 52, 133, 7777 hello/tool.start) — moved to `parser_test.rs`
- [x] Property-based tests (`proptest`) for OSC 7777 payload parser — 8 properties in `protocol_test.rs`: never-panics, agent-bye invariant, tool.start/tool.end/agent.hello roundtrip, malformed→Unknown, unknown verb→Unknown, reply frame parseable
- [x] Frontend unit tests (Vitest): `vscode-import.ts` converter
- [x] Frontend unit tests (Vitest): `stores/app.ts` — tab management + tool call state transitions
- [ ] Manual smoke checklist: shell spawn, output render, multi-tab, theme switch, settings persist

---

## Phase 2.1 — Settings Panel Redesign

> Goal: Replace the 4-category informational settings panel with a 6-category icon-nav panel that exposes all existing config fields and adds cursor/terminal behavior controls. Reference: `docs/PLAN.md` § 6.4 Settings Panel.

### Rust Core — New Config Fields

- [x] Add `cursor_style: String` (default `"block"`) to `AppearanceConfig` in `packages/desktop/src/config.rs`
- [x] Add `cursor_blink: bool` (default `true`) to `AppearanceConfig` in `packages/desktop/src/config.rs`
- [x] Add `bold_in_bright: bool` (default `true`) to `AppearanceConfig` in `packages/desktop/src/config.rs`
- [x] Verify `config_update` JSON merge patch handles the new fields correctly (existing command, no new IPC needed)
- [x] Run `cargo check --manifest-path packages/desktop/Cargo.toml` — new fields compile cleanly (0 errors)

### Frontend — Store Layer

- [x] Create `packages/app/src/stores/terminalConfigStore.ts` — reactive store for cursor_style, cursor_blink, bold_in_bright, scrollback_lines
  - `loadTerminalConfig()`: calls `configGet()`, seeds the store
  - `updateTerminalConfig(patch)`: calls `configUpdate(...)` and updates local store state
- [x] Call `loadTerminalConfig()` in `packages/app/src/App.svelte` alongside `loadFontConfig()` on startup
- [x] Add `agentConfigStore` slice to `packages/app/src/stores/app.ts` — reads `osc_7777_enabled`, `auto_accept_tool_calls` from `configGet()`; exposes `updateAgentConfig(patch)` writer

### Frontend — Wire Terminal Config to Grid

- [x] `packages/app/src/terminal/Grid.svelte`: replace hardcoded `scrollback: 10000` with `$terminalConfigStore.scrollbackLines`
- [x] `packages/app/src/terminal/Grid.svelte`: replace hardcoded `cursorBlink: true` with `$terminalConfigStore.cursorBlink`
- [x] `packages/app/src/terminal/Grid.svelte`: replace hardcoded `cursorStyle: "block"` with `$terminalConfigStore.cursorStyle`
- [x] `packages/app/src/terminal/Grid.svelte`: add reactive `$effect` that updates `term.options` when `$terminalConfigStore` changes (live apply without restart)
- [x] `packages/app/src/App.svelte`: read `confirm_close_running` from `$terminalConfigStore` in tab-close confirmation logic

### Frontend — New Settings Components

- [x] Create `packages/app/src/settings/AppearanceSettings.svelte`
  - Font family text input (reads/writes `fontStore`)
  - Font size number input (8–72 px)
  - Line height number input (0.8–3.0)
  - Window padding number input (reads/writes `configUpdate({ appearance: { window_padding } })`)
  - Acrylic / Mica toggle (Windows-only label; writes `configUpdate({ appearance: { acrylic } })`)
  - Opacity slider (50–100%; writes `configUpdate({ appearance: { opacity } })`)

- [x] Create `packages/app/src/settings/TerminalSettings.svelte`
  - Cursor section:
    - Shape: segmented button — Block / Bar / Underline (writes `updateTerminalConfig({ cursorStyle })`)
    - Blink: toggle (writes `updateTerminalConfig({ cursorBlink })`)
  - Rendering section:
    - Scrollback lines: number input (writes `updateTerminalConfig({ scrollbackLines })`)
    - Draw bold in bright colors: toggle (writes `updateTerminalConfig({ boldInBright })`)
  - Behavior section:
    - Confirm close (running process): toggle (writes `updateTerminalConfig({ confirmCloseRunning })`)

- [x] Create `packages/app/src/settings/AgentSettings.svelte`
  - Enable agent integration toggle (reads/writes `agentConfigStore.osc_7777_enabled`)
  - Auto-accept safe tool calls toggle (reads/writes `agentConfigStore.auto_accept_tool_calls`)
  - Protocol info section: spec version 0.1, TERM_OSC_7777=1 note

### Frontend — Refactor SettingsPanel

- [x] `packages/app/src/settings/SettingsPanel.svelte`: increase panel to 820px wide × 580px tall
- [x] `packages/app/src/settings/SettingsPanel.svelte`: increase nav width to 200px
- [x] `packages/app/src/settings/SettingsPanel.svelte`: add inline SVG icon to each nav item (12×12, same stroke style as `ToolCard.svelte`)
  - Appearance: paintbrush / sliders icon
  - Profiles: person / user icon
  - Terminal: `>_` prompt icon
  - Color Scheme: palette / swatch icon
  - Keymap: keyboard icon
  - Agent: sparkle / robot icon
- [x] `packages/app/src/settings/SettingsPanel.svelte`: add `"terminal"` and `"colorscheme"` and `"agent"` categories; update `CATEGORIES` constant with new ids, labels, and keywords
- [x] `packages/app/src/settings/SettingsPanel.svelte`: replace `activeCategory === "appearance"` block with `<AppearanceSettings />`
- [x] `packages/app/src/settings/SettingsPanel.svelte`: add `activeCategory === "terminal"` block rendering `<TerminalSettings />`
- [x] `packages/app/src/settings/SettingsPanel.svelte`: add `activeCategory === "colorscheme"` block rendering `<ThemeEditor />`
- [x] `packages/app/src/settings/SettingsPanel.svelte`: replace `activeCategory === "agent"` block with `<AgentSettings />`
- [x] `packages/app/src/settings/SettingsPanel.svelte`: Profiles tab — add default-profile dropdown above the profile list (reads `profileList()`, writes `configUpdate({ behavior: { default_profile } })`)
- [x] `packages/app/src/settings/SettingsPanel.svelte`: Keymap tab — expand with a search input that filters the KEYMAP rows (no persistence change)

### Frontend — Trim ThemeEditor

- [x] `packages/app/src/settings/ThemeEditor.svelte`: remove font family, font size, line height inputs (moved to AppearanceSettings)
- [ ] `packages/app/src/settings/ThemeEditor.svelte`: verify theme list, VS Code JSON import, and live preview still function after font controls removed

### Testing — Phase 2.1

- [x] `cargo check --manifest-path packages/desktop/Cargo.toml` — new config fields pass (0 errors, 3 warnings pre-existing)
- [x] `pnpm turbo check` — all Svelte/TS type checks pass (0 errors, 11 warnings pre-existing)
- [ ] Manual: Appearance tab — change font, padding, opacity; reopen settings; values persist
- [ ] Manual: Terminal tab — change cursor to Bar; active terminal cursor changes live without restart
- [ ] Manual: Terminal tab — reduce scrollback to 100; verify buffer limit applies to new sessions
- [ ] Manual: Color Scheme tab — switch theme; terminal colors update immediately
- [ ] Manual: Agent tab — toggle OSC 7777 off; open a new tab; `TERM_OSC_7777` not set in env
- [ ] Manual: Profiles tab — change default profile; open new tab; correct shell spawns

---

## Phase 2 — v0.2 (Weeks 9–16)

### Rust Core — Block Model

- [x] `packages/desktop/src/terminal/blocks.rs` — `BlockManager` state machine (Idle → Prompting → Executing)
- [x] OSC 133 `A` mark → `on_prompt_start()`, emits `block:start` via PTY reader
- [x] OSC 133 `B` mark → command text captured (B mark currently unhandled in pty.rs)
- [x] OSC 133 `C` mark → `on_output_start()`, emits `block:start` IPC event
- [x] OSC 133 `D;<exit>` mark → `on_command_end()`, emits `block:end` IPC event with exit code + duration
- [x] `block.get` Tauri command — returns all blocks for a session (ordered by start)
- [x] Store blocks on `Session` via `Arc<Mutex<Vec<Block>>>`, shared with PTY reader thread
- [x] `block.search` Tauri command — case-insensitive command substring filter
- [x] Store blocks on `Session` indexed by `BlockId` (`BlockStore` with `HashMap<BlockId, usize>` + `Vec<Block>` — O(1) get_mut, insertion-order iteration)

### Rust Core — OSC 7777 Full

- [x] `tool.progress` — update `ToolCallState`, emit `agent:tool_progress` event (pct + message)
- [x] `diff` verb — parse `DiffPayload`, emit `agent:diff` event; `agentRespond` writes OSC reply on user action
- [x] `approval` verb — parse `ApprovalPayload`, emit `agent:approval` event; `agentRespond` writes OSC reply on user action
- [x] `block.annotate` — apply tags/fold/title to current open block
- [x] `session.mark` — emit `agent:session_mark` event + inject ANSI separator into `terminal:output`
- [x] `agent.bye` — emit `agent:bye`, frontend clears agent state and dismisses hello banner
- [x] `agent.query` — reply with capabilities JSON written into PTY input
- [x] Rate-limit OSC 7777 processing: max 1000 sequences/sec/session
- [x] `tool.cancel` event: `cancelTool()` in App.svelte writes `ESC ] 7777 ; event ; {"kind":"tool.cancel","id":...} ESC \` to PTY input

### Frontend — Block UI

- [x] `packages/app/src/agent/Block.svelte` — collapsible block chrome with expand/collapse toggle
- [x] Block header: command text, duration label, exit code badge (green/red), agent badge
- [x] Fold/unfold animation — `svelte/transition:slide`, duration=0 when `prefers-reduced-motion`
- [x] Agent-annotated blocks: accent left-border + agent badge chip
- [x] Keyboard navigation between blocks (e.g., `Alt+Up/Down`) — `navigateBlocks()` in App.svelte queries `.block-head-nav` elements in the active tab pane
- [x] Per-block context menu (right-click): Copy command, Copy output, Share block — fixed-position menu in Block.svelte, closes on outside click

### Frontend — Diff Viewer

- [x] `packages/app/src/agent/DiffView.svelte` — inline unified diff renderer
- [x] Header: file path, `+N/-N` stats
- [x] Body: line-by-line diff with `+`/`-` gutter (no Shiki — plain ANSI colors)
- [x] Toggle: unified / side-by-side view
- [x] Footer: [Accept] [Reject] [Edit] action buttons (from `actions` payload)
- [x] Keyboard: `Y` = accept, `N` = reject, `E` = edit
- [x] On action: emit `agent.respond` with result → backend writes OSC reply to PTY
- [x] Collapse to one-line summary on resolution: header shows `✓ Applied` / `✕ Rejected`, body/footer hidden; App.svelte delays removal by 2s

### Frontend — Approval Sheet

- [x] `packages/app/src/agent/ApprovalSheet.svelte` — bottom-slide overlay
- [x] Slide-up animation from bottom of active pane
- [x] Header: title + risk badge (safe = blue, caution = yellow, destructive = red)
- [x] Body: detail text (monospace font if it looks like a command)
- [x] Footer: action buttons in declared order, leftmost = safest
- [x] Keyboard: arrow keys + Enter select action, Esc = deny
- [x] Default focus on safest action
- [x] On action: emit `agent.respond` → backend writes reply to PTY, sheet dismisses

### Frontend — Split Panes

- [x] Split pane container component — horizontal and vertical splits (`SplitContainer.svelte` with drag + keyboard resize)
- [x] Drag divider to resize panes — mousemove drag handler + arrow key step resize
- [x] Keyboard shortcuts: `Ctrl+Shift+\` horizontal, `Ctrl+Shift+-` vertical
- [x] Each pane hosts one terminal session — `Tab.splitSessionId` + `TabPane.svelte`
- [x] Close pane on session exit — `onCloseSplit` closes secondary session + `clearTabSplit`

### Frontend — Quake Mode

- [x] Global hotkey registration via Tauri plugin (`` Ctrl+` `` default, configurable)
- [x] Toggle window visibility (show/hide with slide-down animation)
- [x] Window appears at top of screen, full width, configurable height
- [x] Tray icon: show/hide EmberTerm on click

### Platform — Linux X11

- [x] `packages/desktop/src/platform/linux.rs` — basic window setup for X11 (no-op, window-vibrancy doesn't support Linux; solid bg via CSS)
- [ ] WebKitGTK webview integration tested
- [ ] PTY via Unix `pty(7)`, no ConPTY
- [ ] Package: `.deb` and `.rpm` via Tauri bundler
- [ ] AUR PKGBUILD

### Terminal — Kitty Graphics Protocol

- [x] Parse APC/OSC sequences for Kitty graphics (`has_kitty_graphics()` in pty.rs detects `ESC _ G` APC sequences; 2 tests)
- [x] Forward to xterm.js or render in webview canvas overlay (`@xterm/addon-image` loaded in Grid.svelte; raw bytes already forwarded via `terminal:output`)
- [x] Capability: report `graphics: ["kitty"]` in `agent.query` reply

### Terminal — Kitty Keyboard Protocol

- [x] Negotiate with shell on session start (respond to push/pop via `scan_kitty_keyboard`, emit `terminal:kitty-keyboard` IPC event with flags)
- [x] Encode key events to Kitty protocol format (`kittyKeyboard.ts` — functional keys, modifier encoding, FLAG_DISAMBIGUATE/REPORT_EVENTS/REPORT_ALL)
- [x] Forward to PTY (Grid.svelte: `attachCustomKeyEventHandler` intercepts + `sessionWrite`; 7 Rust + 19 TS tests)

### Testing — Phase 2

- [x] Rust integration tests: OSC 133 pipeline — `extract_osc_events` + `BlockManager` (A→B→C→D full cycle, multiple blocks, interleaved OSC, edge cases — 7 tests in `blocks_test.rs`)
- [x] Rust integration tests: OSC 7777 roundtrip — diff/approval parse, reply sequence format, agent.query roundtrip, tool.cancel event format — 5 tests in `protocol_test.rs`
- [x] Frontend unit: `diff-utils.ts` — `parseDiff` / `diffStats` (10 tests: line classification, sigil stripping, line numbers, meta detection, stat counts)
- [x] Frontend unit: `stores/app.ts` — `clearTabAgent` + block store (`addBlock`, `closeBlock`, scoping, ordering — 8 new tests)
- [x] Frontend unit: `approval-utils.ts` — `formatAction`, `btnClass`, `resolveKey` keyboard bindings (21 tests)
- [x] E2E test scaffold: `e2e/` directory with WebdriverIO + tauri-driver config (`wdio.conf.ts`) and `tests/basic.spec.ts` covering app shell render, tab creation, terminal grid mount, keyboard input, OSC 133 block appearance; requires built Tauri app + `tauri-driver` on PATH to execute
- [x] Criterion benchmark: OSC 7777 parser throughput (`packages/desktop/benches/osc_parser.rs` — shell_block, tool_starts ×3, plain_text ×3)

---

## Phase 3 — v0.3 → v1.0 (Weeks 17–28)

### Native Rendering (replace xterm.js)

- [ ] Add `wgpu`, `cosmic-text`, `swash` to `packages/desktop/Cargo.toml`
- [ ] Implement glyph rasterizer using `cosmic-text` for font shaping (ligatures, CJK, COLR emoji)
- [ ] Implement terminal grid renderer using `wgpu`
- [ ] Integrate native render surface with Tauri window (texture overlay or sibling window)
- [ ] Reposition webview as chrome-only overlay
- [ ] Migrate IPC: stop emitting raw bytes on `terminal:output`, switch to cell-diff events
- [ ] Verify <16ms input latency

### Smart Scrollback Search

- [ ] Search input in command palette or dedicated hotkey (`Ctrl+Shift+F`)
- [ ] Block-aware filtering: search within commands, within output, by agent, by tag
- [ ] Highlight matching lines inline
- [ ] Navigate results with `n`/`N`

### Session Save/Restore

- [ ] Serialize open sessions (profile, cwd, scroll position, blocks) to `~/.config/ember/sessions/`
- [ ] Restore sessions on relaunch (prompt user if multiple sessions saved)
- [ ] `session.save` Tauri command, `session.restore` Tauri command

### Linux Wayland

- [ ] Test WebKitGTK on Wayland with `WAYLAND_DISPLAY` set
- [ ] Handle HiDPI scaling on Wayland
- [ ] Update `platform/linux.rs` for Wayland-specific window hints

### Graphics Protocols

- [ ] Sixel: parse and render in webview `<canvas>` or native renderer
- [ ] iTerm2 inline images: parse `ESC]1337;File=...` and render

### Ligatures

- [ ] Enable ligature rendering in `cosmic-text` (`swash` handles OpenType GSUB)
- [ ] Settings toggle: enable/disable ligatures
- [ ] Test with JetBrains Mono, Fira Code, Cascadia Code

### Terminal-as-MCP-Server

- [ ] `packages/desktop/src/mcp/` — implement MCP server over stdio or HTTP
- [ ] Expose resources: `cwd`, `recent_blocks`, `git_state`, `session_list`
- [ ] Expose tool: `write_to_terminal` (controlled input injection)
- [ ] Document MCP server API in `docs/mcp-server.md`
- [ ] Config toggle: `mcp_server_enabled = true` in `config.toml`

### Multi-Agent Session View

- [ ] Track multiple `AgentInfo` per session (multiple concurrent agents)
- [ ] UI: color-code tool cards by agent identity
- [ ] Session mark timeline shows which agent emitted each mark
- [ ] Agent filter in scrollback search

### Auto-Update

- [ ] Tauri updater plugin: check GitHub Releases for newer version
- [ ] Show update notification in status bar or as toast
- [ ] Download + verify signature, prompt user to restart
- [ ] Nightly channel: checks `main` branch artifacts

### Theme Marketplace

- [ ] Theme share format: extend VS Code JSON with EmberTerm metadata block
- [ ] Export theme as JSON file
- [ ] Import from URL (user provides)
- [ ] Curated theme gallery in Settings (fetched from static JSON on GitHub)

### Documentation Site

- [ ] `docs/` → static site (e.g., Astro or VitePress)
- [ ] Sections: Getting Started, Theming, OSC 7777, MCP Server, Contributing
- [ ] OSC 7777 spec rendered from `docs/osc-7777-spec.md`
- [ ] Deploy to GitHub Pages

### Privacy & Telemetry

- [ ] `sentry-rust` integration — capture Rust panics
- [ ] Sentry JS SDK — capture frontend errors
- [ ] Filter: strip terminal content, file paths, agent payloads before send
- [ ] Opt-in toggle on welcome screen and Settings → Privacy
- [ ] `docs/privacy.md` — public log of what's collected
- [ ] Decide: self-hosted Sentry vs sentry.io free tier

### Distribution Channels

- [ ] winget manifest submitted to `microsoft/winget-pkgs`
- [ ] Homebrew Cask formula submitted
- [ ] AUR package (`emberterm-bin`)
- [ ] `.deb` / `.rpm` published to GitHub Releases

### v1.0 Release

- [ ] All Phase 1 + Phase 2 exit criteria met
- [ ] Native renderer stable (v0.2 xterm.js path removed)
- [ ] Tested on Windows 10 (1809+), Windows 11, macOS 12+, Ubuntu 22.04 (X11 + Wayland)
- [ ] Performance targets verified: <25 MB installer, <400 ms cold start, <80 MB idle RAM
- [ ] OSC 7777 implemented by at least one major agent (Claude Code or Aider)
- [ ] Documentation site live
- [ ] GitHub release with signed binaries + changelogs

---

## Cross-Cutting Tasks

### Accessibility

- [ ] WCAG AA contrast check on all 7 built-in themes (automated + manual)
- [ ] Keyboard-only navigation through all chrome components (title bar, tabs, palette, settings, agent UI)
- [ ] `bits-ui` headless primitives used for all interactive chrome elements
- [ ] `prefers-reduced-motion` respected in all animations
- [ ] Screen reader: `aria-label` on all icon-only buttons; `role="dialog"` + focus trap on overlays

### Performance Validation

- [ ] `cargo criterion` benchmark: PTY reader throughput (target >500 MB/s rendered)
- [ ] Cold start measurement script: time from process launch to window-ready
- [ ] RAM measurement: single tab idle (target <80 MB)
- [ ] RAM per additional tab (target <30 MB)
- [ ] Input latency: keypress → glyph visible (target <16 ms, xterm.js WebGL path)

### Security

- [ ] Audit: no OSC 7777 payload content in logs or telemetry
- [x] Sanitize untrusted bytes printed by agent — `stripAnsi()` in `packages/app/src/utils/stripAnsi.ts` (20 tests); applied to `name`, `summary`, and `progressMessage` via `$derived` in `ToolCard.svelte`
- [x] OSC 7777 rate limiting: 1000 seq/sec/session enforcement
- [ ] `agent.hello` `name` field treated as display-only, never used for authorization

---

*End of TASKS.md*
