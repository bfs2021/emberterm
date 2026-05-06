# /build — Full Tauri build

Build EmberTerm end-to-end. Useful before committing or packaging.

Steps:
1. Run `pnpm turbo build 2>&1` — builds all JS packages in dependency order (ui → sdk → app)
2. If step 1 passes, run `cargo build --manifest-path packages/desktop/Cargo.toml 2>&1` — Rust debug build

For a release build, add `--release` to step 2 (takes longer, applies LTO + strip).

Report:
- Whether each step passed or failed
- If failed: the first 20 error lines
- If passed: binary location and approximate size

Note: full release builds take several minutes due to `lto = true` in Cargo.toml.
