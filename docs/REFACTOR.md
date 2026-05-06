# EmberTerm — Modular Monolith Refactor

Goal: reorganise into a `packages/`-based workspace matching the OpenCode pattern.
Reference: `docs/PLAN.md` — see also git history for the planning commit.

## Legend
- `[x]` Done
- `[ ]` Not started / in progress

---

## Phase 0 — Workspace Bootstrap

- [x] Create root `Cargo.toml` (workspace member: `packages/desktop`)
- [x] Create root `pnpm-workspace.yaml` (packages: `packages/*`, `e2e`)
- [x] Create root `package.json` (name: emberterm, private, turbo scripts)
- [x] Create root `turbo.json` (tasks: build, check, test, lint)
- [ ] Install pnpm globally on dev machines (if not already)

## Phase 1 — Move Existing Packages

### 1a — Rust: `src-tauri/` → `packages/desktop/`
- [x] Move all files from `src-tauri/` into `packages/desktop/`
- [x] Verify `packages/desktop/build.rs` still finds `tauri.conf.json` (same dir — OK)
- [x] Run `cargo check` from repo root to confirm workspace resolution

### 1b — Update `packages/desktop/tauri.conf.json`
- [x] `frontendDist`: `"../app/dist"` (was `"../frontend/dist"`)
- [x] `beforeDevCommand`: `"pnpm --filter @emberterm/app dev"` (was `"cd frontend && npm run dev"`)
- [x] `beforeBuildCommand`: `"pnpm --filter @emberterm/app build"` (was `"cd frontend && npm run build"`)

### 1c — JS: `frontend/` → `packages/app/`
- [x] Move all files from `frontend/` into `packages/app/`
- [x] Update `packages/app/package.json`: name → `@emberterm/app`
- [x] Delete `frontend/package-lock.json` (replaced by root `pnpm-lock.yaml`)
- [x] Run `pnpm install` from repo root to generate `pnpm-lock.yaml`

## Phase 2 — Create `packages/ui/`

- [x] Create `packages/ui/src/lib/Btn.svelte` (move from `packages/app/src/lib/Btn.svelte`)
- [x] Create `packages/ui/src/index.ts` (barrel: `export { default as Btn } from "./lib/Btn.svelte"`)
- [x] Create `packages/ui/package.json` (name: `@emberterm/ui`, private, lib build)
- [x] Create `packages/ui/vite.config.ts` (Svelte library mode)
- [x] Create `packages/ui/tsconfig.json`
- [x] Add `"@emberterm/ui": "workspace:*"` to `packages/app/package.json` dependencies
- [x] Update import in `packages/app/src/settings/ProfileEditor.svelte`:
      `import { Btn } from "@emberterm/ui"` (was `import Btn from "../lib/Btn.svelte"`)
- [x] Run `pnpm turbo build` — confirm ui builds before app

## Phase 3 — Create `packages/sdk/`

- [x] Create `packages/sdk/src/types.ts` (TypeScript mirrors of `agent/protocol.rs` structs)
- [x] Create `packages/sdk/src/emit.ts` (OSC 7777 emit helpers for agent processes)
- [x] Create `packages/sdk/src/index.ts` (barrel export)
- [x] Create `packages/sdk/package.json` (name: `@emberterm/sdk`, zero runtime deps)
- [x] Create `packages/sdk/tsconfig.json`
- [x] Write SDK unit tests in `packages/sdk/src/emit.test.ts`
- [x] Run `pnpm --filter @emberterm/sdk test`

## Phase 4 — Update CI

### `.github/workflows/ci.yml`
- [x] Replace `actions/setup-node` npm cache with pnpm setup (`pnpm/action-setup@v4`)
- [x] Replace `npm ci` with `pnpm install --frozen-lockfile` (at repo root)
- [x] Replace `working-directory: frontend` steps with `pnpm turbo check` + `pnpm turbo build`
- [x] Replace `working-directory: src-tauri` with `working-directory: packages/desktop`
- [x] Update `Swatinem/rust-cache workspaces:` from `src-tauri` → `packages/desktop`
- [x] Add `pnpm turbo test` step for JS tests across all packages

### `.github/workflows/release.yml`
- [x] Same pnpm migration as ci.yml
- [x] Update `working-directory: packages/desktop` for Rust steps
- [x] Update `Swatinem/rust-cache workspaces: packages/desktop`
- [x] Verify `tauri-apps/tauri-action` still locates `tauri.conf.json` correctly

## Phase 5 — Update Developer Tooling

- [x] Update `.claude/commands/check.md` — new paths/commands
- [x] Update `.claude/commands/test.md` — new paths/commands
- [x] Update `.claude/commands/build.md` — new paths/commands
- [x] Update `.claude/commands/lint.md` — new paths/commands
- [x] Update `CLAUDE.md` — File Map section (all `src-tauri/` → `packages/desktop/`, `frontend/src/` → `packages/app/src/`)
- [x] Update `CLAUDE.md` — Dev Commands section (use pnpm + turbo)
- [x] Update `CLAUDE.md` — Architecture section (mention 4 packages)

## Phase 6 — Smoke Test

- [x] `cargo check` — Rust compiles from repo root
- [x] `cargo test` — all 84 Rust tests pass
- [x] `pnpm install` — lockfile generated cleanly
- [x] `pnpm turbo build` — all packages build (ui → sdk → app) in order
- [x] `pnpm turbo check` — svelte-check passes for app and ui (0 errors)
- [x] `pnpm turbo test` — 118 app tests + 12 SDK tests pass

---

*End of REFACTOR.md*
