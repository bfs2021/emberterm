# EmberTerm — Development Plan

> A modern, lightweight, agent-native terminal emulator.
> Built on the WezTerm engine, wrapped in a Tabby-grade UI, designed from
> day one to work natively with AI coding agents.

**Status:** Pre-v0.1 (Planning)
**Last Updated:** 2026-05-05
**Revision:** 2 (open questions resolved: framework, response channel, theme format, font shaping, telemetry)
**Document Owner:** Project Lead

---

## Table of Contents

1. [Product Overview](#1-product-overview)
2. [Product Specification](#2-product-specification)
3. [Architecture](#3-architecture)
4. [Technical Design](#4-technical-design)
5. [OSC 7777 — Agent Integration Protocol](#5-osc-7777--agent-integration-protocol)
6. [UI/UX Design](#6-uiux-design)
7. [Product Flows](#7-product-flows)
8. [Development Plan](#8-development-plan)
9. [References](#9-references)
10. [Risks & Open Questions](#10-risks--open-questions)
11. [Glossary](#11-glossary)

---

## 1. Product Overview

### 1.1 Vision

A terminal emulator that treats AI coding agents as first-class citizens —
not as just another program emitting bytes — while remaining lightweight,
fast, and beautiful.

### 1.2 One-Sentence Pitch

> EmberTerm is the terminal built around how developers actually work with AI
> agents — open-source, lightweight, account-free, with a published protocol
> that any agent can adopt.

### 1.3 Target Audience

| Segment | Description |
|---|---|
| **Primary** | Developers who use Claude Code, Codex CLI, OpenCode, Aider, or similar agents daily and want a polished, agent-aware host terminal. |
| **Secondary** | Developers tired of Tabby's weight or Cmder's outdated UI but who don't want a power-user tool like WezTerm with no GUI config. |
| **Not the audience** | Terminal connoisseurs who want a Lua-scriptable multiplexer with SSH and Sixel — they should keep using WezTerm. |

### 1.4 Positioning

EmberTerm is **not** competing with WezTerm on terminal-engine quality.
EmberTerm uses WezTerm's engine (`wezterm-term`, `termwiz`, `portable-pty`)
and builds a different product on top.

The relationship is analogous to **VS Code → Cursor**: same editor core,
different product, different audience, different UX layer.

#### Competitive Map

| Product | Engine | Weight | UI | Agent-Aware | Account |
|---|---|---|---|---|---|
| **Cmder** | ConEmu (legacy) | ~50 MB | Outdated | No | No |
| **ConEmu** | Native C++ | ~10 MB | Win32 / outdated | No | No |
| **Tabby** | xterm.js + Electron | ~150 MB | Modern, polished | No | No |
| **Warp** | Custom (Rust) | ~120 MB | Modern, blocks | Partial / closed | **Yes** |
| **WezTerm** | Native Rust + GPU | ~25 MB | Functional, Lua config | No | No |
| **Windows Terminal** | Custom (C++) | ~60 MB | Clean, modern | No | No |
| **Ghostty** | Native Zig + GPU | ~15 MB | Minimal | No | No |
| **EmberTerm** | wezterm-term + Tauri | **Target ~20 MB** | Tabby-grade | **Yes (OSC 7777)** | No |

### 1.5 Value Proposition

1. **Agent-native UX** — inline tool cards, diff viewers, approval sheets,
   block-based scrollback. No other open-source terminal has these.
2. **Tabby-quality polish** at ~10× less weight (Tauri replaces Electron).
3. **Open agent protocol (OSC 7777)** — agents can opt in without
   coordinating with us. Becomes a defensible ecosystem moat over time.
4. **No account, no telemetry by default, no cloud lock-in** — unlike Warp.
5. **Built on WezTerm** — the terminal core is rock solid from day one.

### 1.6 Key Differentiators

- **Block-based UI** — every command + output is a navigable, foldable,
  shareable unit. Powered by OSC 133 prompt marks.
- **OSC 7777 protocol** — standardized agent integration: tool calls,
  diffs, approvals, session marks.
- **Native diff viewer** — inline patch rendering with accept/reject buttons.
- **Approval sheets** — destructive commands get a native UI overlay,
  not the agent's TUI prompt.
- **Terminal-as-MCP-server** — exposes cwd, recent blocks, git state to
  agents via MCP without parsing scrollback.
- **GUI-first config** — settings panel, command palette, profile manager.
  TOML config is optional, not required.

---

## 2. Product Specification

### 2.1 Core Terminal Features

| Feature | v0.1 | v0.2 | v1.0 |
|---|---|---|---|
| ANSI/VT100 compliance | ✅ (via wezterm-term) | — | — |
| Truecolor (24-bit) | ✅ | — | — |
| Bracketed paste | ✅ | — | — |
| OSC 8 hyperlinks | ✅ | — | — |
| OSC 52 clipboard | ✅ | — | — |
| OSC 7 working directory | ✅ | — | — |
| OSC 133 prompt marks | ✅ (parsed) | ✅ (block UI) | — |
| Synchronized output (DEC 2026) | ✅ | — | — |
| Kitty keyboard protocol | — | ✅ | — |
| Mouse (SGR) | ✅ | — | — |
| SGR Pixel mouse | — | ✅ | — |
| Sixel images | — | — | ✅ |
| Kitty graphics protocol | — | ✅ | — |
| iTerm2 inline images | — | — | ✅ |
| Ligatures | — | ✅ | — |

### 2.2 UI Features

| Feature | v0.1 | v0.2 | v1.0 |
|---|---|---|---|
| Tabs (drag-reorder, rename, close) | ✅ | — | — |
| Custom title bar w/ integrated tabs | ✅ | — | — |
| Settings UI (GUI, not file-only) | ✅ | — | — |
| Theme system (built-in themes + custom) | ✅ | — | — |
| Command palette (`Ctrl+Shift+P`) | ✅ | — | — |
| Profile manager | ✅ | — | — |
| Status bar (cwd, git, agent indicator) | ✅ | — | — |
| Acrylic / Mica / Vibrancy | ✅ | — | — |
| Split panes (horizontal/vertical) | — | ✅ | — |
| Quake mode (drop-down, global hotkey) | — | ✅ | — |
| Tray icon | — | ✅ | — |
| Block-based scrollback UI | — | ✅ | — |
| Smart scrollback search | — | — | ✅ |
| Session save/restore | — | — | ✅ |

### 2.3 Agent Features

| Feature | v0.1 | v0.2 | v1.0 |
|---|---|---|---|
| OSC 7777 parser | ✅ (stub) | ✅ (full) | — |
| Tool call inline cards | ✅ (basic) | ✅ (full) | — |
| Diff viewer (inline patches) | — | ✅ | — |
| Approval sheets | — | ✅ | — |
| Agent indicator in status bar | ✅ | — | — |
| Session marks / checkpoints | — | ✅ | — |
| Terminal-as-MCP-server | — | — | ✅ |
| Multi-agent session view | — | — | ✅ |

### 2.4 Performance Targets

| Metric | Target | Notes |
|---|---|---|
| Binary size (installer) | < 25 MB | Tauri baseline ~10 MB + UI assets |
| Cold start | < 400 ms | Window visible, ready for input |
| Idle RAM | < 80 MB | Single tab, no agent connected |
| RAM per additional tab | < 30 MB | xterm.js + per-PTY buffers |
| Throughput (cat large file) | > 500 MB/s rendered | wezterm-term handles parsing |
| Input latency (key → glyph) | < 16 ms (1 frame @ 60Hz) | xterm.js limitation in v0.1 |

### 2.5 Platform Support

| Platform | v0.1 | v0.2 | v1.0 |
|---|---|---|---|
| Windows 10 (1809+) | ✅ | — | — |
| Windows 11 | ✅ | — | — |
| macOS 12+ | ✅ | — | — |
| Linux (X11) | — | ✅ | — |
| Linux (Wayland) | — | — | ✅ |

ConPTY requires Windows 10 1809+. We do not support older Windows.

---

## 3. Architecture

### 3.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                       EmberTerm Window                      │
│  ┌───────────────────────────────────────────────────────┐  │
│  │              Custom Title Bar + Tab Strip             │  │
│  │  [+] [tab 1] [tab 2 *] [tab 3]              [_][□][X] │  │
│  ├───────────────────────────────────────────────────────┤  │
│  │                                                       │  │
│  │            Terminal Grid (xterm.js v0.1 →             │  │
│  │            native wgpu in v0.2+)                      │  │
│  │                                                       │  │
│  │     [agent: tool card]  [agent: diff viewer]          │  │
│  │                                                       │  │
│  ├───────────────────────────────────────────────────────┤  │
│  │  Status Bar: zsh ● ~/code/proj (main) ● claude-code   │  │
│  └───────────────────────────────────────────────────────┘  │
│           ↑ webview                       ↑ overlays        │
└─────────────────────────────────────────────────────────────┘
                           ⇅ IPC (Tauri events + commands)
┌─────────────────────────────────────────────────────────────┐
│                  Rust Core (packages/desktop)                │
│                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌──────────────────┐     │
│  │  Terminal   │  │     PTY     │  │  OSC Parser /    │     │
│  │  Sessions   │←→│  Manager    │  │  OSC 7777 ext.   │     │
│  │ (wezterm-   │  │ (portable-  │  │   (termwiz +     │     │
│  │   term)     │  │   pty)      │  │   custom)        │     │
│  └─────────────┘  └─────────────┘  └──────────────────┘     │
│         ↑                 ↑                    ↑            │
│         └─────────────────┴────────────────────┘            │
│                           ↓                                 │
│  ┌─────────────┐  ┌─────────────┐  ┌──────────────────┐     │
│  │   Block     │  │   Agent     │  │   Profile /      │     │
│  │   Model     │  │   State     │  │   Config         │     │
│  └─────────────┘  └─────────────┘  └──────────────────┘     │
│                                                             │
│             AppState (Tabs, Sessions, Config)               │
└─────────────────────────────────────────────────────────────┘
                           ⇅
                    Shells / Agents
                    (cmd, pwsh, bash, zsh, claude-code, ...)
```

### 3.2 Project Structure

```
emberterm/
├── Cargo.toml                    ← Cargo workspace root
├── package.json                  ← pnpm workspace root + turbo scripts
├── pnpm-workspace.yaml           ← declares packages/* + e2e
├── turbo.json                    ← build/test task graph
├── PLAN.md                       ← this document
├── README.md
├── LICENSE                       ← MIT
├── .github/
│   └── workflows/
│       ├── ci.yml
│       └── release.yml
├── packages/
│   ├── desktop/                  ← Rust core (Tauri 2 backend)
│   │   ├── src/
│   │   │   ├── main.rs               ← Tauri app entry
│   │   │   ├── terminal/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── session.rs        ← wraps wezterm_term::Terminal
│   │   │   │   ├── pty.rs            ← portable-pty wrapper
│   │   │   │   ├── parser.rs         ← OSC 7777 hooks into termwiz
│   │   │   │   └── blocks.rs         ← OSC 133 → Block model
│   │   │   ├── agent/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── protocol.rs       ← OSC 7777 verb definitions
│   │   │   │   ├── tools.rs          ← active tool-call tracking
│   │   │   │   └── handlers.rs       ← verb → IPC event dispatch
│   │   │   ├── profile.rs            ← TOML profiles + hot reload
│   │   │   ├── config.rs             ← Global config (theme, font, etc.)
│   │   │   ├── ipc.rs                ← Tauri command handlers
│   │   │   ├── state.rs              ← AppState
│   │   │   └── platform/
│   │   │       ├── windows.rs        ← Acrylic/Mica, ConPTY specifics
│   │   │       ├── macos.rs          ← Vibrancy, NSWindow tweaks
│   │   │       └── linux.rs          ← (v0.2+)
│   │   ├── tauri.conf.json
│   │   ├── Cargo.toml
│   │   └── build.rs
│   ├── app/                      ← Webview UI (Svelte 5 + TS)
│   │   ├── src/
│   │   │   ├── main.ts
│   │   │   ├── App.svelte
│   │   │   ├── ipc/
│   │   │   │   ├── client.ts         ← Tauri command bindings
│   │   │   │   └── events.ts         ← event subscriptions
│   │   │   ├── terminal/
│   │   │   │   └── Grid.svelte       ← xterm.js wrapper
│   │   │   ├── chrome/
│   │   │   │   ├── TitleBar.svelte
│   │   │   │   ├── TabStrip.svelte
│   │   │   │   └── StatusBar.svelte
│   │   │   ├── agent/
│   │   │   │   ├── ToolCard.svelte
│   │   │   │   ├── DiffView.svelte      ← v0.2
│   │   │   │   └── ApprovalSheet.svelte ← v0.2
│   │   │   ├── palette/
│   │   │   │   └── CommandPalette.svelte
│   │   │   ├── settings/
│   │   │   │   ├── SettingsPanel.svelte
│   │   │   │   ├── ThemeEditor.svelte
│   │   │   │   └── ProfileEditor.svelte
│   │   │   ├── theme/
│   │   │   │   ├── tokens.ts            ← design tokens
│   │   │   │   ├── vscode-import.ts     ← VS Code theme converter
│   │   │   │   └── themes/
│   │   │   │       ├── ember-dark.json     ← VS Code-format JSON
│   │   │   │       ├── tokyo-night.json
│   │   │   │       └── catppuccin.json
│   │   │   └── stores/
│   │   │       └── app.ts               ← Svelte stores for UI state
│   │   ├── index.html
│   │   ├── package.json
│   │   ├── tsconfig.json
│   │   └── vite.config.ts
│   ├── ui/                       ← Shared Svelte component library
│   │   ├── src/
│   │   │   ├── lib/
│   │   │   │   └── Btn.svelte
│   │   │   └── index.ts              ← barrel export
│   │   ├── package.json
│   │   └── vite.config.ts
│   └── sdk/                      ← OSC 7777 TypeScript SDK (zero deps)
│       ├── src/
│       │   ├── types.ts              ← mirrors agent/protocol.rs structs
│       │   ├── emit.ts               ← emitHello/emitToolStart/etc.
│       │   └── index.ts              ← barrel export
│       ├── package.json
│       └── tsconfig.json
├── e2e/                          ← End-to-end tests (WebdriverIO + tauri-driver)
├── docs/
│   ├── osc-7777-spec.md          ← public protocol spec
│   ├── architecture.md
│   ├── contributing.md
│   └── theming.md
└── examples/
    ├── profiles/                  ← example profile TOMLs
    └── agents/                    ← example agent integrations
```

### 3.3 Process & Threading Model

- **Main thread (Rust):** Tauri runtime, IPC dispatch, AppState owner.
- **PTY reader threads:** one per terminal session, reads bytes from PTY,
  feeds them into `wezterm_term::Terminal` parser, emits IPC events.
- **PTY writer:** synchronous from IPC command handlers (keystrokes are tiny).
- **Webview process:** OS webview (WebView2 / WKWebView / WebKitGTK), runs UI.
- **Async runtime:** `tokio` for I/O; `wezterm-term` is sync but cheap to drive
  from a tokio task.

### 3.4 Rendering Strategy

#### v0.1: xterm.js in webview

- PTY bytes → Rust parser → emit `terminal:output` event with raw bytes
- Webview xterm.js consumes bytes, renders via canvas/WebGL
- **Pros:** ship in weeks, proven library, easy theming
- **Cons:** webview overhead (~5-10 MB extra RAM), latency floor ~16ms

#### v0.2+: Native wgpu rendering

- Rust renders the terminal grid to a native window or a texture surface
- Tauri webview is positioned as overlay/sibling for chrome only
- Inspired by Warp / Ghostty's approach
- **Pros:** sub-frame latency, GPU-accelerated, lower RAM
- **Cons:** months of work, font shaping is hard, platform window hacks

**Decision:** Ship v0.1 with xterm.js. Keep the IPC contract minimal so the
grid is swappable. Migrate in v0.2 only after v0.1 is in users' hands.

---

## 4. Technical Design

### 4.1 Tech Stack

#### Rust Core

| Concern | Choice | Reason |
|---|---|---|
| App framework | **Tauri 2.x** | Native webview, ~10× lighter than Electron |
| Async runtime | **tokio** | De facto standard, fits Tauri |
| Terminal model | **`wezterm-term`** | Battle-tested, full ANSI/VT |
| Terminal capabilities | **`termwiz`** | Cell types, escape parsing, surfaces |
| PTY | **`portable-pty`** | ConPTY + Unix PTY, one API |
| Serialization | **`serde` + `serde_json`** | IPC payloads |
| TOML parsing | **`toml`** | Profiles, config |
| File watching | **`notify`** | Hot-reload config / profiles |
| Logging | **`tracing` + `tracing-subscriber`** | Structured logs |
| Error handling | **`thiserror` + `anyhow`** | Lib + bin idioms |

#### Webview / Frontend

| Concern | Choice | Reason |
|---|---|---|
| Framework | **Svelte 5 + TypeScript** | Compiler-based, ~10× smaller bundle than React, fine-grained reactivity ideal for streaming agent updates |
| Build tool | **Vite** | Fast, default for Tauri scaffolds, first-class Svelte support |
| Terminal renderer | **xterm.js** | v0.1 only; replace with native in v0.2 |
| xterm addons | `xterm-addon-fit`, `xterm-addon-web-links`, `xterm-addon-ligatures`, `xterm-addon-webgl` | Standard fare |
| State management | **Svelte stores + runes** | Built into Svelte 5, no extra dependency |
| Styling | **Tailwind CSS + CSS variables** | Theme tokens via CSS vars, utility classes elsewhere |
| Component primitives | **`bits-ui`** (or `melt-ui`) | Headless, accessible primitives — Svelte equivalent of Radix UI |
| Icons | **`lucide-svelte`** | Clean, MIT, large set |
| Type-safe IPC | **`ts-rs`** (Rust → TS codegen) | One source of truth for types |

### 4.2 Key Data Models

#### Session

```rust
pub struct Session {
    pub id: SessionId,
    pub profile_id: ProfileId,
    pub terminal: wezterm_term::Terminal,
    pub pty_pair: portable_pty::PtyPair,
    pub child: Box<dyn portable_pty::Child + Send>,
    pub cwd: Option<PathBuf>,
    pub title: String,
    pub blocks: Vec<Block>,
    pub active_tools: HashMap<ToolId, ToolCallState>,
    pub agent: Option<AgentInfo>,
}
```

#### Block (OSC 133-driven)

```rust
pub struct Block {
    pub id: BlockId,
    pub command: String,
    pub started_at: Instant,
    pub ended_at: Option<Instant>,
    pub exit_code: Option<i32>,
    pub output_byte_range: Range<usize>,  // into session scrollback
    pub annotations: BlockAnnotations,
    pub tool_calls: Vec<ToolId>,
}

pub struct BlockAnnotations {
    pub agent_id: Option<String>,
    pub tags: Vec<String>,
    pub fold_default: FoldMode,
}
```

#### Profile

```toml
# ~/.config/ember/profiles/default.toml
id = "default"
name = "Default Shell"
icon = "terminal"
shell = "pwsh.exe"          # or full path
args = ["-NoLogo"]
cwd = "~"                   # supports ~ expansion
env = { TERM = "ember" }
theme = "ember-dark"        # optional override
font_family = "JetBrainsMono Nerd Font"
font_size = 13
```

#### Global Config

```toml
# ~/.config/ember/config.toml
[appearance]
theme = "ember-dark"
font_family = "JetBrainsMono Nerd Font"
font_size = 13
window_padding = 8
acrylic = true              # Windows 11 Mica when supported
opacity = 0.95

[behavior]
default_profile = "default"
confirm_close_running = true
scrollback_lines = 10000

[keymap]
"Ctrl+Shift+T" = "tab.new"
"Ctrl+Shift+W" = "tab.close"
"Ctrl+Shift+P" = "palette.open"
"Ctrl+,"       = "settings.open"

[agent]
osc_7777_enabled = true
auto_accept_tool_calls = false
mcp_server_enabled = false  # v1.0
```

### 4.3 IPC Contract

#### Tauri Commands (webview → Rust, request/response)

```typescript
// generated by ts-rs
type Command =
  | { type: "session.create"; profile_id: string }       // → SessionId
  | { type: "session.write"; id: SessionId; bytes: Uint8Array }
  | { type: "session.resize"; id: SessionId; cols: number; rows: number }
  | { type: "session.close"; id: SessionId }
  | { type: "agent.respond"; action_id: string; result: AgentResult }
  | { type: "block.get"; id: BlockId }
  | { type: "block.search"; query: string; filters?: BlockFilter }
  | { type: "profile.list" }
  | { type: "profile.upsert"; profile: Profile }
  | { type: "config.get" }
  | { type: "config.update"; patch: Partial<Config> };
```

#### Tauri Events (Rust → webview, fire-and-forget)

```typescript
type Event =
  | { type: "terminal.output"; session_id: SessionId; bytes: Uint8Array }
  | { type: "terminal.title"; session_id: SessionId; title: string }
  | { type: "terminal.cwd"; session_id: SessionId; cwd: string }
  | { type: "terminal.bell"; session_id: SessionId }
  | { type: "block.start"; session_id: SessionId; block: Block }
  | { type: "block.end"; session_id: SessionId; block: Block }
  | { type: "agent.hello"; session_id: SessionId; info: AgentInfo }
  | { type: "agent.tool_start"; session_id: SessionId; tool: ToolCall }
  | { type: "agent.tool_progress"; session_id: SessionId; id: ToolId; pct?: number; message?: string }
  | { type: "agent.tool_end"; session_id: SessionId; id: ToolId; status: ToolStatus }
  | { type: "agent.diff"; session_id: SessionId; diff: DiffPayload }
  | { type: "agent.approval"; session_id: SessionId; approval: ApprovalPayload }
  | { type: "session.exit"; session_id: SessionId; code: i32 };
```

---

## 5. OSC 7777 — Agent Integration Protocol

### 5.1 Goals

- **Open** — any agent can target it without coordinating with EmberTerm.
- **Versioned** — each verb carries a payload version field.
- **Graceful degradation** — agents that don't speak it still work fine.
- **Single namespace** — all verbs live under OSC 7777.

### 5.2 Wire Format

```
ESC ] 7777 ; <verb> ; <json-payload> ESC \
```

`<verb>` is a dotted identifier (e.g. `tool.start`).
`<json-payload>` is a single-line JSON object. Newlines must be encoded.

### 5.3 Verbs (v0.1 of the spec)

#### `agent.hello`

Agent announces itself on startup.

```json
{ "v": 1, "name": "claude-code", "version": "1.0.0", "pid": 12345 }
```

#### `agent.query`

Agent asks the terminal for capabilities.

```json
{ "v": 1, "id": "q1", "ask": "capabilities" }
```

Terminal replies via stdin:

```json
{ "v": 1, "id": "q1", "result": {
    "osc_7777": ["tool.*", "diff", "approval", "block.annotate", "session.mark"],
    "graphics": ["kitty"],
    "blocks": true
}}
```

#### `tool.start` / `tool.progress` / `tool.end`

```json
{ "v": 1, "id": "t_42", "name": "edit_file",
  "summary": "Edit src/main.rs",
  "args": { "path": "src/main.rs" } }

{ "v": 1, "id": "t_42", "pct": 60, "message": "Applying patch..." }

{ "v": 1, "id": "t_42", "status": "ok",
  "summary": "Edited 12 lines" }
```

`status`: `"ok"` | `"error"` | `"cancelled"`

#### `diff`

```json
{ "v": 1, "id": "d_1", "path": "src/main.rs",
  "patch": "<unified diff>",
  "actions": ["accept", "reject", "edit"] }
```

Terminal renders inline diff viewer with action buttons.
On user action, terminal sends response:

```json
{ "v": 1, "id": "d_1", "result": "accept" }
```

#### `approval`

```json
{ "v": 1, "id": "a_1",
  "title": "Run shell command",
  "detail": "rm -rf node_modules",
  "risk": "destructive",
  "actions": ["allow_once", "allow_session", "deny"] }
```

`risk`: `"safe"` | `"caution"` | `"destructive"`

#### `block.annotate`

```json
{ "v": 1, "tags": ["agent", "tool-call"],
  "agent_id": "claude-code", "fold": "auto" }
```

`fold`: `"auto"` | `"open"` | `"closed"`

Annotates the *current* block (the one in progress).

#### `session.mark`

```json
{ "v": 1, "label": "Refactor complete", "kind": "checkpoint" }
```

### 5.4 Response Channel

Terminal responses use the **same OSC 7777 namespace, written back into the
PTY's input stream** (the side that the agent reads as stdin). This is the
same mechanism terminals have always used to reply to DSR (Device Status
Report) and other CSI/OSC queries — agents already understand the pattern.

**Wire format (terminal → agent):**

```
ESC ] 7777 ; reply ; <json-payload> ESC \
```

**Example:**

```
ESC ] 7777 ; reply ; {"v":1,"id":"d_1","result":"accept"} ESC \
```

The agent reads its stdin, recognizes the `ESC ] 7777 ; reply ; ` prefix,
parses the JSON payload, and matches it to a pending request by `id`. Any
bytes outside an OSC 7777 sequence are passed through as normal stdin input.

**Why this design over alternatives:**

| Option | Verdict |
|---|---|
| Stdin sentinel prefix (e.g. `\x1bember:...`) | Custom, non-standard, agents need bespoke parsers. |
| Named pipe / Unix socket | Platform-specific code, complicates deployment. |
| **OSC 7777 reply on PTY input** | **Symmetric protocol, zero platform-specific code, parser is already needed for outgoing OSC.** |

**Reply verbs:**

- `reply` — generic response, payload includes `id` and `result`
- `event` — terminal-initiated event (e.g., user pressed cancel hotkey on a tool card)

**Example: user-initiated cancel:**

```
ESC ] 7777 ; event ; {"v":1,"kind":"tool.cancel","id":"t_42"} ESC \
```

This lets the user kill an in-flight tool call from the terminal UI; the
agent receives the event and can stop the operation cleanly.

### 5.5 Resolved Spec Decisions

- ✅ **Response channel:** OSC 7777 reply on PTY input (above)
- ✅ **No sentinel prefix needed** — OSC sequence boundaries are unambiguous
- 🟡 **Nested tool calls:** `tool.start` accepts optional `parent_id`; defer
  rendering nuances to v0.2
- 🟡 **Multi-file diffs:** v0.1 spec supports one path per `diff` verb;
  agents emit multiple `diff` verbs for multi-file changes; v0.2 may add
  a `diff.batch` verb if real-world usage demands it

---

## 6. UI/UX Design

### 6.1 Design Principles

1. **Terminal first, chrome second.** UI never steals focus or vertical space
   from the grid by default.
2. **Agent affordances inline.** Tool cards, diffs, approvals appear *with*
   the conversation, not in a side drawer the user must check.
3. **Keyboard-first, mouse-friendly.** Every action has a hotkey; nothing
   *requires* the mouse.
4. **Sane defaults, zero-config first run.** A new install should look
   beautiful before the user opens settings.
5. **Familiar where possible, novel where it matters.** Tabs and palette work
   the way VS Code users expect; agent affordances are new because they
   have to be.

### 6.2 Visual Identity

- **Name:** EmberTerm
- **Binary:** `ember`
- **Tagline:** *Where shells and agents meet.*
- **Primary accent color:** warm ember orange (`#FF6B35` approx)
- **Default theme:** *Ember Dark* — deep charcoal background, warm accent,
  high-contrast foreground, subtle ember-orange highlights for agent UI.

### 6.3 Layout

```
┌──────────────────────────────────────────────────────────────┐
│ [ ember icon ] [ tab1 ] [ tab2● ] [ tab3 ] [+]    _ □ X      │  ← title bar (28px)
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  $ npm install                                               │  ← block 1 (collapsed)
│  ▸ 234 packages installed in 4.2s                            │
│                                                              │
│  $ claude-code                                               │  ← block 2 (active)
│  ┌──────────────────────────────────────────────────┐        │
│  │ 🔧 edit_file · src/main.rs              [▾]      │        │  ← tool card
│  │   Edited 12 lines · 1.2s                         │        │
│  └──────────────────────────────────────────────────┘        │
│                                                              │
│  > _                                                         │  ← active prompt
│                                                              │
├──────────────────────────────────────────────────────────────┤
│  zsh ● ~/code/ember (main *) ● claude-code v1.0      14:32   │  ← status bar (24px)
└──────────────────────────────────────────────────────────────┘
```

### 6.4 Component Library

#### Title Bar
- Custom-drawn, integrates tabs (no native title bar above)
- Tab: rounded-top, drag-reorder, close-on-middle-click, double-click rename
- "+" button opens new-tab dropdown (profile picker)
- Window controls: native-style, OS-correct positioning (Windows right, macOS left)

#### Tab
- States: `active` | `inactive` | `unread` | `running` | `agent-active`
- Visual cues:
  - `active`: full opacity, accent underline
  - `unread`: dot indicator
  - `running`: subtle pulse animation
  - `agent-active`: ember-orange dot

#### Tool Card (inline)
- Collapsed: one line — icon, tool name, summary, status
- Expanded: arguments JSON, output, duration, [retry]/[cancel]
- States: `pending` | `running` | `ok` | `error`
- Color coding: subtle, status-driven

#### Diff Viewer (inline)
- Header: file path, +N/-N stats
- Body: unified or side-by-side (toggle)
- Footer: [Accept] [Reject] [Edit] (keyboard: y/n/e)
- Syntax highlighting via Shiki or PrismJS

#### Approval Sheet
- Slides up from bottom of active pane
- Header: title + risk badge
- Body: detail text (monospace if command)
- Footer: action buttons, leftmost = safest
- Keyboard: arrow keys + Enter, Esc = deny

#### Command Palette
- Triggered by `Ctrl+Shift+P` / `Cmd+Shift+P`
- Centered, ~600px wide, vertically constrained
- Sections: Recent, Commands, Profiles, Themes, Settings
- Fuzzy search via `fzf`-style algorithm

#### Settings Panel

Modelled after Tabby terminal: icon + label left nav, 820×580px overlay, six well-scoped categories.

**Panel layout:**
- Left nav: 200px, icon (12px SVG) + label per category
- Right content pane: flex-1, scrollable
- Search box at top of nav filters across all categories

**Categories:**

| # | Category | Content |
|---|---|---|
| 1 | **Appearance** | Font family/size/line-height; window padding; Acrylic/Mica toggle (Windows 11); opacity slider |
| 2 | **Profiles** | Default-profile dropdown; profile list (user-created / built-in sections); `ProfileEditor` for create/edit |
| 3 | **Terminal** | Cursor shape (block/bar/underline segmented button); cursor blink toggle; scrollback lines; bold-in-bright toggle; confirm-close toggle |
| 4 | **Color Scheme** | Theme browser with live preview swatches; searchable built-in list; VS Code JSON import |
| 5 | **Keymap** | Searchable table of all shortcuts and bindings (read-only display in v0.2; configurable in v1.0) |
| 6 | **Agent** | Enable OSC 7777 toggle; auto-accept safe tool calls toggle; spec version info |

**New Rust config fields** (added to `AppearanceConfig`):
- `cursor_style: String` — `"block"` | `"bar"` | `"underline"` (default `"block"`)
- `cursor_blink: bool` (default `true`)
- `bold_in_bright: bool` (default `true`)

**New Svelte components:**
- `AppearanceSettings.svelte` — font + window controls (replaces the font section previously in ThemeEditor)
- `TerminalSettings.svelte` — cursor + scrollback + behavior toggles
- `AgentSettings.svelte` — agent config toggles
- `ThemeEditor.svelte` — trimmed to color scheme selection + import only

**Data flow:** All changes go through the existing `configUpdate` JSON merge patch IPC command. A new `terminalConfigStore.ts` bridges `appearance.cursor_*` and `behavior.scrollback_lines` to live xterm.js option updates in `Grid.svelte`.

### 6.5 Theme System

#### Format: VS Code-Compatible JSON

EmberTerm consumes **VS Code theme JSON directly**. This unlocks the entire
VS Code theme ecosystem (thousands of themes) on day one.

**Why VS Code format:**
- Massive existing ecosystem
- Well-documented schema
- Users can paste any VS Code theme and it works
- Theme authors don't need to learn a new format

#### Schema (subset we consume)

```jsonc
{
  "name": "Ember Dark",
  "type": "dark",
  "colors": {
    // Terminal palette (we consume all of these)
    "terminal.background": "#0d1117",
    "terminal.foreground": "#e6edf3",
    "terminal.ansiBlack": "#484f58",
    "terminal.ansiRed": "#ff7b72",
    "terminal.ansiGreen": "#7ee787",
    "terminal.ansiYellow": "#d29922",
    "terminal.ansiBlue": "#58a6ff",
    "terminal.ansiMagenta": "#bc8cff",
    "terminal.ansiCyan": "#39c5cf",
    "terminal.ansiWhite": "#b1bac4",
    "terminal.ansiBrightBlack": "#6e7681",
    "terminal.ansiBrightRed": "#ffa198",
    // ... bright variants ...
    "terminalCursor.foreground": "#e6edf3",
    "terminalCursor.background": "#0d1117",

    // UI chrome (we map these to our chrome tokens)
    "editor.background": "#0d1117",          // → ui.bg
    "sideBar.background": "#010409",         // → ui.bgElevated
    "sideBar.border": "#21262d",             // → ui.border
    "tab.activeBackground": "#0d1117",       // → tab active bg
    "tab.inactiveBackground": "#010409",     // → tab inactive bg
    "tab.activeForeground": "#e6edf3",       // → tab active fg
    "statusBar.background": "#010409",       // → status bar bg
    "statusBar.foreground": "#7d8590",       // → status bar fg
    "focusBorder": "#1f6feb",                // → ui.accent
    "errorForeground": "#f85149"             // → ui.danger
  }
}
```

`tokenColors` (syntax highlighting) is parsed but only used in the inline
diff viewer for code rendering — main terminal output uses ANSI colors.

#### Importing Themes

Three paths:

1. **Built-in browser** — Settings → Themes browses a curated list of VS
   Code themes we've vetted (tokyo-night, catppuccin, dracula, etc.)
2. **Paste JSON** — Settings → Themes → Import → paste any VS Code theme JSON
3. **Local file** — drop `.json` file into `~/.config/ember/themes/`

#### Built-in Themes (v0.1)

All shipped as VS Code-format JSON files in `packages/app/src/theme/themes/`:

1. **Ember Dark** (default, custom)
2. **Ember Light** (custom)
3. **Tokyo Night** (community, included with attribution)
4. **Catppuccin Mocha** (community)
5. **Dracula** (community)
6. **GitHub Dark** (community)
7. **Solarized Dark** (community)

#### Internal Theme Type

The Rust core stores a normalized theme:

```rust
pub struct Theme {
    pub id: String,
    pub name: String,
    pub kind: ThemeKind,           // Dark | Light
    pub ansi: AnsiPalette,         // 16 ANSI + bright
    pub terminal: TerminalColors,  // fg, bg, cursor, selection
    pub ui: UiColors,              // chrome tokens
    pub effects: Effects,          // acrylic, opacity
}
```

A `vscode_to_theme()` converter handles the mapping documented above.

### 6.6 Keyboard Shortcuts (defaults)

| Action | Windows/Linux | macOS |
|---|---|---|
| New tab | `Ctrl+Shift+T` | `Cmd+T` |
| Close tab | `Ctrl+Shift+W` | `Cmd+W` |
| Next/prev tab | `Ctrl+Tab` / `Ctrl+Shift+Tab` | `Cmd+Opt+→/←` |
| Command palette | `Ctrl+Shift+P` | `Cmd+Shift+P` |
| Settings | `Ctrl+,` | `Cmd+,` |
| Copy / Paste | `Ctrl+Shift+C/V` | `Cmd+C/V` |
| Find in scrollback | `Ctrl+Shift+F` | `Cmd+F` |
| Approval: allow / deny | `Y` / `N` | `Y` / `N` |
| Diff: accept / reject / edit | `Y` / `N` / `E` | `Y` / `N` / `E` |
| Toggle Quake mode (v0.2) | `Ctrl+`\` (configurable) | `Cmd+`\` |

### 6.7 Accessibility

- WCAG AA contrast minimum on all themes
- Screen reader: terminal grid is opaque to AT, but chrome (tabs, palette,
  settings, agent UI) uses `bits-ui` primitives which are AT-friendly
- Keyboard-only navigation through all chrome
- Respects `prefers-reduced-motion`

---

## 7. Product Flows

### 7.1 First Launch

```
┌─ User installs and opens EmberTerm
│
├─ Welcome screen (one-time):
│  • Pick default theme (3 thumbnails: Dark / Light / Tokyo Night)
│  • Pick default shell (auto-detected: pwsh, cmd, bash, zsh, wsl)
│  • Optional: enable agent integration (default ON)
│  • [Get Started]
│
├─ First tab opens with chosen profile
│
└─ Status bar shows hint: "Press Ctrl+Shift+P for the command palette"
```

### 7.2 Daily Use (no agent)

Indistinguishable from any modern terminal. Tabs, splits (v0.2), palette,
themes. Should feel like Tabby with less weight.

### 7.3 Daily Use (with agent)

```
┌─ User opens new tab with shell profile
│
├─ User runs: claude-code
│
├─ Agent emits OSC 7777 agent.hello → terminal updates status bar:
│  "● claude-code v1.0"
│
├─ User: "refactor src/main.rs to use async"
│
├─ Agent emits OSC 7777 tool.start (read_file)
│  → terminal renders inline tool card "📄 read_file · src/main.rs"
│
├─ Agent emits OSC 7777 tool.end → card collapses to "✓ read_file (0.1s)"
│
├─ Agent emits OSC 7777 diff:
│  → terminal renders inline diff viewer with [Accept] [Reject] [Edit]
│
├─ User presses Y:
│  → terminal sends {"id":"d_1","result":"accept"} to agent stdin
│  → diff viewer collapses to "✓ Applied to src/main.rs"
│
├─ Agent emits OSC 7777 approval (run tests command):
│  → approval sheet slides up
│
├─ User presses Y:
│  → response sent to agent
│  → sheet dismisses
│
└─ Agent runs tests, output streams normally
```

### 7.4 Settings Flow

```
┌─ User presses Ctrl+,
│
├─ Settings panel opens (overlays terminal, doesn't replace it)
│
├─ User searches "font" → filters to font-related controls
│
├─ User changes font size → terminal grid updates live behind the panel
│
├─ User clicks [Done] or presses Esc → panel closes, settings persisted
│
└─ Underneath: config.toml on disk is rewritten atomically
```

### 7.5 Profile Creation Flow

```
┌─ User opens Settings → Profiles
│
├─ Clicks [+ New Profile]
│
├─ Form: name, icon, shell command, args, cwd, env, theme override
│
├─ [Test] button: spawns the profile in a temporary tab to validate
│
├─ [Save] writes ~/.config/ember/profiles/<id>.toml
│
└─ Profile appears in new-tab dropdown immediately (file watcher)
```

---

## 8. Development Plan

### 8.1 Milestones

#### Phase 0 — Foundations (Weeks 1–2)

- [ ] Repo scaffold (Tauri + Svelte 5 + TypeScript + Vite)
- [ ] CI: build matrix (Windows, macOS), clippy, fmt, test
- [ ] Cargo workspace; add wezterm-term, termwiz, portable-pty
- [ ] Basic Tauri command/event plumbing with `ts-rs` codegen
- [ ] "Hello world" terminal: spawn shell, render bytes via xterm.js

#### Phase 1 — v0.1 (Weeks 3–8)

**Goal:** A usable, polished terminal that beats Cmder on UX and is lighter
than Tabby. Agent integration is a stub but visible.

- [ ] Terminal session lifecycle (create, write, resize, close)
- [ ] Multi-tab support with custom title bar
- [ ] Profile system (TOML, default profiles per OS, hot-reload)
- [ ] Settings UI (Appearance, Profiles, Keymap)
- [ ] Theme system + 5 built-in themes
- [ ] Command palette
- [ ] Status bar (cwd, git branch via simple shell-out, agent indicator)
- [ ] OSC handlers: 7, 8, 52, 133, 1337 (title)
- [ ] Synchronized output (DEC 2026)
- [ ] OSC 7777 parser: `agent.hello`, `tool.start`, `tool.end`
- [ ] Inline tool card rendering (basic)
- [ ] Acrylic on Windows 11; Vibrancy on macOS
- [ ] Installer: NSIS (Windows), DMG (macOS)
- [ ] Public spec doc: `docs/osc-7777-spec.md` (v0.1 of spec)
- [ ] Landing site / README with screenshots

**Exit criteria:** Internal dogfooding for 1 week; three external users
testing without showstoppers.

#### Phase 2 — v0.2 (Weeks 9–16)

**Goal:** Block UI lands. Agent integration becomes the headline feature.
Linux support.

- [ ] Block model: OSC 133 → block events → UI rendering
- [ ] Block UI: collapsible, navigable, per-block actions
- [ ] OSC 7777: `diff` verb + inline diff viewer
- [ ] OSC 7777: `approval` verb + approval sheet
- [ ] OSC 7777: `block.annotate`, `session.mark`, `tool.progress`
- [ ] Split panes
- [ ] Quake mode + global hotkey + tray icon
- [ ] Linux X11 support
- [ ] Kitty graphics protocol
- [ ] Kitty keyboard protocol
- [ ] Outreach: get one major agent (Claude Code / Aider) to ship OSC 7777 support

#### Phase 3 — v0.3 → v1.0 (Weeks 17–28)

- [ ] Native wgpu rendering (replace xterm.js)
- [ ] Smart scrollback search (block-aware filters)
- [ ] Session save/restore
- [ ] Linux Wayland
- [ ] Sixel + iTerm2 graphics
- [ ] Ligatures
- [ ] MCP server: terminal-as-MCP (cwd, blocks, history, git state)
- [ ] Multi-agent session view
- [ ] Theme marketplace / share format
- [ ] Auto-update
- [ ] Documentation site
- [ ] **v1.0 release**

### 8.2 Testing Strategy

| Layer | Approach |
|---|---|
| Rust unit tests | Standard `cargo test`; mock PTY for session tests |
| Rust integration | Spawn real shells in CI, send keystrokes, assert output |
| OSC 7777 parser | Property-based tests (proptest) on payloads |
| Frontend unit | Vitest for hooks/components |
| E2E | Playwright + WebDriver for Tauri (`tauri-driver`) |
| Manual smoke | Documented checklist run before each release |
| Performance | Criterion benchmarks for parser; manual frame-rate eval for UI |

### 8.3 Release Strategy

- **Versioning:** SemVer. Pre-1.0 versions can break config schema.
- **Channels:** stable + nightly (auto-built from `main`).
- **Distribution:**
  - GitHub Releases (binaries + installers)
  - winget (Windows), Homebrew Cask (macOS), AUR/.deb/.rpm (Linux v0.3+)
  - Auto-update via Tauri updater (signed)
- **Cadence:** v0.1 once, then minor releases ~monthly until v1.0.

### 8.4 License

- **Application:** MIT (encourages broad adoption)
- **OSC 7777 spec:** CC-BY-4.0 (free for any project to implement)

### 8.5 Contribution Model

- `CONTRIBUTING.md` covers dev setup, code style, PR template
- Issue templates: bug, feature, agent-integration
- `good-first-issue` label for onboarding
- All design discussions in GitHub Discussions until they reach RFC stage,
  then `docs/rfcs/NNNN-name.md`

---

## 9. References

### 9.1 Primary Engine Reference

- **WezTerm** — https://github.com/wezterm/wezterm
  - Use crates directly: `wezterm-term`, `termwiz`, `portable-pty`
  - Read for reference: image protocol implementations, key bindings
  - **Do not** copy: Lua config, font/render layer, multiplexer

### 9.2 UI / UX References

- **Tabby** — https://github.com/Eugeny/tabby — UI quality bar, profile system
- **Warp** — https://www.warp.dev — block UI, agent affordances (closed-source, observe only)
- **Windows Terminal** — https://github.com/microsoft/terminal — title bar / tab integration
- **Hyper** — https://github.com/vercel/hyper — extensibility model (don't follow, but study)
- **VS Code** — command palette UX, settings panel UX

### 9.3 Origin Inspirations

- **Cmder** — https://github.com/cmderdev/cmder — portability + nostalgic DNA
- **ConEmu** — https://github.com/ConEmu/ConEmu — Windows terminal heritage

### 9.4 Other Modern Terminals (Study)

- **Alacritty** — https://github.com/alacritty/alacritty — pure-perf reference
- **Ghostty** — https://github.com/ghostty-org/ghostty — Zig, native rendering
- **Kitty** — https://github.com/kovidgoyal/kitty — graphics protocol origin
- **Rio** — https://github.com/raphamorim/rio — Rust + WGPU example

### 9.5 Specifications

- **ANSI escape codes** — https://en.wikipedia.org/wiki/ANSI_escape_code
- **VT100 reference** — https://vt100.net/
- **Kitty keyboard protocol** — https://sw.kovidgoyal.net/kitty/keyboard-protocol/
- **Kitty graphics protocol** — https://sw.kovidgoyal.net/kitty/graphics-protocol/
- **OSC 8 hyperlinks** — https://gist.github.com/egmontkob/eb114294efbcd5adb1944c9f3cb5feda
- **OSC 133 prompt marks** — https://wezterm.org/shell-integration.html
- **OSC 52 clipboard** — https://invisible-island.net/xterm/ctlseqs/ctlseqs.html
- **DEC 2026 synchronized output** — https://gist.github.com/christianparpart/d8a62cc1ab659194337d73e399004036

### 9.6 Tauri Ecosystem

- **Tauri 2.x docs** — https://v2.tauri.app/
- **`ts-rs`** — https://github.com/Aleph-Alpha/ts-rs

### 9.7 Frontend

- **Svelte 5** — https://svelte.dev/
- **xterm.js** — https://github.com/xtermjs/xterm.js
- **bits-ui** — https://www.bits-ui.com/
- **Tailwind CSS** — https://tailwindcss.com/
- **VS Code theme color reference** — https://code.visualstudio.com/api/references/theme-color

### 9.8 Native Rendering (v0.2+)

- **`cosmic-text`** — https://github.com/pop-os/cosmic-text
- **`swash`** — https://github.com/dfrg/swash
- **`wgpu`** — https://wgpu.rs/

### 9.9 Telemetry

- **`sentry-rust`** — https://github.com/getsentry/sentry-rust
- **Sentry self-hosted** — https://develop.sentry.dev/self-hosted/

---

## 10. Risks & Open Questions

### 10.1 Technical Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| `wezterm-term` crate API changes | Med | Med | Pin version; fork if needed; track upstream |
| xterm.js perf insufficient for high-throughput | Low | Med | WebGL renderer + sync output; migrate to native earlier if needed |
| ConPTY edge cases on older Win10 | Low | High | Require 1809+; test on multiple builds |
| Tauri webview inconsistencies (WebView2 vs WKWebView) | Med | Low | CSS feature detection; avoid bleeding-edge web APIs |
| Native OS effects (acrylic) breaking on updates | Med | Low | Feature-detect; graceful fallback to opaque |

### 10.2 Product Risks

| Risk | Mitigation |
|---|---|
| Warp ships open-source / free tier | Differentiate on protocol openness, no account, lighter footprint |
| Microsoft ships agent UI in Windows Terminal | Race to ship OSC 7777 ecosystem first; cross-platform from v0.2 |
| No agents adopt OSC 7777 | Ship reference integrations for Claude Code, Aider ourselves; submit PRs upstream |
| User finds OSC 7777 redundant with MCP | Position OSC 7777 as the *display* layer, MCP as the *data* layer; not competing |

### 10.3 Resolved Decisions (was: Open Questions)

| Question | Decision | Rationale |
|---|---|---|
| Frontend framework | **Svelte 5 + TypeScript** | ~10× smaller bundle than React; better fine-grained reactivity for streaming agent updates; aligns with binary size and idle RAM targets |
| OSC 7777 response channel | **OSC 7777 reply on PTY input** | Symmetric protocol, no platform IPC, agents already understand OSC reply pattern from DSR |
| Theme format | **VS Code-compatible JSON** | Instant access to massive theme ecosystem; theme authors don't learn new format |
| Native font shaping (v0.2+) | **`cosmic-text` + `swash`** | Pure-Rust, no FFI, handles ligatures + COLR emoji + CJK; production-proven in COSMIC desktop |
| Telemetry scope | **Crash reports + version + OS only, opt-in** | Conservative default; nothing about commands, output, or agent content |

### 10.4 Telemetry

#### What we collect (only with explicit opt-in)

- **Crash reports** — stack traces, OS, EmberTerm version, last-action context
  (NOT terminal content, NOT command lines, NOT agent payloads)
- **Version + OS** — anonymized ping on startup so we know what to support
- **Nothing else.** No usage analytics, no feature tracking, no profile data.

#### Implementation

- **Sentry** via `sentry-rust` for Rust panics + frontend errors
- **Self-hosted Sentry instance** preferred over sentry.io for full control
  (alternative: Plausible / Umami for the version+OS ping if Sentry feels heavy)
- Opt-in toggle: **default OFF** on first launch
- Welcome screen explicitly asks ("Help us fix bugs by sending anonymous
  crash reports?") with [Yes] / [No, thanks] buttons
- Settings → Privacy → Telemetry: toggle + "View what's sent" link
- `docs/privacy.md` is the public source of truth for what's collected

#### What we never collect

- Terminal output bytes
- Command lines typed
- File paths from cwd / OSC 7
- Agent OSC 7777 payloads
- Theme content or profile names
- IP addresses (Sentry config strips them)

This is stricter than most desktop apps. The principle: a terminal sees
the most sensitive thing on a developer's machine. Default to silence.

### 10.5 Remaining Open Questions

- **Initial Sentry self-host vs sentry.io free tier:** decide before v0.1 release
- **Crash report symbolication:** strip in CI, upload symbols separately, or
  ship debuginfo with binaries?
- **`tool.cancel` event UX:** keyboard shortcut on tool cards? Right-click menu?

---

## 11. Glossary

| Term | Definition |
|---|---|
| **OSC** | Operating System Command — a class of ANSI escape sequences. Format: `ESC ] <num> ; <data> ESC \` |
| **PTY** | Pseudo-Terminal — a kernel object pair (master/slave) that lets a program emulate a terminal. ConPTY on Windows, `pty(7)` on Unix. |
| **ConPTY** | Windows pseudo-console API, available since Windows 10 1809. |
| **VT100/VT220** | DEC terminal models whose escape sequence vocabulary became the lingua franca of modern terminals. |
| **Block** | In EmberTerm: a single command + its output, treated as a navigable unit. Bounded by OSC 133 marks. |
| **Profile** | A preset combining shell command, args, env, cwd, and visual overrides. |
| **Agent** | An AI program emitting OSC 7777 sequences (Claude Code, Codex CLI, Aider, OpenCode, etc.). |
| **MCP** | Model Context Protocol — Anthropic's protocol for giving agents access to tools and data. |
| **wgpu** | Rust GPU abstraction over Vulkan/Metal/DX12/WebGPU; planned native renderer for v0.2+. |

---

*End of PLAN.md*