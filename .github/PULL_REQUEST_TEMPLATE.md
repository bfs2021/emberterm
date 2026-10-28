## What and why

<!-- What changed, and why. Link the issue if there is one. -->

## How you checked it

<!-- Commands you ran and what you saw. For interface changes, add a screenshot. -->

CI runs only after a merge, so these are checked locally:

- [ ] `cargo fmt --check`, `cargo clippy -- -D warnings` and `cargo test` pass
- [ ] `pnpm turbo check test build` and `pnpm version:check` pass
- [ ] Bug fixes have a test that failed before the fix
- [ ] No secrets, personal paths or machine names in the diff
- [ ] Docs and `CHANGELOG.md` updated if behaviour changed
