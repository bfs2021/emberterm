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
 * Fonts to offer in the picker: fixed-width ones only (terminals need them). The font
 * currently in use is always included so the select can show it.
 */
export function pickableFamilies(all: InstalledFont[], current: string): InstalledFont[] {
  return all.filter((f) => f.monospaced || f.name === current);
}
