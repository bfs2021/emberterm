# /check — Quick type-check without building

Run a fast type-check on the EmberTerm codebase. Does not do a full build or run tests.

Steps:
1. Run `cargo check --manifest-path packages/desktop/Cargo.toml` — Rust type + borrow check
2. Run `pnpm turbo check` — TypeScript + Svelte type check via svelte-check across all packages

After both complete, summarise:
- List any errors grouped by file (path:line — message)
- If everything passes, say "✓ All clear"
- Do not auto-fix anything — report only
