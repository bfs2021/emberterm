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
  const pick = (key: string, fallback: string) => c[key] ?? fallback;

  const bg = pick("editor.background", "#0d1117");
  const fg = pick("terminal.foreground", "#e6edf3");
  const bgElev = pick("sideBar.background", "#010409");
  const border = pick("sideBar.border", "#21262d");
  const accent = pick("focusBorder", "#FF6B35");
  const danger = pick("errorForeground", "#f85149");
  const fgDim = pick("terminal.ansiBlack", "#484f58");
  const borderSubtle = pick("panel.border", `color-mix(in srgb, ${border} 60%, transparent)`);

  // Semantic status colors mapped from ANSI palette
  const green = pick("terminal.ansiBrightGreen", "#3fb950");
  const red = pick("terminal.ansiRed", "#ff7b72");
  const yellow = pick("terminal.ansiYellow", "#d29922");
  const blue = pick("terminal.ansiBlue", "#58a6ff");
  const cyan = pick("terminal.ansiCyan", "#39c5cf");
  const magenta = pick("terminal.ansiMagenta", "#bc8cff");

  const statusBg = pick("statusBar.background", bgElev);
  const statusFg = pick("statusBar.foreground", "#7d8590");
  const tabActiveBg = pick("tab.activeBackground", bg);
  const tabInactiveBg = pick("tab.inactiveBackground", bgElev);
  const tabActiveFg = pick("tab.activeForeground", fg);
  const tabInactiveFg = pick("tab.inactiveForeground", "#7d8590");

  // Computed tokens — CSS color-mix() references resolved at paint time
  const bgElev2 = pick("editorWidget.background", `color-mix(in srgb, ${bg} 85%, ${bgElev})`);
  const bgBlock = pick("editor.lineHighlightBackground", `color-mix(in srgb, ${bg} 92%, ${bgElev})`);
  const accentSoft = `color-mix(in srgb, ${accent} 12%, transparent)`;
  const accentGlow = `color-mix(in srgb, ${accent} 35%, transparent)`;

  // Diff line colors — transparent overlays, theme-adaptive
  const diffAddBg = `color-mix(in srgb, ${green} 12%, transparent)`;
  const diffAddLine = `color-mix(in srgb, ${green} 35%, transparent)`;
  const diffDelBg = `color-mix(in srgb, ${danger} 12%, transparent)`;
  const diffDelLine = `color-mix(in srgb, ${danger} 35%, transparent)`;

  return {
    id: id ?? vsc.name.toLowerCase().replace(/\s+/g, "-"),
    name: vsc.name,
    kind: vsc.type,
    vars: {
      "--bg": bg,
      "--bg-elev": bgElev,
      "--bg-elev-2": bgElev2,
      "--bg-block": bgBlock,
      "--border": border,
      "--border-subtle": borderSubtle,
      "--fg": fg,
      "--fg-muted": tabInactiveFg,
      "--fg-dim": fgDim,
      "--accent": accent,
      "--accent-soft": accentSoft,
      "--accent-glow": accentGlow,
      "--danger": danger,
      "--green": green,
      "--red": red,
      "--yellow": yellow,
      "--blue": blue,
      "--cyan": cyan,
      "--magenta": magenta,
      "--diff-add-bg": diffAddBg,
      "--diff-add-line": diffAddLine,
      "--diff-del-bg": diffDelBg,
      "--diff-del-line": diffDelLine,
      "--radius-sm": "6px",
      "--radius": "10px",
      "--radius-lg": "14px",
      "--shadow-lg": "0 24px 60px -12px rgba(0,0,0,0.55), 0 0 0 1px rgba(255,255,255,0.04)",
      "--status-bg": statusBg,
      "--status-fg": statusFg,
      "--tab-active-bg": tabActiveBg,
      "--tab-inactive-bg": tabInactiveBg,
      "--tab-active-fg": tabActiveFg,
      "--tab-inactive-fg": tabInactiveFg,
    },
    xterm: {
      background: pick("terminal.background", bg),
      foreground: fg,
      cursor: pick("terminalCursor.foreground", fg),
      cursorAccent: pick("terminalCursor.background", bg),
      black: pick("terminal.ansiBlack", "#484f58"),
      red: pick("terminal.ansiRed", "#ff7b72"),
      green: pick("terminal.ansiGreen", "#7ee787"),
      yellow: pick("terminal.ansiYellow", "#d29922"),
      blue: pick("terminal.ansiBlue", "#58a6ff"),
      magenta: pick("terminal.ansiMagenta", "#bc8cff"),
      cyan: pick("terminal.ansiCyan", "#39c5cf"),
      white: pick("terminal.ansiWhite", "#b1bac4"),
      brightBlack: pick("terminal.ansiBrightBlack", "#6e7681"),
      brightRed: pick("terminal.ansiBrightRed", "#ffa198"),
      brightGreen: pick("terminal.ansiBrightGreen", "#56d364"),
      brightYellow: pick("terminal.ansiBrightYellow", "#e3b341"),
      brightBlue: pick("terminal.ansiBrightBlue", "#79c0ff"),
      brightMagenta: pick("terminal.ansiBrightMagenta", "#d2a8ff"),
      brightCyan: pick("terminal.ansiBrightCyan", "#56d4dd"),
      brightWhite: pick("terminal.ansiBrightWhite", "#f0f6fc"),
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
