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
- [x] Configure Cargo workspace in `src-tauri/`
- [ ] Set up `ts-rs` for Rust → TypeScript type codegen
- [x] Configure `tracing` + `tracing-subscriber` in Rust core
- [x] Add `.github/workflows/ci.yml` — build matrix (Windows, macOS), `cargo clippy`, `cargo fmt`, `cargo test`
- [x] Add `.github/workflows/release.yml` — build + sign + upload binaries
- [x] Add MIT `LICENSE` file
- [x] Add root `README.md` with project overview and build instructions

### Rust Core — Dependencies

- [ ] Add `wezterm-term` to `Cargo.toml` (pin version) — deferred to Phase 1; not on crates.io, requires git dep
- [x] Add `termwiz` to `Cargo.toml`
- [x] Add `portable-pty` to `Cargo.toml`
- [x] Add `tokio` (full features) to `Cargo.toml`
- [x] Add `serde` + `serde_json` to `Cargo.toml`
- [x] Add `toml` crate to `Cargo.toml`
- [x] Add `notify` (file watcher) to `Cargo.toml`
- [x] Add `dirs`, `window-vibrancy` to `Cargo.toml`
- [x] Add `thiserror` + `anyhow` to `Cargo.toml`

### "Hello World" Terminal

- [x] `src-tauri/src/main.rs` — Tauri app entry, window setup
- [x] `src-tauri/src/terminal/pty.rs` — wrap `portable-pty`, spawn shell process
- [x] `src-tauri/src/terminal/session.rs` — `Session` struct (wezterm-term integration deferred to Phase 1)
- [x] `src-tauri/src/ipc.rs` — `session.create`, `session.write`, `session.resize`, `session.close` Tauri commands
- [x] `src-tauri/src/state.rs` — `AppState` with `Mutex<HashMap<SessionId, Session>>`
- [x] PTY reader task: reads bytes from PTY master, emits `terminal:output` Tauri event to webview
- [x] `frontend/src/terminal/Grid.svelte` — minimal xterm.js wrapper, subscribes to `terminal:output`
- [x] `frontend/src/App.svelte` — mounts Grid, calls `session.create` on load
- [x] Verify shell spawns and output renders in xterm.js (`cargo check` ✅, `npm run build` ✅)

---

## Phase 1 — v0.1 (Weeks 3–8)

### Rust Core — Terminal Session Lifecycle

- [x] `session.create` — spawn PTY + shell, register in `AppState`, return `SessionId`
- [x] `session.write` — forward raw bytes from webview to PTY
- [x] `session.resize` — resize PTY (cols × rows)
- [x] `session.close` — kill child process, remove from `AppState`, emit `session.exit`
- [x] Handle `session.exit` event: detect child exit, emit code to frontend

### Rust Core — OSC Handlers

- [x] `src-tauri/src/terminal/parser.rs` — intercept OSC sequences via termwiz
- [x] OSC 7 (`\e]7;file://...`) — parse and emit `terminal:cwd` event
- [x] OSC 8 (`\e]8;...`) — detect hyperlinks, pass data to frontend via `terminal:hyperlink`
- [x] OSC 52 (`\e]52;...`) — clipboard write via `terminal:clipboard-write` event (read not supported v0.1)
- [x] OSC 133 (`\e]133;A/B/C/D`) — parse prompt marks (block UI deferred to Phase 2)
- [x] OSC title (SetWindowTitle) — parse and emit `terminal:title`
- [ ] DEC 2026 (`\e[?2026h/l`) — synchronized output: buffer until sync end marker
- [ ] SGR mouse (1002/1006) — pass mouse events from xterm.js to PTY

### Rust Core — Profile System

- [x] `src-tauri/src/profile.rs` — `Profile` struct + TOML deserialization
- [x] Default profiles per OS: `pwsh.exe` (Windows), `zsh`/`bash` (macOS)
- [x] `~` expansion in `cwd` field
- [x] `profile.list` Tauri command — scan `~/.config/ember/profiles/`
- [x] `profile.upsert` Tauri command — write `.toml` atomically
- [x] File watcher (`notify`) on profiles dir — emit profile list update event on change
- [x] Hot-reload: apply profile changes without restarting the app

### Rust Core — Config System

- [x] `src-tauri/src/config.rs` — `Config` struct + TOML deserialization (appearance, behavior, agent sections)
- [x] `config.get` Tauri command
- [x] `config.update` Tauri command — merge patch, write `config.toml` atomically
- [x] File watcher on `config.toml` — re-apply on external edits

### Rust Core — Platform Layer

- [x] `src-tauri/src/platform/windows.rs` — Windows 11 Mica via `window-vibrancy` crate
- [x] `src-tauri/src/platform/windows.rs` — ConPTY explicit attribute (portable-pty handles automatically — N/A)
- [x] `src-tauri/src/platform/macos.rs` — NSWindow vibrancy via `window-vibrancy` crate
- [x] Hide native title bar — `decorations: false` in `tauri.conf.json`

### Rust Core — OSC 7777 Stub

- [x] `src-tauri/src/agent/protocol.rs` — `OscVerb` enum (agent.hello, tool.start/progress/end)
- [x] `src-tauri/src/terminal/parser.rs` — route OSC 7777 sequences to agent module
- [x] `src-tauri/src/agent/handlers.rs` — `agent.hello`, `tool.start/progress/end` dispatch
- [x] `src-tauri/src/agent/tools.rs` — `ToolCallState` tracking + `ActiveTools` manager
- [x] `agent.respond` Tauri command — write OSC 7777 reply sequence into PTY input stream

### Frontend — Chrome: Title Bar

- [x] `frontend/src/chrome/TitleBar.svelte` — custom title bar with tab strip + window controls
- [x] Integrate tab strip inside title bar (no native title bar above)
- [x] Window controls: minimize, maximize/restore, close — right side (Windows)
- [x] `data-tauri-drag-region` on non-tab areas for window drag
- [x] Double-click on empty title bar area → maximize/restore

### Frontend — Chrome: Tab Strip

- [x] `frontend/src/chrome/TabStrip.svelte` — render tab list from store
- [x] Tab states: `active` (accent underline), `inactive`, `unread` (dot), `running` (pulse animation), `agent-active` (ember-orange dot)
- [x] "+" button — open new tab
- [x] Close tab on middle-click
- [ ] Drag to reorder tabs
- [x] Double-click tab to rename (inline edit)
- [x] Keyboard: `Ctrl+Shift+T` new tab, `Ctrl+Shift+W` close, `Ctrl+Tab` / `Ctrl+Shift+Tab` next/prev

### Frontend — Chrome: Status Bar

- [x] `frontend/src/chrome/StatusBar.svelte` — 24px height, flex row
- [x] Segment: cwd (updates on `terminal:cwd` event, home dir shortened to `~`)
- [x] Segment: git branch (shell-out `git rev-parse --abbrev-ref HEAD` on cwd change)
- [x] Segment: agent indicator — shows `● <agent-name>` on `agent:hello` event

### Frontend — Terminal Grid

- [x] `frontend/src/terminal/Grid.svelte` — xterm.js `Terminal` instance per session
- [x] Addons: `xterm-addon-fit`, `xterm-addon-webgl`, `xterm-addon-web-links`
- [x] Subscribe to `terminal:output` event, pipe bytes to xterm.js `term.write()`
- [x] Emit `session.write` on keystrokes (xterm.js `onData`)
- [x] Emit `session.resize` on container resize (`ResizeObserver` + `FitAddon`)
- [x] Apply theme colors (ANSI palette) from active theme to xterm.js; updates reactively on theme switch
- [x] Handle bell on `terminal.bell` event
- [x] Tab focus management: active tab's Grid is focused

### Frontend — Agent: Tool Card

- [x] `frontend/src/agent/ToolCard.svelte` — inline card component
- [x] Collapsed state: icon + tool name + summary + status badge (1 line)
- [x] Expanded state: summary + progress message
- [x] State variants: `pending`, `running` (spin animation), `ok` (green), `error` (red), `cancelled`
- [x] Expand/collapse toggle
- [x] Icon mapping: `pencil`, `file`, `terminal`, `network`, `search`, `bug`
- [x] Wire to `agent:tool_start` / `agent:tool_end` / `agent:tool_progress` IPC events
- [x] Cancel button → emit `agent.respond` with cancel (requires agent.respond command)

### Frontend — Command Palette

- [x] `frontend/src/palette/CommandPalette.svelte` — centered overlay, 600px wide
- [x] Trigger: `Ctrl+Shift+P`
- [x] Input field with fuzzy search (sequential character match)
- [x] Keyboard: arrow keys navigate, Enter selects, Esc closes
- [x] Commands: new tab, settings, all themes as commands
- [x] Sections: Commands, Themes (grouped display)
- [x] Focus trap

### Frontend — Settings Panel

- [x] `frontend/src/settings/SettingsPanel.svelte` — two-pane overlay
- [x] Trigger: `Ctrl+,`
- [x] Left pane: category list (Appearance, Profiles, Keymap, Agent)
- [x] Right pane: controls for selected category
- [x] Esc button closes panel
- [x] Appearance section: full theme browser + VS Code JSON import
- [x] Profiles section: list + create/edit flow
- [x] Keymap section: read-only reference table
- [x] Agent section: OSC 7777 status info
- [x] Search input filtering controls across categories
- [ ] Live font/size preview

### Frontend — Settings: Profile Editor

- [x] `frontend/src/settings/ProfileEditor.svelte` — form inside Settings → Profiles
- [x] Fields: name, shell, args, cwd
- [x] [+ New Profile] button
- [x] [Save] button — calls `profile.upsert`
- [ ] [Test] button — spawns profile in a temporary tab
- [ ] [Delete] confirmation
- [ ] env vars, icon picker, font/theme overrides

### Frontend — Theme System

- [x] `frontend/src/theme/tokens.ts` — CSS variable names (`--bg`, `--accent`, `--border`, etc.)
- [x] `frontend/src/theme/vscode-import.ts` — `vscodeToTheme()` converter + `applyThemeVars()`
  - Maps all 16 ANSI colors, chrome tokens, xterm.js palette
- [x] `frontend/src/theme/store.ts` — reactive theme store; applies CSS vars to `:root` on switch
- [x] Built-in theme: `ember-dark.json` — `#0d1117` bg, `#FF6B35` accent
- [x] Built-in theme: `ember-light.json`
- [x] Built-in theme: `tokyo-night.json`
- [x] Built-in theme: `catppuccin-mocha.json`
- [x] Built-in theme: `dracula.json`
- [x] Built-in theme: `github-dark.json`
- [x] Built-in theme: `solarized-dark.json`
- [x] `frontend/src/settings/ThemeEditor.svelte` — theme browser with swatches + VS Code JSON import

### Frontend — Stores

- [x] `frontend/src/stores/app.ts` — tabs, activeTabId, toolCalls, cwd/agent derived stores
- [x] `frontend/src/ipc/client.ts` — typed wrappers for all Tauri commands
- [x] `frontend/src/ipc/events.ts` — typed subscriptions for all Tauri events (terminal + agent)

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

- [ ] Rust unit tests: `session.rs` — mock PTY, verify `Session` struct lifecycle
- [x] Rust unit tests: `parser.rs` — OSC sequence parsing (OSC 7, 8, 52, 133, 7777 hello/tool.start)
- [ ] Property-based tests (`proptest`) for OSC 7777 payload parser
- [x] Frontend unit tests (Vitest): `vscode-import.ts` converter
- [x] Frontend unit tests (Vitest): `stores/app.ts` — tab management + tool call state transitions
- [ ] Manual smoke checklist: shell spawn, output render, multi-tab, theme switch, settings persist

---

## Phase 2 — v0.2 (Weeks 9–16)

### Rust Core — Block Model

- [x] `src-tauri/src/terminal/blocks.rs` — `BlockManager` state machine (Idle → Prompting → Executing)
- [x] OSC 133 `A` mark → `on_prompt_start()`, emits `block:start` via PTY reader
- [ ] OSC 133 `B` mark → command text captured (B mark currently unhandled in pty.rs)
- [x] OSC 133 `C` mark → `on_output_start()`, emits `block:start` IPC event
- [x] OSC 133 `D;<exit>` mark → `on_command_end()`, emits `block:end` IPC event with exit code + duration
- [x] `block.get` Tauri command — returns all blocks for a session (ordered by start)
- [x] Store blocks on `Session` via `Arc<Mutex<Vec<Block>>>`, shared with PTY reader thread
- [ ] `block.search` Tauri command
- [ ] Store blocks on `Session` indexed by `BlockId` (HashMap lookup, deferred)

### Rust Core — OSC 7777 Full

- [x] `tool.progress` — update `ToolCallState`, emit `agent:tool_progress` event (pct + message)
- [x] `diff` verb — parse `DiffPayload`, emit `agent:diff` event; `agentRespond` writes OSC reply on user action
- [x] `approval` verb — parse `ApprovalPayload`, emit `agent:approval` event; `agentRespond` writes OSC reply on user action
- [ ] `block.annotate` — apply tags/fold/title to current open block
- [x] `session.mark` — emit `agent:session_mark` event + inject ANSI separator into `terminal:output`
- [x] `agent.bye` — emit `agent:bye`, frontend clears agent state and dismisses hello banner
- [ ] `agent.query` — reply with capabilities JSON written into PTY input
- [ ] Rate-limit OSC 7777 processing: max 1000 sequences/sec/session
- [x] `tool.cancel` event: `cancelTool()` in App.svelte writes `ESC ] 7777 ; event ; {"kind":"tool.cancel","id":...} ESC \` to PTY input

### Frontend — Block UI

- [x] `frontend/src/agent/Block.svelte` — collapsible block chrome with expand/collapse toggle
- [x] Block header: command text, duration label, exit code badge (green/red), agent badge
- [x] Fold/unfold animation — `svelte/transition:slide`, duration=0 when `prefers-reduced-motion`
- [x] Agent-annotated blocks: accent left-border + agent badge chip
- [ ] Keyboard navigation between blocks (e.g., `Alt+Up/Down`)
- [ ] Per-block context menu (right-click): Copy command, Copy output, Share block

### Frontend — Diff Viewer

- [x] `frontend/src/agent/DiffView.svelte` — inline unified diff renderer
- [x] Header: file path, `+N/-N` stats
- [x] Body: line-by-line diff with `+`/`-` gutter (no Shiki — plain ANSI colors)
- [ ] Toggle: unified / side-by-side view
- [x] Footer: [Accept] [Reject] [Edit] action buttons (from `actions` payload)
- [x] Keyboard: `Y` = accept, `N` = reject, `E` = edit
- [x] On action: emit `agent.respond` with result → backend writes OSC reply to PTY
- [x] Collapse to one-line summary on resolution: header shows `✓ Applied` / `✕ Rejected`, body/footer hidden; App.svelte delays removal by 2s

### Frontend — Approval Sheet

- [x] `frontend/src/agent/ApprovalSheet.svelte` — bottom-slide overlay
- [x] Slide-up animation from bottom of active pane
- [x] Header: title + risk badge (safe = blue, caution = yellow, destructive = red)
- [x] Body: detail text (monospace font if it looks like a command)
- [x] Footer: action buttons in declared order, leftmost = safest
- [x] Keyboard: arrow keys + Enter select action, Esc = deny
- [x] Default focus on safest action
- [x] On action: emit `agent.respond` → backend writes reply to PTY, sheet dismisses

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
- [x] Frontend unit: `diff-utils.ts` — `parseDiff` / `diffStats` (10 tests: line classification, sigil stripping, line numbers, meta detection, stat counts)
- [x] Frontend unit: `stores/app.ts` — `clearTabAgent` + block store (`addBlock`, `closeBlock`, scoping, ordering — 8 new tests)
- [ ] Frontend unit: `ApprovalSheet.svelte` — keyboard nav, action dispatch (needs @testing-library/svelte)
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
