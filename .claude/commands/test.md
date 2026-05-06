# /test — Run the full test suite

Run all automated tests for EmberTerm.

Steps:
1. Run `cargo test --manifest-path packages/desktop/Cargo.toml 2>&1` — Rust unit tests
2. Run `pnpm turbo test 2>&1` — Vitest tests across all JS/TS packages (app + sdk)

After both complete, report:
- Pass / fail count per suite
- Names of any failing tests with the failure message (first 10 lines each)
- Total: "X/Y tests passed"

If $ARGUMENTS is provided, use it as a test filter:
- For Rust: `cargo test --manifest-path packages/desktop/Cargo.toml $ARGUMENTS`
- For Vitest (single package): `pnpm --filter @emberterm/app test -- --reporter=verbose $ARGUMENTS`
- For SDK tests: `pnpm --filter @emberterm/sdk test`
