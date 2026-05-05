export interface VsCodeTheme {
  name: string;
  type: "dark" | "light";
  colors: Record<string, string>;
}

export interface EmberTheme {
  id: string;
  name: string;
  kind: "dark" | "light";
  /** CSS variable map applied to :root */
  vars: Record<string, string>;
  /** xterm.js ITheme-compatible palette */
  xterm: XtermPalette;
}

export interface XtermPalette {
  background: string;
  foreground: string;
  cursor: string;
  cursorAccent: string;
  black: string;
  red: string;
  green: string;
  yellow: string;
  blue: string;
  magenta: string;
  cyan: string;
  white: string;
  brightBlack: string;
  brightRed: string;
  brightGreen: string;
  brightYellow: string;
  brightBlue: string;
  brightMagenta: string;
  brightCyan: string;
  brightWhite: string;
}

export function parseVscodeTheme(json: string): VsCodeTheme {
  return JSON.parse(json) as VsCodeTheme;
}

/** Convert a VS Code-format theme JSON into an EmberTheme. */
export function vscodeToTheme(vsc: VsCodeTheme, id?: string): EmberTheme {
  const c = vsc.colors;
  const get = (key: string, fallback: string) => c[key] ?? fallback;

  const bg = get("editor.background", "#0d1117");
  const fg = get("terminal.foreground", "#e6edf3");
  const elevatedBg = get("sideBar.background", "#010409");
  const border = get("sideBar.border", "#21262d");
  const accent = get("focusBorder", "#FF6B35");
  const danger = get("errorForeground", "#f85149");
  const statusBg = get("statusBar.background", elevatedBg);
  const statusFg = get("statusBar.foreground", "#7d8590");
  const tabActiveBg = get("tab.activeBackground", bg);
  const tabInactiveBg = get("tab.inactiveBackground", elevatedBg);
  const tabActiveFg = get("tab.activeForeground", fg);
  const tabInactiveFg = get("tab.inactiveForeground", "#7d8590");

  return {
    id: id ?? vsc.name.toLowerCase().replace(/\s+/g, "-"),
    name: vsc.name,
    kind: vsc.type,
    vars: {
      "--bg": bg,
      "--bg-elevated": elevatedBg,
      "--border": border,
      "--fg": fg,
      "--fg-muted": tabInactiveFg,
      "--accent": accent,
      "--danger": danger,
      "--status-bg": statusBg,
      "--status-fg": statusFg,
      "--tab-active-bg": tabActiveBg,
      "--tab-inactive-bg": tabInactiveBg,
      "--tab-active-fg": tabActiveFg,
      "--tab-inactive-fg": tabInactiveFg,
    },
    xterm: {
      background: get("terminal.background", bg),
      foreground: fg,
      cursor: get("terminalCursor.foreground", fg),
      cursorAccent: get("terminalCursor.background", bg),
      black: get("terminal.ansiBlack", "#484f58"),
      red: get("terminal.ansiRed", "#ff7b72"),
      green: get("terminal.ansiGreen", "#7ee787"),
      yellow: get("terminal.ansiYellow", "#d29922"),
      blue: get("terminal.ansiBlue", "#58a6ff"),
      magenta: get("terminal.ansiMagenta", "#bc8cff"),
      cyan: get("terminal.ansiCyan", "#39c5cf"),
      white: get("terminal.ansiWhite", "#b1bac4"),
      brightBlack: get("terminal.ansiBrightBlack", "#6e7681"),
      brightRed: get("terminal.ansiBrightRed", "#ffa198"),
      brightGreen: get("terminal.ansiBrightGreen", "#56d364"),
      brightYellow: get("terminal.ansiBrightYellow", "#e3b341"),
      brightBlue: get("terminal.ansiBrightBlue", "#79c0ff"),
      brightMagenta: get("terminal.ansiBrightMagenta", "#d2a8ff"),
      brightCyan: get("terminal.ansiBrightCyan", "#56d4dd"),
      brightWhite: get("terminal.ansiBrightWhite", "#f0f6fc"),
    },
  };
}

/** Apply an EmberTheme's CSS vars to :root. */
export function applyThemeVars(theme: EmberTheme) {
  const root = document.documentElement;
  for (const [key, value] of Object.entries(theme.vars)) {
    root.style.setProperty(key, value);
  }
}
