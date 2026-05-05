# Contributing to EmberTerm

## Dev Setup

**Prerequisites:** Rust 1.85+, Node 20+, pnpm or npm.

```bash
# Clone
git clone https://github.com/your-org/emberterm
cd emberterm

# Install frontend deps
cd frontend && npm install && cd ..

# Run in dev mode (hot-reload UI + Rust rebuild on change)
cargo tauri dev
```

### First-run notes

- On Windows, you need the [WebView2 runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (pre-installed on Windows 11).
- On macOS, Xcode Command Line Tools are required (`xcode-select --install`).

## Project Layout

```
src-tauri/   Rust core (PTY, OSC parser, IPC, config)
frontend/    Svelte 5 + TypeScript UI
docs/        Specifications and guides
```

## Build

```bash
# Type-check frontend
cd frontend && npm run check

# Run Rust tests
cargo test --manifest-path src-tauri/Cargo.toml

# Run frontend tests (vitest)
cd frontend && npm test

# Production build
cargo tauri build
```

## Code Style

- **Rust:** `cargo fmt` before committing. `cargo clippy -- -D warnings` must pass.
- **TypeScript/Svelte:** no linter configured yet; follow the existing style.
- **Commits:** imperative present tense, one sentence. No `Co-Authored-By` trailers.

## Pull Requests

1. Branch from `main`.
2. Keep PRs focused — one feature or fix per PR.
3. Add tests for new Rust parsing logic (`src-tauri/src/terminal/parser.rs`).
4. Update `docs/TASKS.md` checkboxes if you complete a planned item.
5. For OSC 7777 spec changes, update `docs/OSC-777-SPEC.md` and bump the changelog.

## Issue Templates

- **Bug:** include OS, EmberTerm version, steps to reproduce, and expected vs actual output.
- **Feature:** describe the use case before the implementation.
- **Agent integration:** link to the agent project and describe which OSC 7777 verbs you need.

## License

MIT. By contributing you agree your changes are released under the same license.
