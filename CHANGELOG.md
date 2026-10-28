# Changelog

All notable changes are listed here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow [Semantic Versioning](https://semver.org).

## [Unreleased]

### Security

- The status bar no longer runs `git` in a network path or a missing folder, and runs it with `core.fsmonitor` off and without repository locks. The folder comes from terminal output, which a program can forge.
- Programs can no longer set the clipboard with OSC 52 unless you turn on **Settings → Terminal → Let programs set the clipboard** (off by default; tmux or vim clipboard sync over ssh needs it). Payloads over 100,000 bytes are refused.
- Profile ids are limited to letters, digits, `-` and `_`; an id such as `../x` can no longer write or delete files outside the profiles folder.
- A Content Security Policy is set for the app window.
- Updated `rustls-webpki` and `webpki-root-certs` (RUSTSEC-2026-0382, -0383).
- Release workflow actions are pinned to commit SHAs and Dependabot keeps them current.

## [0.4.0] - 2026-10-22

### Added

- Reorder tabs by dragging, from the context menu, or with the keyboard.

### Fixed

- The status bar git branch and changes refresh when a command finishes.
- The console window no longer flashes when the status bar runs `git` on Windows.

## [0.3.0] - 2026-10-17

### Added

- A folder picker for a profile's working directory in Settings.

## [0.2.0] - 2026-10-17

### Added

- A universal macOS build in the release workflow.

### Changed

- PTY output is coalesced into batched channel sends and acknowledged by the webview, and terminals render with the xterm.js WebGL addon, so heavy output stays responsive.

## [0.1.1] - 2026-10-16

### Changed

- One version for the whole repo, checked in CI and on release.

### Fixed

- Closing a tab when its shell exits, and closing the window with the last tab.
- Settings that were not wired up, and the confirm-close prompt for running programs.
- The status bar clock now follows the system clock.

## [0.1.0] - 2026-10-16

### Added

- Tabs with their own shell (Git Bash, WSL, PowerShell, cmd), split panes, and session restore for tabs, splits, titles and folders.
- Copy, paste, a right-click menu, and find in scrollback.
- Busy and unread indicators on background tabs.
- Seven built-in themes, VS Code theme import, a font picker, a command palette and a status bar.
- Window size, position and maximized state are remembered.
- One-click updates from Settings → About.
- Config, profiles and `session.json` hot-reload; a broken file is moved aside with a toast.
