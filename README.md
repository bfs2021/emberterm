# EmberTerm

> A fast, simple terminal.

A lightweight personal terminal emulator built on Tauri 2 (Rust) + Svelte 5 + xterm.js, using `portable-pty`. Many tabs with different shells (Git Bash, WSL, PowerShell), splits, and the previous session restored on reopen.

## Status

**Pre-v0.1 — active development.** Phase 1 (v0.1 feature set) is implemented and under internal testing.

What's working:
- Multi-tab terminal (xterm.js) with custom title bar, tab strip, and status bar
- Tab and split-pane restore: tabs, titles, shells and each pane's folder come back on reopen
- OSC 7 (folder), window title and OSC 52 clipboard write, handled in xterm.js
- VS Code theme system (7 built-in themes + paste any VS Code JSON)
- Command palette, settings panel, profile manager
- File-watcher hot-reload for config and profiles; a broken file never blocks startup
- Git branch in status bar
- Welcome screen on first launch
- Windows 11 Mica / macOS vibrancy

## Features

- **Session restore** — closing and reopening brings back every tab, split and folder
- **Light** — Tauri instead of Electron
- **No account, no telemetry by default, no cloud**
- **VS Code theme ecosystem** — paste any VS Code theme JSON and it works

## Folder restore

EmberTerm restores each tab's folder for Git Bash, WSL and PowerShell by injecting a prompt hook.
If your `~/.bashrc` *assigns* `PROMPT_COMMAND=...`, the hook is replaced. Append instead:

    PROMPT_COMMAND="${PROMPT_COMMAND:+$PROMPT_COMMAND;}your_command"

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
# Rust unit tests
cargo test --manifest-path packages/desktop/Cargo.toml

# All JS/TS tests across every package
pnpm turbo test
```

## Project Structure

```
packages/
  desktop/   Rust/Tauri core — PTY, shell hooks, IPC, config, profiles, session file
  app/       Svelte 5 UI — xterm.js, tabs, theme system (@emberterm/app)
  ui/        Shared Svelte component library (@emberterm/ui)
docs/        Plan and task list
```

See [`docs/PLAN.md`](docs/PLAN.md) for the architecture and non-goals.

## License

[MIT](LICENSE)
