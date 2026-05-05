# /test — Run the full test suite

Run all automated tests for EmberTerm.

Steps:
1. Run `cargo test --manifest-path src-tauri/Cargo.toml 2>&1` — Rust unit tests
2. Run `cd frontend && npm test 2>&1` — Vitest frontend tests

After both complete, report:
- Pass / fail count per suite
- Names of any failing tests with the failure message (first 10 lines each)
- Total: "X/Y tests passed"

If $ARGUMENTS is provided, use it as a test filter:
- For Rust: `cargo test --manifest-path src-tauri/Cargo.toml $ARGUMENTS`
- For Vitest: `cd frontend && npm test -- --reporter=verbose $ARGUMENTS`
