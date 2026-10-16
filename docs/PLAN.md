# EmberTerm — Plan

**Status:** personal terminal, pivoted 2026-10-16.

## Intent
A lightweight, fast, reliable Windows terminal for daily use: many tabs, each with its own shell
(Git Bash, WSL, pwsh), splits, good UI/UX, and the previous session restored on reopen.
Replaces Tabby, which failed to launch after updates and forgot sessions.

## Architecture
- Rust (Tauri 2): PTY spawn/write/resize/close, DEC 2026 coalescing, shell hooks, config/profiles/session files, git status.
- Frontend (Svelte 5 + xterm.js): rendering and all escape-sequence parsing, tabs/splits, settings, palette, themes, session snapshots.
- Output: Tauri Channel, raw bytes. Session: `session.json`, autosaved (500 ms debounce), atomic writes.

## Non-goals
AI-agent protocols, scrollback restore, background shells, auto-update, installers/signing, native rendering, cmd folder tracking.

## Next
- UI/UX polish pass.
- Packaging for personal install (unsigned NSIS) if needed.
