// Design token definitions — CSS variable names mapped from VS Code theme keys.
// Applied to :root on theme switch. Phase 1.

export const TOKEN_MAP: Record<string, string> = {
  "editor.background": "--ui-bg",
  "sideBar.background": "--ui-bg-elevated",
  "sideBar.border": "--ui-border",
  "tab.activeBackground": "--tab-active-bg",
  "tab.inactiveBackground": "--tab-inactive-bg",
  "tab.activeForeground": "--tab-active-fg",
  "statusBar.background": "--statusbar-bg",
  "statusBar.foreground": "--statusbar-fg",
  "focusBorder": "--ui-accent",
  "errorForeground": "--ui-danger",
  "terminal.background": "--terminal-bg",
  "terminal.foreground": "--terminal-fg",
};
