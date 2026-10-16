export interface InstalledFont {
  name: string;
  monospaced: boolean;
}

/** First family of a CSS font-family stack, without quotes. */
export function primaryFamily(stack: string): string {
  const first = stack.split(",")[0] ?? "";
  return first.trim().replace(/^["']|["']$/g, "");
}

/** A stack for one chosen family, falling back to the generic monospace face. */
export function stackFor(name: string): string {
  return `"${name.replace(/["']/g, "")}", monospace`;
}

/**
 * Fonts to offer in the picker: monospaced ones (terminals need fixed-width glyphs) unless
 * `showAll`; the font currently in use is always included so the select can show it.
 */
export function pickableFamilies(all: InstalledFont[], showAll: boolean, current: string): InstalledFont[] {
  return all.filter((f) => showAll || f.monospaced || f.name === current);
}
