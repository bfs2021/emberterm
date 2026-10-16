import { writable, derived, get } from "svelte/store";
import { configGet, configUpdate } from "../ipc/client.js";
import { pushToast } from "../stores/app.js";
import { parseThemeConfig, resolveThemeId } from "./themeConfig.js";
import { vscodeToTheme, applyThemeVars, type EmberTheme, type VsCodeTheme } from "./vscode-import.js";

import emberDark from "./themes/ember-dark.json";
import emberLight from "./themes/ember-light.json";
import tokyoNight from "./themes/tokyo-night.json";
import catppuccinMocha from "./themes/catppuccin-mocha.json";
import dracula from "./themes/dracula.json";
import githubDark from "./themes/github-dark.json";
import solarizedDark from "./themes/solarized-dark.json";

const BUILT_IN_THEMES: EmberTheme[] = [
  vscodeToTheme(emberDark as VsCodeTheme, "ember-dark"),
  vscodeToTheme(emberLight as VsCodeTheme, "ember-light"),
  vscodeToTheme(tokyoNight as VsCodeTheme, "tokyo-night"),
  vscodeToTheme(catppuccinMocha as VsCodeTheme, "catppuccin-mocha"),
  vscodeToTheme(dracula as VsCodeTheme, "dracula"),
  vscodeToTheme(githubDark as VsCodeTheme, "github-dark"),
  vscodeToTheme(solarizedDark as VsCodeTheme, "solarized-dark"),
];

export const builtInThemes = BUILT_IN_THEMES;

export const activeThemeId = writable<string>("ember-dark");
export const customThemes = writable<EmberTheme[]>([]);

export const allThemes = derived(customThemes, ($custom) => [
  ...BUILT_IN_THEMES,
  ...$custom,
]);

export const activeTheme = derived(
  [activeThemeId, allThemes],
  ([$id, $all]) => $all.find((t) => t.id === $id) ?? BUILT_IN_THEMES[0],
);

// Whenever the active theme changes, apply its CSS vars to :root.
activeTheme.subscribe((theme) => {
  applyThemeVars(theme);
});

const DEFAULT_THEME_ID = "ember-dark";

// Imported themes are saved as the original JSON text and re-imported at startup.
const customSources = new Map<string, string>();

function persist(appearance: Record<string, unknown>) {
  configUpdate({ appearance }).catch((e) => pushToast(`Couldn't save the theme: ${e}`));
}

export function setTheme(id: string) {
  activeThemeId.set(id);
  persist({ theme: id });
}

function addCustomTheme(json: string): EmberTheme | null {
  try {
    const theme = vscodeToTheme(JSON.parse(json) as VsCodeTheme);
    customThemes.update((ts) => {
      const idx = ts.findIndex((t) => t.id === theme.id);
      if (idx >= 0) {
        ts[idx] = theme;
        return [...ts];
      }
      return [...ts, theme];
    });
    customSources.set(theme.id, json);
    return theme;
  } catch {
    return null;
  }
}

export function importTheme(json: string): EmberTheme | null {
  const theme = addCustomTheme(json);
  if (theme) persist({ custom_themes: [...customSources.values()] });
  return theme;
}

/** Restores the imported themes and the selected theme from config.toml (call once at startup). */
export async function loadThemeConfig(): Promise<void> {
  try {
    const { themeId, customJson } = parseThemeConfig(await configGet());
    customJson.forEach(addCustomTheme);
    const ids = get(allThemes).map((t) => t.id);
    activeThemeId.set(resolveThemeId(themeId, ids, DEFAULT_THEME_ID));
  } catch {
    // keep the default theme
  }
}
