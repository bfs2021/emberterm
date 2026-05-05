import { writable, derived } from "svelte/store";
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

export function setTheme(id: string) {
  activeThemeId.set(id);
}

export function importTheme(json: string): EmberTheme | null {
  try {
    const vsc = JSON.parse(json) as VsCodeTheme;
    const theme = vscodeToTheme(vsc);
    customThemes.update((ts) => {
      const idx = ts.findIndex((t) => t.id === theme.id);
      if (idx >= 0) {
        ts[idx] = theme;
        return [...ts];
      }
      return [...ts, theme];
    });
    return theme;
  } catch {
    return null;
  }
}
