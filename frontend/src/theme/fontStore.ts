import { writable, get } from "svelte/store";
import { configGet, configUpdate } from "../ipc/client.js";

export interface FontConfig {
  fontFamily: string;
  fontSize: number;
  lineHeight: number;
}

/** Defaults mirror `AppearanceConfig::default()` in config.rs. */
const DEFAULTS: FontConfig = {
  fontFamily: '"JetBrains Mono", "Cascadia Code", "Fira Code", monospace',
  fontSize: 13,
  lineHeight: 1.2,
};

export const fontConfig = writable<FontConfig>({ ...DEFAULTS });

/** Pull font settings from the persisted config and sync into the store. */
export async function loadFontConfig(): Promise<void> {
  try {
    const cfg = await configGet() as {
      appearance?: { font_family?: string; font_size?: number; line_height?: number };
    };
    const a = cfg?.appearance;
    if (a) {
      fontConfig.set({
        fontFamily: a.font_family ?? DEFAULTS.fontFamily,
        fontSize: a.font_size ?? DEFAULTS.fontSize,
        lineHeight: a.line_height ?? DEFAULTS.lineHeight,
      });
    }
  } catch {
    // Keep defaults if config isn't available yet
  }
}

/**
 * Update one or more font settings, apply them to the store immediately (live
 * preview), and persist to config.toml in the background.
 */
export async function updateFontConfig(patch: Partial<FontConfig>): Promise<void> {
  const updated = { ...get(fontConfig), ...patch };
  fontConfig.set(updated);
  // Keys must be snake_case — serde serializes Rust struct fields without renaming.
  await configUpdate({
    appearance: {
      font_family: updated.fontFamily,
      font_size: updated.fontSize,
      line_height: updated.lineHeight,
    },
  });
}
