# EmberTerm — Claude Context

## Project Docs

@docs/PLAN.md
@docs/OSC-777-SPEC.md
@docs/TASKS.md

---

## Overview

EmberTerm is an **agent-native terminal emulator**: Tauri 2 (Rust backend) + Svelte 5 (webview frontend). It uses `wezterm-term`/`termwiz`/`portable-pty` for the terminal engine and defines **OSC 7777**, an open protocol for AI agent integration that lets terminals display tool calls, diffs, and approval prompts inline — without any agent-specific plugins.

**Positioning:** WezTerm engine + Tabby-quality UI + no account + open agent protocol. Target audience: developers who run Claude Code, Aider, or Codex CLI daily and want a polished host terminal.

**Stack:**
- Rust `1.85`: `tauri 2`, `termwiz 0.23`, `portable-pty 0.9`, `serde`, `anyhow`/`thiserror`, `notify`, `tracing`
- Frontend: Svelte 5 (runes), TypeScript, Vite 6, xterm.js 5
- IPC: Tauri events (Rust → frontend) and Tauri commands (frontend → Rust)

**Performance targets:** < 25 MB installer · < 400 ms cold start · < 80 MB idle RAM (1 tab) · < 30 MB per extra tab · > 500 MB/s PTY throughput · < 16 ms input latency

---

## UI Design Reference

**`references/design.html`** is the canonical design mockup for EmberTerm's UI. Open it in a browser to inspect the visual target — colors, layout, component shapes, spacing, and interaction patterns. All frontend work must match or align with this mockup. When implementing or reviewing UI components, check them against `references/design.html` first.

See `docs/review-ui-ux.md` for the current gap analysis and prioritised task checklist before starting any frontend work.

---

## Architecture

### File Map

```
src-tauri/src/
  main.rs                ← Tauri entry, window setup, AppState registration
  state.rs               ← AppState: Mutex<HashMap<SessionId, Session>>
  ipc.rs                 ← all #[tauri::command] handlers
  profile.rs             ← Profile TOML deserialization + hot-reload watcher
  config.rs              ← Config TOML deserialization + hot-reload watcher
  terminal/
    mod.rs
    session.rs           ← Session struct (PTY master + metadata)
    pty.rs               ← spawn_session() — PTY reader thread + OSC dispatch
    parser.rs            ← extract_osc_events() using termwiz
    blocks.rs            ← Phase 2: OSC 133 → Block model
  agent/
    mod.rs
    protocol.rs          ← OscEvent enum
    handlers.rs          ← dispatch() routes OscEvent → Tauri emit
    tools.rs             ← ActiveTools, ToolCallState lifecycle
  platform/
    windows.rs           ← Mica/Acrylic via window-vibrancy
    macos.rs             ← NSWindow vibrancy via window-vibrancy

frontend/src/
  main.ts                ← Svelte mount
  App.svelte             ← root: IPC listeners, tab lifecycle, keyboard shortcuts
  ipc/
    client.ts            ← typed invoke() wrappers (source of truth for commands)
    events.ts            ← typed listen() wrappers (source of truth for events)
  stores/
    app.ts               ← tabs, toolCalls, activeCwd, activeAgentName
  terminal/
    Grid.svelte          ← xterm.js Terminal instance per session
  chrome/
    TitleBar.svelte      ← custom title bar with tab strip + window controls
    TabStrip.svelte      ← tab list: rename, close, active/agent/running states
    StatusBar.svelte     ← cwd, git branch, agent name, clock
  agent/
    ToolCard.svelte      ← inline OSC 7777 tool call card (Phase 1)
    DiffView.svelte      ← Phase 2: inline diff viewer
    ApprovalSheet.svelte ← Phase 2: bottom-anchored approval overlay
  palette/
    CommandPalette.svelte← Ctrl+Shift+P overlay with fuzzy search
  settings/
    SettingsPanel.svelte ← two-pane overlay (Appearance, Profiles, Keymap, Agent)
    ThemeEditor.svelte   ← theme browser + VS Code JSON import
    ProfileEditor.svelte ← profile create/edit form
  welcome/
    WelcomeScreen.svelte ← first-launch onboarding
  theme/
    tokens.ts            ← CSS variable name constants (match design spec)
    vscode-import.ts     ← VS Code JSON → CSS vars converter + applyThemeVars()
    store.ts             ← reactive theme store; applies CSS vars to :root
    themes/              ← 7 built-in VS Code-format JSON themes
```

### Data Flow

```
Shell/Agent
    │ bytes
    ▼
portable-pty (PTY master)
    │
    ├─ raw bytes ──────────► tauri emit "terminal:output" ──► xterm.js (render)
    │
    └─ extract_osc_events()
         │
         ├─ Title/Cwd/Hyperlink/Clipboard ──► tauri emit typed event
         ├─ PromptMark ─────────────────────► Phase 2 block model
         └─ Osc7777 { verb, payload } ───────► handlers::dispatch()
                                                    │
                                                    └─► tauri emit agent:* event ──► ToolCard / DiffView / ApprovalSheet
```

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
- Unit tests in `mod tests` at the bottom of each source file, guarded by `#[cfg(test)]`
- Integration tests in `src-tauri/tests/`
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
- All stores live in `stores/app.ts` — no per-component stores
- Mutations only via exported updater functions; never `.update()` from outside the module
- Derived stores computed at the top of `stores/app.ts` with a one-line comment

### General

- Comments explain WHY, not WHAT — well-named code explains itself
- No TODO/FIXME in committed code — file a task in `docs/TASKS.md` instead
- No dead code without `// used in Phase N` annotation
- **Max 300 lines per file.** If a file exceeds 300 lines, split it: extract a cohesive sub-module (Rust) or a focused child component / utility module (Svelte/TS). Tests in `mod tests` / `*.test.ts` are excluded from the line count.
- Keep functions under ~40 lines; split on responsibility, not arbitrary size
- No feature flags or backwards-compatibility shims — change the code directly

---

## IPC Quick Reference

### Tauri Commands (frontend → Rust)

| Function | Returns | Notes |
|---|---|---|
| `sessionCreate()` | `Promise<string>` | SessionId |
| `sessionWrite(id, bytes)` | `Promise<void>` | bytes = `number[]` |
| `sessionResize(id, cols, rows)` | `Promise<void>` | |
| `sessionClose(id)` | `Promise<void>` | kills child, removes from AppState |
| `agentRespond(id, verb, payload)` | `Promise<void>` | writes OSC 7777 reply to PTY stdin |
| `profileList()` | `Promise<Profile[]>` | scans `~/.config/ember/profiles/` |
| `profileUpsert(p)` | `Promise<void>` | writes TOML atomically |
| `configGet()` | `Promise<Config>` | |
| `configUpdate(patch)` | `Promise<void>` | merged patch, writes TOML atomically |
| `configIsFirstRun()` | `Promise<boolean>` | true if config.toml does not yet exist |
| `gitBranch(cwd)` | `Promise<string \| null>` | shells out `git rev-parse` |

### Tauri Events (Rust → frontend)

| Event | Payload fields | Trigger |
|---|---|---|
| `terminal:output` | `session_id, data: number[]` | raw PTY bytes |
| `terminal:title` | `session_id, title` | OSC 0/2 |
| `terminal:cwd` | `session_id, cwd` | OSC 7 |
| `terminal:bell` | `session_id` | BEL (0x07) |
| `terminal:hyperlink` | `session_id, url, id?` | OSC 8 |
| `terminal:clipboard-write` | `session_id, data` | OSC 52 (termwiz decodes base64) |
| `terminal:exit` | `session_id, code` | shell exit |
| `agent:hello` | `session_id, name, version, pid?` | OSC 7777 agent.hello |
| `agent:tool_start` | `session_id, id, name, summary, icon?, parent_id?` | OSC 7777 tool.start |
| `agent:tool_progress` | `session_id, id, pct?, message?` | OSC 7777 tool.progress |
| `agent:tool_end` | `session_id, id, status, summary?, duration_ms?` | OSC 7777 tool.end |
| `profiles:updated` | — | `notify` watcher on profiles dir |
| `config:updated` | — | `notify` watcher on config.toml |

---

## OSC 7777 Quick Reference

**Wire format:** `ESC ] 7777 ; <verb> ; <json-payload> ESC \`

| Verb | Direction | Purpose |
|---|---|---|
| `agent.hello` | agent → terminal | announce name / version / pid |
| `agent.query` | agent → terminal | query terminal capabilities |
| `tool.start` | agent → terminal | begin tool call (id, name, summary, args) |
| `tool.progress` | agent → terminal | progress update (pct, message) |
| `tool.end` | agent → terminal | finish tool call (status: ok / error / cancelled) |
| `diff` | agent → terminal | show inline diff viewer (path, patch, actions) |
| `approval` | agent → terminal | show approval sheet (title, detail, risk, actions) |
| `block.annotate` | agent → terminal | tag current OSC 133 block |
| `session.mark` | agent → terminal | insert checkpoint divider |
| `reply` | terminal → agent | response to query / diff / approval (id + result) |
| `event` | terminal → agent | terminal-initiated event (e.g. tool.cancel) |

**Response channel:** terminal writes `ESC ] 7777 ; reply ; <json> ESC \` **to PTY stdin** — agent reads it as normal stdin. This is the same pattern as DSR (device status report) replies.

**TERM_OSC_7777=1** is set in the shell environment by `pty.rs` so agents can feature-detect support without a query round-trip.

---

## Key Conventions

- Rust structs are the source of truth for IPC types — TypeScript types are derived
- OSC 7777 spec is CC-BY-4.0 — keep verbs vendor-neutral (no EmberTerm-only verbs in the public spec)
- Themes use VS Code-compatible JSON — do not invent a new format
- Config and profiles live in `~/.config/ember/` on all platforms
- `SessionId` is a `String` UUID — never expose PTY handles or OS pids to the frontend
- The `--bg-elev` CSS variable naming convention from `references/design.html` is canonical — not `--bg-elevated`

---

## Dev Commands

Quick bash reference:

```bash
# Type-check (fast, no build)
cargo check --manifest-path src-tauri/Cargo.toml
cd frontend && npm run check

# Tests
cargo test --manifest-path src-tauri/Cargo.toml
cd frontend && npm test

# Lint
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check

# Full build
cd frontend && npm run build
cargo build --manifest-path src-tauri/Cargo.toml
```

Custom slash commands (defined in `.claude/commands/`):
- `/check` — type-check Rust + TS/Svelte without building
- `/test [filter]` — run Rust + Vitest test suites
- `/build` — full frontend + Rust build
- `/lint` — clippy + fmt check + svelte-check
- `/task [query]` — look up open tasks in TASKS.md
- `/ui-review [component]` — compare component against design mockup
