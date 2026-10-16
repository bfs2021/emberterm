/** What the persisted config says about the theme. */
export interface ThemeConfig {
  themeId: string | null;
  /** VS Code theme JSON the user imported, verbatim. */
  customJson: string[];
}

export function parseThemeConfig(cfg: unknown): ThemeConfig {
  const a = (cfg as { appearance?: { theme?: unknown; custom_themes?: unknown } } | undefined)?.appearance;
  const custom = a?.custom_themes;
  return {
    themeId: typeof a?.theme === "string" ? a.theme : null,
    customJson: Array.isArray(custom) ? custom.filter((x): x is string => typeof x === "string") : [],
  };
}

/** The saved theme if it still exists, otherwise the fallback. */
export function resolveThemeId(saved: string | null, available: string[], fallback: string): string {
  return saved && available.includes(saved) ? saved : fallback;
}
