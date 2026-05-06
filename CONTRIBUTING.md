# Contributing to EmberTerm

## Dev Setup

**Prerequisites:** Rust 1.85+, Node.js 22+, pnpm 9+.

```bash
# Install pnpm if you don't have it
npm install -g pnpm

# Clone
git clone https://github.com/emberterm/emberterm
cd emberterm

# Install all JS workspace deps (app, ui, sdk, e2e) in one shot
pnpm install

# Run in dev mode (Vite hot-reload UI + Rust rebuild on change)
pnpm dev
```

### First-run notes

- On Windows, you need the [WebView2 runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (pre-installed on Windows 11).
- On macOS, Xcode Command Line Tools are required (`xcode-select --install`).

## Project Layout

```
packages/
  desktop/   Rust/Tauri backend (PTY, OSC parser, IPC, config, profiles)
  app/       Svelte 5 + TypeScript UI (@emberterm/app)
  ui/        Shared Svelte component library (@emberterm/ui)
  sdk/       Public OSC 7777 TypeScript SDK for agents (@emberterm/sdk)
e2e/         End-to-end tests (WebdriverIO + tauri-driver)
docs/        Specs, architecture, task tracking
```

## Build & Test

```bash
# Type-check all packages (fast, no build)
cargo check --manifest-path packages/desktop/Cargo.toml
pnpm turbo check

# Run all tests
cargo test --manifest-path packages/desktop/Cargo.toml   # 84 Rust tests
pnpm turbo test                                           # 130 JS/TS tests (app + sdk)

# Full production build
pnpm turbo build                                          # JS packages (ui → sdk → app)
cargo build --manifest-path packages/desktop/Cargo.toml  # Rust binary

# Lint
cargo clippy --manifest-path packages/desktop/Cargo.toml -- -D warnings
cargo fmt --manifest-path packages/desktop/Cargo.toml -- --check
pnpm turbo check
```

## Code Style

- **Rust:** `cargo fmt` before committing. `cargo clippy -- -D warnings` must pass.
- **TypeScript/Svelte:** Svelte 5 runes only (`$state`, `$derived`, `$effect`, `$props`). See `CLAUDE.md` for full conventions.
- **Commits:** imperative present tense, one sentence. No `Co-Authored-By` trailers.

## Pull Requests

1. Branch from `main`.
2. Keep PRs focused — one feature or fix per PR.
3. Add tests for new Rust parsing logic (`packages/desktop/src/terminal/parser.rs`).
4. Update `docs/TASKS.md` checkboxes if you complete a planned item.
5. For OSC 7777 spec changes, update `docs/osc-7777-spec.md` and bump the changelog.

## Package Responsibilities

| Package | Owner area | Test command |
|---|---|---|
| `packages/desktop` | PTY, OSC parser, IPC | `cargo test --manifest-path packages/desktop/Cargo.toml` |
| `packages/app` | Svelte UI, xterm.js, stores | `pnpm --filter @emberterm/app test` |
| `packages/ui` | Shared Svelte components | `pnpm --filter @emberterm/ui check` |
| `packages/sdk` | OSC 7777 emit helpers | `pnpm --filter @emberterm/sdk test` |
| `e2e` | End-to-end (needs tauri-driver) | `pnpm --filter emberterm-e2e test:e2e` |

## Issue Templates

- **Bug:** include OS, EmberTerm version, steps to reproduce, and expected vs actual output.
- **Feature:** describe the use case before the implementation.
- **Agent integration:** link to the agent project and describe which OSC 7777 verbs you need.

## License

MIT. By contributing you agree your changes are released under the same license.
