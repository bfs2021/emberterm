# EmberTerm — Theming Guide

EmberTerm uses **VS Code-compatible theme JSON** files. Any VS Code color theme works
out of the box.

## Built-in Themes

| Theme | Kind |
|---|---|
| Ember Dark | Dark (default) |
| Ember Light | Light |
| Tokyo Night | Dark |
| Catppuccin Mocha | Dark |
| Dracula | Dark |
| GitHub Dark | Dark |
| Solarized Dark | Dark |

Switch themes via `Ctrl+Shift+P` → type "Theme" — or open **Settings → Color Scheme**. Your choice is remembered across restarts.

## Importing a VS Code Theme

1. Find a theme on the [VS Code Marketplace](https://marketplace.visualstudio.com/search?target=VSCode&category=Themes).
2. Download the `.json` file from the extension's source (usually `themes/*.json`).
3. Open Settings → Color Scheme → **Import VS Code Theme**.
4. Paste the JSON and click **Import**.

Imported themes are stored in `~/.config/ember/config.toml` and stay available after a restart.

## Writing a Custom Theme

Create a JSON file following the VS Code color reference. The keys EmberTerm reads:

```jsonc
{
  "name": "My Theme",
  "type": "dark",
  "colors": {
    // Terminal grid
    "terminal.background": "#0d1117",
    "terminal.foreground": "#e6edf3",
    "terminal.ansiBlack": "#484f58",
    "terminal.ansiRed": "#ff7b72",
    "terminal.ansiGreen": "#7ee787",
    "terminal.ansiYellow": "#d29922",
    "terminal.ansiBlue": "#58a6ff",
    "terminal.ansiMagenta": "#bc8cff",
    "terminal.ansiCyan": "#39c5cf",
    "terminal.ansiWhite": "#b1bac4",
    "terminal.ansiBrightBlack": "#6e7681",
    "terminal.ansiBrightRed": "#ffa198",
    "terminal.ansiBrightGreen": "#56d364",
    "terminal.ansiBrightYellow": "#e3b341",
    "terminal.ansiBrightBlue": "#79c0ff",
    "terminal.ansiBrightMagenta": "#d2a8ff",
    "terminal.ansiBrightCyan": "#56d4dd",
    "terminal.ansiBrightWhite": "#f0f6fc",
    "terminalCursor.foreground": "#e6edf3",
    "terminalCursor.background": "#0d1117",

    // UI chrome (title bar, tabs, status bar, settings)
    "editor.background": "#0d1117",
    "sideBar.background": "#010409",
    "sideBar.border": "#21262d",
    "tab.activeBackground": "#0d1117",
    "tab.inactiveBackground": "#010409",
    "tab.activeForeground": "#e6edf3",
    "tab.inactiveForeground": "#7d8590",
    "statusBar.background": "#010409",
    "statusBar.foreground": "#7d8590",

    // Accent + danger
    "focusBorder": "#FF6B35",
    "errorForeground": "#f85149"
  }
}
```

Keys not present fall back to safe defaults (the Ember Dark values).

## CSS Variables

EmberTerm maps theme colors to CSS custom properties on `:root`:

| Variable | Mapped from |
|---|---|
| `--bg` | `editor.background` |
| `--bg-elev` | `sideBar.background` |
| `--border` | `sideBar.border` |
| `--fg` | `terminal.foreground` |
| `--fg-muted` | `tab.inactiveForeground` |
| `--accent` | `focusBorder` |
| `--danger` | `errorForeground` |
| `--status-bg` | `statusBar.background` |
| `--status-fg` | `statusBar.foreground` |
