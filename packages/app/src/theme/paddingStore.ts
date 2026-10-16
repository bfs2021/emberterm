import { writable, get } from "svelte/store";
import { configGet, configUpdate } from "../ipc/client.js";

/** Mirrors `window_padding` in config.rs. */
export const DEFAULT_PADDING = 8;
const MIN = 0;
const MAX = 64;

export function clampPadding(n: number, fallback = DEFAULT_PADDING): number {
  if (!Number.isFinite(n)) return fallback;
  return Math.min(MAX, Math.max(MIN, Math.round(n)));
}

/** Space between the terminal text and the edge of its pane, in px. */
export const terminalPadding = writable<number>(DEFAULT_PADDING);

// The terminal reads it as a CSS variable; keep the variable in step with the store.
terminalPadding.subscribe((n) => {
  if (typeof document !== "undefined") document.documentElement.style.setProperty("--term-pad", `${n}px`);
});

export async function loadPadding(): Promise<void> {
  try {
    const cfg = (await configGet()) as { appearance?: { window_padding?: number } };
    terminalPadding.set(clampPadding(cfg?.appearance?.window_padding ?? DEFAULT_PADDING));
  } catch {
    // keep the default
  }
}

export async function updatePadding(n: number): Promise<void> {
  const value = clampPadding(n, get(terminalPadding));
  terminalPadding.set(value);
  await configUpdate({ appearance: { window_padding: value } });
}
