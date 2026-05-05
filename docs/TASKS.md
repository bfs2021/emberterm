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

- [ ] Initialize Tauri 2.x project with Svelte 5 + TypeScript + Vite scaffold
- [ ] Configure Cargo workspace in `src-tauri/`
- [ ] Set up `ts-rs` for Rust → TypeScript type codegen
- [ ] Configure `tracing` + `tracing-subscriber` in Rust core
- [ ] Add `.github/workflows/ci.yml` — build matrix (Windows, macOS), `cargo clippy`, `cargo fmt`, `cargo test`
- [ ] Add `.github/workflows/release.yml` — build + sign + upload binaries
- [ ] Add MIT `LICENSE` file
- [ ] Add root `README.md` with project overview and build instructions

### Rust Core — Dependencies

- [ ] Add `wezterm-term` to `Cargo.toml` (pin version)
- [ ] Add `termwiz` to `Cargo.toml`
- [ ] Add `portable-pty` to `Cargo.toml`
- [ ] Add `tokio` (full features) to `Cargo.toml`
- [ ] Add `serde` + `serde_json` to `Cargo.toml`
- [ ] Add `toml` crate to `Cargo.toml`
- [ ] Add `notify` (file watcher) to `Cargo.toml`
- [ ] Add `thiserror` + `anyhow` to `Cargo.toml`

### "Hello World" Terminal

- [ ] `src-tauri/src/main.rs` — Tauri app entry, window setup
- [ ] `src-tauri/src/terminal/pty.rs` — wrap `portable-pty`, spawn shell process
- [ ] `src-tauri/src/terminal/session.rs` — `Session` struct with `wezterm_term::Terminal`
- [ ] `src-tauri/src/ipc.rs` — `session.create`, `session.write`, `session.resize`, `session.close` Tauri commands
- [ ] `src-tauri/src/state.rs` — `AppState` with `Mutex<HashMap<SessionId, Session>>`
- [ ] PTY reader task: reads bytes from PTY master, emits `terminal:output` Tauri event to webview
- [ ] `frontend/src/terminal/Grid.svelte` — minimal xterm.js wrapper, subscribes to `terminal:output`
- [ ] `frontend/src/App.svelte` — mounts Grid, calls `session.create` on load
- [ ] Verify shell spawns and output renders in xterm.js

---

## Phase 1 — v0.1 (Weeks 3–8)

### Rust Core — Terminal Session Lifecycle

- [ ] `session.create` — spawn PTY + shell, register in `AppState`, return `SessionId`
- [ ] `session.write` — forward raw bytes from webview to PTY
- [ ] `session.resize` — resize PTY + `wezterm_term::Terminal` (cols × rows)
- [ ] `session.close` — kill child process, remove from `AppState`, emit `session.exit`
- [ ] Handle `session.exit` event: detect child exit, emit code to frontend

### Rust Core — OSC Handlers

- [ ] `src-tauri/src/terminal/parser.rs` — intercept OSC sequences from `wezterm-term`
- [ ] OSC 7 (`\e]7;file://...`) — parse and emit `terminal.cwd` event
- [ ] OSC 8 (`\e]8;...`) — detect hyperlinks, pass data to frontend
- [ ] OSC 52 (`\e]52;...`) — read/write clipboard via Tauri clipboard API
- [ ] OSC 133 (`\e]133;A/B/C/D`) — parse prompt marks, track command start/end
- [ ] OSC 1337 (`\e]1337;SetUserVar` / title variants) — parse title updates, emit `terminal.title`
- [ ] DEC 2026 (`\e[?2026h/l`) — synchronized output: buffer until sync end marker
- [ ] SGR mouse (1002/1006) — pass mouse events from xterm.js to PTY

### Rust Core — Profile System

- [ ] `src-tauri/src/profile.rs` — `Profile` struct + TOML deserialization
- [ ] Default profiles per OS: `pwsh.exe` (Windows), `zsh`/`bash` (macOS)
- [ ] `~` expansion in `cwd` field
- [ ] `profile.list` Tauri command — scan `~/.config/ember/profiles/`
- [ ] `profile.upsert` Tauri command — write `.toml` atomically
- [ ] File watcher (`notify`) on profiles dir — emit profile list update event on change
- [ ] Hot-reload: apply profile changes without restarting the app

### Rust Core — Config System

- [ ] `src-tauri/src/config.rs` — `Config` struct + TOML deserialization (appearance, behavior, keymap, agent sections)
- [ ] `config.get` Tauri command
- [ ] `config.update` Tauri command — merge patch, write `config.toml` atomically
- [ ] File watcher on `config.toml` — re-apply on external edits

### Rust Core — Platform Layer

- [ ] `src-tauri/src/platform/windows.rs` — detect Windows 11, enable Mica/Acrylic via `DWMWA_SYSTEMBACKDROP_TYPE`
- [ ] `src-tauri/src/platform/windows.rs` — ConPTY: set `PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE`
- [ ] `src-tauri/src/platform/macos.rs` — NSWindow vibrancy (`NSVisualEffectView`)
- [ ] `src-tauri/src/platform/macos.rs` — hide native title bar, enable fullsize content view

### Rust Core — OSC 7777 Stub

- [ ] `src-tauri/src/agent/protocol.rs` — define `OscVerb` enum (all verbs from spec §7–8)
- [ ] `src-tauri/src/terminal/parser.rs` — route OSC 7777 sequences to agent module
- [ ] `src-tauri/src/agent/handlers.rs` — `agent.hello`: parse, update `session.agent`, emit `agent.hello` IPC event
- [ ] `src-tauri/src/agent/tools.rs` — `tool.start`: insert `ToolCallState` into `session.active_tools`, emit `agent.tool_start` IPC event
- [ ] `src-tauri/src/agent/tools.rs` — `tool.end`: resolve `ToolCallState`, emit `agent.tool_end` IPC event
- [ ] `agent.respond` Tauri command — write OSC 7777 reply sequence into PTY input stream

### Frontend — Chrome: Title Bar

- [ ] `frontend/src/chrome/TitleBar.svelte` — custom-drawn title bar, 28px height
- [ ] Integrate tab strip inside title bar (no native title bar above)
- [ ] Window controls: minimize, maximize/restore, close — right side (Windows), left side (macOS)
- [ ] `data-tauri-drag-region` on non-tab areas for window drag
- [ ] Double-click on empty title bar area → maximize/restore

### Frontend — Chrome: Tab Strip

- [ ] `frontend/src/chrome/TabStrip.svelte` — render tab list from store
- [ ] Tab states: `active` (accent underline, full opacity), `inactive`, `unread` (dot), `running` (pulse animation), `agent-active` (ember-orange dot)
- [ ] "+" button — open new-tab dropdown with profile picker
- [ ] Close tab on middle-click
- [ ] Drag to reorder tabs
- [ ] Double-click tab to rename (inline edit)
- [ ] Keyboard: `Ctrl+Shift+T` new tab, `Ctrl+Shift+W` close, `Ctrl+Tab` / `Ctrl+Shift+Tab` next/prev

### Frontend — Chrome: Status Bar

- [ ] `frontend/src/chrome/StatusBar.svelte` — 24px height, flex row
- [ ] Segment: current shell name (from profile)
- [ ] Segment: cwd (updates on `terminal.cwd` event, truncated if long)
- [ ] Segment: git branch (shell-out `git rev-parse --abbrev-ref HEAD` on cwd change)
- [ ] Segment: agent indicator — hidden by default; shows `● <agent-name> <version>` on `agent.hello` event

### Frontend — Terminal Grid

- [ ] `frontend/src/terminal/Grid.svelte` — xterm.js `Terminal` instance per session
- [ ] Add addons: `xterm-addon-fit`, `xterm-addon-webgl`, `xterm-addon-web-links`
- [ ] Subscribe to `terminal:output` event, pipe bytes to xterm.js `term.write()`
- [ ] Emit `session.write` on keystrokes (xterm.js `onData`)
- [ ] Emit `session.resize` on container resize (`ResizeObserver` + `FitAddon`)
- [ ] Apply theme colors (ANSI palette) from active theme to xterm.js options
- [ ] Show/hide cursor, handle bell on `terminal.bell` event
- [ ] Tab focus management: active tab's Grid is focused

### Frontend — Agent: Tool Card

- [ ] `frontend/src/agent/ToolCard.svelte` — inline card component
- [ ] Collapsed state: icon + tool name + summary + status badge (1 line)
- [ ] Expanded state: `args` JSON, output text, duration, [Retry]/[Cancel] buttons
- [ ] State variants: `pending` (spinner), `running` (progress), `ok` (green check), `error` (red x)
- [ ] Expand/collapse toggle (`▾`/`▸`) with smooth animation
- [ ] Respect `prefers-reduced-motion`
- [ ] Icon mapping: `pencil`, `file`, `terminal`, `network`, `search`, `bug` → lucide-svelte icons
- [ ] Wire to `agent.tool_start` / `agent.tool_end` IPC events
- [ ] Cancel button → emit `agent.respond` with cancel

### Frontend — Command Palette

- [ ] `frontend/src/palette/CommandPalette.svelte` — centered overlay, ~600px wide
- [ ] Trigger: `Ctrl+Shift+P` (Windows/Linux), `Cmd+Shift+P` (macOS)
- [ ] Input field with fuzzy search (implement fzf-style scoring)
- [ ] Sections: Recent, Commands, Profiles, Themes, Settings
- [ ] Keyboard: arrow keys navigate, Enter selects, Esc closes
- [ ] Commands: new tab, close tab, settings, profile picker, theme picker
- [ ] Accessible: `role="dialog"`, `aria-modal`, focus trap

### Frontend — Settings Panel

- [ ] `frontend/src/settings/SettingsPanel.svelte` — two-pane overlay
- [ ] Trigger: `Ctrl+,` (Windows/Linux), `Cmd+,` (macOS)
- [ ] Left pane: category list (Appearance, Profiles, Keymap, Agent, Advanced)
- [ ] Right pane: controls for selected category
- [ ] Search input at top — filters controls across all categories
- [ ] Esc or [Done] button closes panel
- [ ] Appearance section: theme picker, font family, font size, opacity, acrylic toggle
- [ ] Live preview: font/size/opacity changes reflect in terminal grid immediately behind the panel
- [ ] Footer: "Open config file" link — opens `config.toml` in default editor

### Frontend — Settings: Profile Editor

- [ ] `frontend/src/settings/ProfileEditor.svelte` — form inside Settings → Profiles
- [ ] Fields: name, icon picker, shell path, args, cwd, env vars (key-value list), theme override, font override
- [ ] [+ New Profile] button
- [ ] [Test] button — spawns profile in a temporary tab
- [ ] [Save] button — calls `profile.upsert`
- [ ] [Delete] confirmation

### Frontend — Theme System

- [ ] `frontend/src/theme/tokens.ts` — CSS variable definitions mapping VS Code keys to EmberTerm tokens
- [ ] `frontend/src/theme/vscode-import.ts` — `vscodeToTheme(json)` converter function
  - Maps `terminal.background/foreground` → CSS vars
  - Maps all 16 ANSI colors (`terminal.ansiBlack` … `terminal.ansiBrightWhite`)
  - Maps `editor.background`, `sideBar.background`, `sideBar.border`, `tab.*`, `statusBar.*`, `focusBorder`, `errorForeground` → chrome tokens
- [ ] Apply theme CSS vars to `:root` on theme switch
- [ ] Built-in theme: `ember-dark.json` — `#0d1117` bg, `#FF6B35` accent, warm charcoal
- [ ] Built-in theme: `ember-light.json`
- [ ] Built-in theme: `tokyo-night.json` (community, with attribution)
- [ ] Built-in theme: `catppuccin-mocha.json` (community, with attribution)
- [ ] Built-in theme: `dracula.json` (community)
- [ ] Built-in theme: `github-dark.json` (community)
- [ ] Built-in theme: `solarized-dark.json` (community)
- [ ] `frontend/src/settings/ThemeEditor.svelte` — theme browser + import UI
  - List built-in themes with preview swatch
  - [Import JSON] button → paste or file picker
  - Drop `.json` to `~/.config/ember/themes/` and hot-reload

### Frontend — Stores

- [ ] `frontend/src/stores/app.ts` — Svelte stores: `sessions`, `activeSessionId`, `tabs`, `config`, `profiles`, `agent`
- [ ] `frontend/src/ipc/client.ts` — typed wrappers for all Tauri commands (auto-generated or hand-written until ts-rs codegen is wired)
- [ ] `frontend/src/ipc/events.ts` — typed event subscriptions for all Tauri events

### Frontend — First Launch / Welcome Screen

- [ ] Welcome screen (one-time, shown if no config exists)
- [ ] Theme picker: 3 thumbnails (Ember Dark, Ember Light, Tokyo Night)
- [ ] Shell picker: auto-detect available shells, present dropdown
- [ ] Agent integration toggle (default ON)
- [ ] [Get Started] button → writes initial `config.toml`, opens first tab
- [ ] Status bar hint on first launch: "Press Ctrl+Shift+P for the command palette"

### Packaging & Distribution

- [ ] Configure Tauri `tauri.conf.json`: bundle identifier, version, icons, window defaults
- [ ] NSIS installer for Windows
- [ ] DMG + `.app` bundle for macOS
- [ ] Code signing: Windows (EV cert or self-signed for dev), macOS (Developer ID)
- [ ] Notarization for macOS

### Documentation

- [ ] `docs/osc-7777-spec.md` — publish v0.1 of OSC 7777 spec (already drafted as `OSC-777-SPEC.md`)
- [ ] `README.md` — landing page with screenshots, install instructions, quick start
- [ ] `CONTRIBUTING.md` — dev setup, code style, PR template, issue labels
- [ ] `docs/theming.md` — theme system guide

### Testing — Phase 1

- [ ] Rust unit tests: `session.rs` — mock PTY, verify `Session` struct lifecycle
- [ ] Rust unit tests: `parser.rs` — OSC sequence parsing (OSC 7, 8, 52, 133, 7777 hello/tool.start/tool.end)
- [ ] Property-based tests (`proptest`) for OSC 7777 payload parser
- [ ] Frontend unit tests (Vitest): `vscode-import.ts` converter
- [ ] Frontend unit tests (Vitest): `ToolCard.svelte` — state transitions
- [ ] Manual smoke checklist: shell spawn, output render, multi-tab, theme switch, settings persist

---

## Phase 2 — v0.2 (Weeks 9–16)

### Rust Core — Block Model

- [ ] `src-tauri/src/terminal/blocks.rs` — `Block` + `BlockAnnotations` structs
- [ ] OSC 133 `A` mark → `block.start` event with new `Block` (command extracted from prompt line)
- [ ] OSC 133 `B` mark → command text captured
- [ ] OSC 133 `C` mark → output started
- [ ] OSC 133 `D;<exit>` mark → `block.end` event with exit code
- [ ] `block.get` and `block.search` Tauri commands
- [ ] Store blocks on `Session`, index by `BlockId`

### Rust Core — OSC 7777 Full

- [ ] `tool.progress` — update `ToolCallState`, emit `agent.tool_progress` event (pct + message)
- [ ] `diff` verb — parse `DiffPayload`, emit `agent.diff` event; write `reply` to PTY input on user action
- [ ] `approval` verb — parse `ApprovalPayload`, emit `agent.approval` event; write `reply` to PTY input on user action
- [ ] `block.annotate` — apply tags/fold/title to current open block
- [ ] `session.mark` — insert `SessionMark` into session timeline, emit event
- [ ] `agent.bye` — clear `session.agent`, update status bar
- [ ] `agent.query` — reply with capabilities JSON written into PTY input
- [ ] Rate-limit OSC 7777 processing: max 1000 sequences/sec/session
- [ ] `tool.cancel` event: write `ESC ] 7777 ; event ; {"kind":"tool.cancel","id":...} ESC \` to PTY input

### Frontend — Block UI

- [ ] `frontend/src/agent/` — `BlockView.svelte` — wraps a `Block` with collapsible output
- [ ] Block header: command text, duration, exit code badge
- [ ] Fold/unfold animation (respects `prefers-reduced-motion`)
- [ ] Agent-annotated blocks: show agent badge + tag chips
- [ ] Keyboard navigation between blocks (e.g., `Alt+Up/Down`)
- [ ] Per-block context menu (right-click): Copy command, Copy output, Share block

### Frontend — Diff Viewer

- [ ] `frontend/src/agent/DiffView.svelte` — inline unified diff renderer
- [ ] Header: file path, `+N/-N` stats
- [ ] Body: line-by-line diff with `+`/`-` gutter, syntax highlighting (Shiki)
- [ ] Toggle: unified / side-by-side view
- [ ] Footer: [Accept] [Reject] [Edit] action buttons
- [ ] Keyboard: `Y` = accept, `N` = reject, `E` = edit
- [ ] On action: emit `agent.respond` with result → backend writes OSC reply to PTY
- [ ] Collapse to one-line summary on resolution: `✓ Applied to src/main.rs`

### Frontend — Approval Sheet

- [ ] `frontend/src/agent/ApprovalSheet.svelte` — bottom-slide overlay
- [ ] Slide-up animation from bottom of active pane
- [ ] Header: title + risk badge (safe = blue, caution = yellow, destructive = red)
- [ ] Body: detail text (monospace font if it looks like a command)
- [ ] Footer: action buttons in declared order, leftmost = safest
- [ ] Keyboard: arrow keys + Enter select action, Esc = deny
- [ ] Default focus on safest action
- [ ] On action: emit `agent.respond` → backend writes reply to PTY, sheet dismisses

### Frontend — Split Panes

- [ ] Split pane container component — horizontal and vertical splits
- [ ] Drag divider to resize panes
- [ ] Keyboard shortcuts: `Ctrl+Shift+\` horizontal, `Ctrl+Shift+-` vertical
- [ ] Each pane hosts one terminal session
- [ ] Close pane on session exit

### Frontend — Quake Mode

- [ ] Global hotkey registration via Tauri plugin (`` Ctrl+` `` default, configurable)
- [ ] Toggle window visibility (show/hide with slide-down animation)
- [ ] Window appears at top of screen, full width, configurable height
- [ ] Tray icon: show/hide EmberTerm on click

### Platform — Linux X11

- [ ] `src-tauri/src/platform/linux.rs` — basic window setup for X11
- [ ] WebKitGTK webview integration tested
- [ ] PTY via Unix `pty(7)`, no ConPTY
- [ ] Package: `.deb` and `.rpm` via Tauri bundler
- [ ] AUR PKGBUILD

### Terminal — Kitty Graphics Protocol

- [ ] Parse APC/OSC sequences for Kitty graphics
- [ ] Forward to xterm.js or render in webview canvas overlay
- [ ] Capability: report `graphics: ["kitty"]` in `agent.query` reply

### Terminal — Kitty Keyboard Protocol

- [ ] Negotiate with shell on session start
- [ ] Encode key events to Kitty protocol format
- [ ] Forward to PTY

### Testing — Phase 2

- [ ] Rust integration tests: spawn shell, emit OSC 133 marks, assert block events
- [ ] Rust integration tests: emit OSC 7777 `diff` + simulate user reply, assert PTY input bytes
- [ ] Frontend unit: `DiffView.svelte` — renders patch, fires events
- [ ] Frontend unit: `ApprovalSheet.svelte` — keyboard nav, action dispatch
- [ ] Playwright E2E: open tab, run command, verify block appears
- [ ] Criterion benchmark: OSC 7777 parser throughput

---

## Phase 3 — v0.3 → v1.0 (Weeks 17–28)

### Native Rendering (replace xterm.js)

- [ ] Add `wgpu`, `cosmic-text`, `swash` to `Cargo.toml`
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

- [ ] `src-tauri/src/mcp/` — implement MCP server over stdio or HTTP
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
- [ ] Sanitize untrusted bytes printed by agent (strip `ESC` before rendering in tool card summaries)
- [ ] OSC 7777 rate limiting: 1000 seq/sec/session enforcement
- [ ] `agent.hello` `name` field treated as display-only, never used for authorization

---

*End of TASKS.md*
