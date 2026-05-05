# EmberTerm UI/UX Design Review

**Reviewer role:** Senior UI/UX designer
**Reference:** `references/design.html` (canonical mockup)
**Implementation:** `frontend/src/`
**Date:** 2026-05-05
**Status:** Gap analysis — Phase 1 complete, Phase 2 in planning

---

## Executive Summary

The current implementation covers the structural skeleton of the design well: tabs, title bar, terminal grid, status bar, command palette, settings panel, and tool cards are all present. The layout hierarchy matches the mockup.

However, there are three layers of gaps:

1. **Design token mismatch** — variable names and values differ from the spec, causing subtle color divergence across all components.
2. **Component incompleteness** — several implemented components are missing visual details (status bar clock, tool card args/time, elevation differentiation).
3. **Missing Phase 2 components** — diff viewer, approval sheet, block UI, and agent-hello banner have no implementation and no stub styling.

Overall fidelity estimate: **65%** for implemented components, **40%** of the full design surface.

---

## 1. Design Token System

### Gaps

| Token (design) | Current equivalent | Issue |
|---|---|---|
| `--bg` | `--bg` | ✅ matches |
| `--bg-elev` | `--bg-elevated` | ❌ name mismatch — components referencing design names will break |
| `--bg-elev-2` | _(missing)_ | ❌ no second elevation level |
| `--bg-block` | _(missing)_ | ❌ needed for block backgrounds |
| `--border` | `--border` | ⚠️ value differs: design `#30363d`, current `#21262d` (too dark, reduces contrast) |
| `--fg` | `--fg` | ✅ matches |
| `--fg-muted` | `--fg-muted` | ✅ matches |
| `--accent` | `--accent` | ✅ matches |
| `--accent-soft` | _(missing)_ | ❌ used in hover states and secondary highlights |
| `--accent-glow` | _(missing)_ | ❌ used in active/focus glow effects |
| `--diff-add-bg` | _(missing)_ | ❌ required for diff viewer |
| `--diff-add-line` | _(missing)_ | ❌ required for diff viewer |
| `--diff-del-bg` | _(missing)_ | ❌ required for diff viewer |
| `--diff-del-line` | _(missing)_ | ❌ required for diff viewer |
| `--green` | _(missing)_ | ❌ semantic color, used in tool status OK |
| `--red` | _(missing)_ | ❌ semantic color, used in tool status error + risk badges |
| `--yellow` | _(missing)_ | ❌ semantic color, used in caution risk badge |
| `--blue` | _(missing)_ | ❌ semantic color |
| `--cyan` | _(missing)_ | ❌ semantic color |
| `--magenta` | _(missing)_ | ❌ semantic color |

The most impactful gap is the `--bg-elev` / `--bg-elevated` naming mismatch. Any future component referencing the design-spec variable name will silently fall back to the browser default.

The `--border` value being darker than the spec (`#21262d` vs `#30363d`) flattens the UI — borders blend into the background rather than providing visible surface separation.

---

## 2. Title Bar & Tabs

### Status: Good structural match

The title bar and tab strip are close to the mockup. Key differences:

| Aspect | Design | Current | Gap |
|---|---|---|---|
| Tab active indicator | 2px accent underline, flush bottom | 3px bottom border | Minor: height and flush position differ |
| Tab close button | `×` visible only on hover | `×` always visible on inactive tabs | ❌ wastes space, clutters strip |
| Tab `agent-active` state | Ember-orange dot left of title | Not implemented | ❌ no visual agent state on tab |
| Tab `running` state | Subtle pulse animation on dot | Not implemented | ❌ no in-progress feedback |
| Tab `unread` state | Small dot indicator | Not implemented | ❌ no unread feedback |
| New-tab `+` dropdown | Opens profile picker on click | Opens new tab directly | ⚠️ profile selection not surfaced |
| Double-click rename | ✅ implemented | ✅ implemented | — |

---

## 3. Status Bar

### Status: Missing segments and wrong height

| Aspect | Design | Current | Gap |
|---|---|---|---|
| Height | 26px | 24px | ❌ 2px short — tight with 12px text |
| `.status-shell` segment | Shell name (e.g. "zsh") | Not present | ❌ missing segment |
| `.status-cwd` segment | Current working directory | ✅ `⌂ ~/code` | Minor: `⌂` unicode vs design clean path icon |
| `.status-git` segment | Branch name with icon | ✅ present | ✅ |
| `.status-agent` segment | Agent name + version | ✅ present | ✅ |
| `.status-clock` segment | Current time (HH:MM) | ❌ not present | ❌ missing — visible in mockup right side |
| Separator dots | `●` between segments | ✅ present | ✅ |
| Font size | 12px | 12px | ✅ |

The missing clock is a visible absence — the right side of the status bar is empty in the current build. The missing `status-shell` segment means there's no quick way to know which shell is running.

---

## 4. Tool Card

### Status: Functional but visually incomplete

| Aspect | Design | Current | Gap |
|---|---|---|---|
| Card background | `--bg-elev` | Uses local `background` | ⚠️ not using elevation token |
| Icon | SVG icon per tool type | Emoji (`🔧`, `📄`, etc.) | ❌ emoji are inconsistent across OS/font |
| `.tool-args` display | Collapsible args block (key: value pairs) | Not implemented | ❌ expanded state shows no args |
| `.tool-time` badge | Duration in small monospace badge | Not implemented | ❌ no time shown in collapsed state |
| Progress bar | Full-width bar with `--accent` fill, 3px height | Not implemented | ❌ no visual progress indication |
| Status icon | Colored dot: green OK, red error, orange running | Text status only | ❌ no color coding |
| Cancel button | `[×]` icon button, right-aligned | ✅ implemented | ✅ |
| Expand/collapse | `[▾]` chevron, click to toggle args | Not implemented | ❌ no expand interaction |
| Border radius | 6px | ✅ matches | ✅ |
| Stacking (max 5) | Last 5 overlap from bottom | ✅ `.slice(-5)` in App.svelte | ✅ |

---

## 5. Command Palette

### Status: Close match

| Aspect | Design | Current | Gap |
|---|---|---|---|
| Width | ~600px max | 600px | ✅ |
| Backdrop | `blur(4px)` rgba | Plain rgba, no blur | ⚠️ missing frosted glass effect |
| Input border-radius | 8px on input | Not inspected — likely differs | Minor |
| Section headers | "Recent", "Commands", "Profiles", "Themes" | Not implemented | ⚠️ flat list, no sections |
| Keyboard navigation | Arrow keys + Enter | ✅ | ✅ |
| Close on Esc | ✅ | ✅ | ✅ |
| Fuzzy search | Fuzzy match with highlights | Substring match, no highlights | ⚠️ weaker search experience |

---

## 6. Settings Panel

### Status: Structurally correct

| Aspect | Design | Current | Gap |
|---|---|---|---|
| Two-pane layout | Category list left, controls right | ✅ | ✅ |
| Categories | Appearance, Profiles, Keymap, Agent, Advanced | ✅ all present | ✅ |
| Search | Filters across categories | ✅ | ✅ |
| Live preview | Theme/font update behind panel | Not implemented | ⚠️ changes apply on save, not live |
| Panel width | 640px | 600px | Minor |
| Backdrop | `blur(4px)` | None | ⚠️ missing blur |
| Transition | `220ms cubic-bezier(.2,.8,.2,1)` slide-in | Simple `opacity` fade | ⚠️ different feel |

---

## 7. Welcome Screen

### Status: Functionally complete, visually rough

| Aspect | Design | Current | Gap |
|---|---|---|---|
| Theme swatches | Rendered terminal preview thumbnails | Colored circles | ⚠️ not representative |
| Shell picker | Dropdown with auto-detected shells | ✅ | ✅ |
| Agent toggle | Styled toggle switch | ✅ | ✅ |
| Backdrop | Full-screen `blur(4px)` overlay | Plain rgba overlay | ⚠️ missing blur |

---

## 8. Missing Components (Phase 2)

These components exist in the design mockup but have no implementation or stub styling.

### 8.1 Block UI (`.block`, `.block-head`, `.block-body`)

**Implemented.** `Block.svelte` drives from `block:start` / `block:end` IPC events (emitted by the Rust `BlockManager` OSC 133 state machine). Each command+output unit has a collapsible header with command text, exit code badge, agent-id badge, and duration. Block store lives in `stores/app.ts`. Full xterm.js scrollback byte-range integration is deferred to a later phase.

### 8.2 Diff Viewer (`.diff`, `.diff-head`, `.diff-body`, `.diff-line`)

**Implemented.** `DiffView.svelte` parses unified diff patches into add/remove/context lines using `--diff-add-bg` / `--diff-del-bg` tokens. Header shows path + `+N/-N` stats. Footer renders action buttons from the `actions` payload. Wired to `agent:diff` IPC event; user actions call `agentRespond()` which writes the OSC 7777 reply to PTY stdin.

### 8.3 Approval Sheet (`.approval-sheet`, `.risk-badge`)

**Implemented.** `ApprovalSheet.svelte` slides up from the bottom with `220ms cubic-bezier(.2,.8,.2,1)` animation. Risk badge has `safe` / `caution` / `destructive` variants. Keyboard: `Y` = first action, `N` / `Esc` = deny. Wired to `agent:approval` IPC event; actions call `agentRespond()`.

### 8.4 Agent Hello Banner (`.agent-hello`)

**Implemented.** `HelloBanner.svelte` appears when `agent:hello` is received, shows agent name + version, auto-dismisses after 3s, has manual dismiss button. Cleared automatically on `agent:bye`.

### 8.5 Session Mark (`.session-mark`)

**Implemented.** `session.mark` OSC 7777 verb causes the Rust backend to emit ANSI-styled `────── label ──────` separator bytes via `terminal:output`, rendered inline by xterm.js without any extra frontend component.

### 8.6 Progress Bar (standalone `.progress-bar`)

Standalone progress bar component distinct from the tool card usage — used in block footers and potentially other contexts. Not implemented.

---

## 9. Global Interaction Patterns

| Pattern | Design spec | Current | Gap |
|---|---|---|---|
| Transition easing | `220ms cubic-bezier(.2,.8,.2,1)` on all panels/overlays | `0.1s ease` or no transition | ⚠️ feels snappier than intended |
| Backdrop blur | `blur(4px)` on overlays (palette, settings, welcome, approval) | None | ⚠️ overlays lack depth |
| Button system | `.btn` base + `.btn.primary`, `.btn.danger`, `.btn.ghost` modifiers | Ad-hoc per component | ⚠️ no shared button component |
| Focus ring | `2px solid var(--accent)` | Browser default or none | ⚠️ inconsistent keyboard navigation affordance |
| Scrollbar | `4px` custom scrollbar, `--bg-elev` track, `--fg-muted` thumb | OS default | Minor |

---

## Task Checklist

### P0 — Correctness (affects all components)

- [x] **Rename `--bg-elevated` → `--bg-elev` in `vscode-import.ts` and all component usages** — aligns variable names with design spec so future components work without manual mapping
- [x] **Add missing design tokens** to `tokens.ts` and `ember-dark.json`: `--bg-elev-2`, `--bg-block`, `--accent-soft`, `--accent-glow`, `--green`, `--red`, `--yellow`, `--blue`, `--cyan`, `--magenta`, `--diff-add-bg`, `--diff-add-line`, `--diff-del-bg`, `--diff-del-line`
- [x] **Fix `--border` value** from `#21262d` to `#30363d` in `ember-dark.json`

### P1 — Visible gaps in implemented components

- [x] **Status bar: add `.status-clock`** — right-aligned time display, updates every minute via `setInterval`
- [x] **Status bar: add `.status-shell`** — expose shell name from session (currently in `Session.title` on Rust side, needs IPC event or command)
- [x] **Status bar: increase height** from 24px to 26px
- [x] **Tool card: add `.tool-args`** — collapsible key/value arg display in expanded state; add expand/collapse chevron
- [x] **Tool card: add `.tool-time` badge** — show duration in monospace badge after `tool.end`
- [x] **Tool card: add progress bar** — full-width `--accent`-filled bar, shown during `tool.progress`, hidden after `tool.end`
- [x] **Tool card: replace emoji icons** with a minimal SVG icon set or CSS-based icons keyed on `tool.name`
- [x] **Tool card: add status dot** — colored dot (green/red/orange) indicating status state
- [x] **Tabs: show close `×` only on hover** for inactive tabs; always show for active tab
- [x] **Tabs: add `.agent-active` state** — ember-orange dot when agent is connected on that session
- [x] **Command palette: add section headers** — "Commands", "Themes" groupings with section dividers

### P2 — Polish and interaction quality

- [x] **Add `backdrop-filter: blur(4px)`** to: CommandPalette, SettingsPanel, WelcomeScreen overlays
- [x] **Update panel/overlay transitions** to `220ms cubic-bezier(.2,.8,.2,1)` (settings slide-in, palette slide-down, welcome slide-up)
- [x] **Extract shared `.btn` component** with `.primary`, `.danger`, `.ghost` variants; replace ad-hoc button styles in ToolCard, WelcomeScreen, CommandPalette, SettingsPanel
- [x] **Add `:focus-visible` ring** — `2px solid var(--accent)` on all interactive elements, via `app.css`
- [x] **Welcome screen: improve theme swatches** — small rendered terminal previews instead of color circles
- [x] **Tabs: add `running` pulse animation** — subtle CSS keyframe animation on tab dot when PTY is active
- [x] **Settings panel: live preview** — apply theme/font changes immediately as the user drags/selects; revert on cancel

### P3 — Phase 2 components (new files required)

- [x] **`ApprovalSheet.svelte`** — bottom-anchored overlay, slide-up transition, risk badge, action buttons; wire to `approval` OSC 7777 verb handler and `agentRespond()` IPC
- [x] **`DiffView.svelte`** — unified diff renderer with `--diff-add-bg`/`--diff-del-bg` coloring, accept/reject/edit actions (Y/N/E); wire to `diff` OSC 7777 verb handler
- [x] **Block model + `Block.svelte`** — drive from OSC 133 `PromptMark` events; collapsible command+output units with header, exit code badge, agent-id badge
- [x] **Agent hello banner** — transient pill banner on `agent.hello`; auto-dismiss after 3s; wired in App.svelte
- [x] **Session mark** — ANSI-styled `────── label ──────` separator bytes emitted via `terminal:output` on `session.mark` verb; rendered by xterm.js inline in scrollback
- [x] **Custom scrollbar styles** — 4px-wide overlay scrollbar using `--fg-muted` thumb, via `app.css`

---

*Reference: `references/design.html` — open in a browser to inspect all component states.*
