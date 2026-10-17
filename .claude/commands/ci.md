# /ci — Run what CI runs, locally

CI (`.github/workflows/ci.yml`) runs only on pushes to `main`, so a PR is checked here: run this
before opening a PR or marking a draft ready for review. It is the list in CONTRIBUTING.md
("Before you open a PR").

Steps (run all of them, even after a failure):
1. `cargo fmt --manifest-path packages/desktop/Cargo.toml -- --check 2>&1`
2. `cargo clippy --manifest-path packages/desktop/Cargo.toml --all-targets -- -D warnings 2>&1`
3. `cargo test --manifest-path packages/desktop/Cargo.toml 2>&1`
4. `pnpm turbo check test build 2>&1`
5. `pnpm version:check 2>&1`

Report:
- Pass / fail per step; for a failure, the first 30 relevant lines
- Test counts (Rust and Vitest)
- A short "Verification" list to paste into the PR body (the commands and their result)

CI also builds and tests on macOS after the merge; this run covers only the local platform, so say so
when a change touches platform-specific code.
