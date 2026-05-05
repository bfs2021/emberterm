# /lint — Run all linters and format checks

Run every static analysis and format check. Run this before opening a PR.

Steps:
1. `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings 2>&1`
   — Rust lints; treat all warnings as errors
2. `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check 2>&1`
   — Rust format check (does not modify files)
3. `cd frontend && npm run check 2>&1`
   — TypeScript + Svelte type check

For each step, report:
- Pass (no output) or Fail (first 30 lines of output)

4. Check for files exceeding 300 lines (excluding test-only files):
   `find src-tauri/src frontend/src -name "*.rs" -o -name "*.svelte" -o -name "*.ts" | xargs wc -l | sort -rn | awk '$1 > 300 && $2 != "total"'`
   — list any over-long files and suggest how to split them

If any step fails, identify the root cause and offer to fix it.
Auto-format Rust only if the user confirms: `cargo fmt --manifest-path src-tauri/Cargo.toml`
