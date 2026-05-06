# EmberTerm

> Where shells and agents meet.

A modern, lightweight, agent-native terminal emulator built on Tauri 2 (Rust) + Svelte 5. EmberTerm uses `termwiz` and `portable-pty` for its terminal core and defines **OSC 7777**, an open protocol for AI coding agent integration.

## Status

**Pre-v0.1 — active development.** Phase 1 (v0.1 feature set) is implemented and under internal testing.

What's working:
- Multi-tab terminal (xterm.js) with custom title bar, tab strip, and status bar
- OSC 7, 8, 52, 133, 7777 parsing; window title; cwd tracking; clipboard write
- OSC 7777 agent protocol: `agent.hello`, `tool.start/progress/end` — inline tool cards with cancel
- VS Code theme system (7 built-in themes + paste any VS Code JSON)
- Command palette, settings panel, profile manager
- File-watcher hot-reload for config and profiles
- Git branch in status bar
- Welcome screen on first launch
- Windows 11 Mica / macOS vibrancy

## Features

- **Agent-native UX** — inline tool cards, diff viewers (v0.2), approval sheets (v0.2) via OSC 7777
- **Tabby-quality UI** at ~10× less weight (Tauri vs Electron)
- **Open protocol** — [OSC 7777 spec](docs/osc-7777-spec.md) is CC-BY-4.0, any agent can adopt it
- **No account, no telemetry by default, no cloud**
- **VS Code theme ecosystem** — paste any VS Code theme JSON and it works

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) 1.85+
- [Node.js](https://nodejs.org/) 22+
- [pnpm](https://pnpm.io/) 9+ (`npm install -g pnpm`)
- **Windows:** WebView2 (pre-installed on Windows 11)
- **macOS:** Xcode Command Line Tools (`xcode-select --install`)

### Development

```sh
# Clone and install all dependencies (JS workspace + Rust)
git clone https://github.com/emberterm/emberterm
cd emberterm
pnpm install

# Start dev server (Vite hot-reload UI + Rust rebuild on change)
pnpm dev
```

### Build

```sh
# Build all JS packages then the Tauri app
pnpm turbo build
cargo build --manifest-path packages/desktop/Cargo.toml
```

Installers are written to `packages/desktop/target/release/bundle/`.

### Run tests

```sh
# Rust unit tests (OSC parser, protocol, blocks, etc.)
cargo test --manifest-path packages/desktop/Cargo.toml

# All JS/TS tests across every package (app + sdk)
pnpm turbo test
```

## Project Structure

```
packages/
  desktop/   Rust/Tauri core — PTY, OSC parser, IPC, config, profiles
  app/       Svelte 5 UI — xterm.js, theme system, agent cards (@emberterm/app)
  ui/        Shared Svelte component library (@emberterm/ui)
  sdk/       Public OSC 7777 TypeScript SDK for agent developers (@emberterm/sdk)
e2e/         End-to-end tests (WebdriverIO + tauri-driver)
docs/        Specifications and planning documents
```

See [`docs/PLAN.md`](docs/PLAN.md) for the full architecture and roadmap.  
See [`docs/osc-7777-spec.md`](docs/osc-7777-spec.md) for the OSC 7777 agent protocol specification.

## OSC 7777 SDK

Agent developers can use `@emberterm/sdk` to emit structured OSC 7777 sequences from any Node.js process — no EmberTerm-specific dependencies required:

```ts
import { emitHello, emitToolStart, emitToolEnd } from "@emberterm/sdk";

emitHello({ v: 1, name: "my-agent", version: "1.0.0" });
emitToolStart({ v: 1, id: "t1", name: "read_file", summary: "Reading src/main.rs" });
// ... do work ...
emitToolEnd({ v: 1, id: "t1", status: "ok", summary: "Done", duration_ms: 120 });
```

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for dev setup, code style, and PR guidelines.

Issues and discussions welcome at [github.com/emberterm/emberterm](https://github.com/emberterm/emberterm).

## License

- **Application:** [MIT](LICENSE)
- **OSC 7777 Specification:** [CC-BY-4.0](docs/osc-7777-spec.md)
