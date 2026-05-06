import { writable, get } from "svelte/store";
import { configGet, configUpdate, setWindowOpacity } from "../ipc/client.js";

// ── Window opacity ───────────────────────────────────────────────────────────

/** 0–1 float; drives xterm.js rgba background + CSS variable for chrome. */
export const windowOpacity = writable(1);


/**
 * Apply a new opacity value:
 *   - updates the reactive store (Grid.svelte reacts to it live)
 *   - sets --window-opacity on :root for any CSS consumers
 *   - calls setWindowOpacity() IPC to apply OS-level transparency via
 *     SetLayeredWindowAttributes (Windows) so the entire compositor surface —
 *     including WebGL canvases — becomes transparent at once
 */
export function applyOpacity(v: number): void {
  windowOpacity.set(v);
  document.documentElement.style.setProperty("--window-opacity", String(v));
  // OS-level window transparency — title bar, status bar, and xterm.js WebGL
  // canvas all become transparent together without any double-fade artefacts.
  setWindowOpacity(v).catch(() => {});
}

/**
 * Temporarily override the OS window opacity without changing the stored
 * user preference. Used to restore full opacity while modal overlays are open
 * so they remain readable regardless of the configured transparency level.
 */
export function setTransientOpacity(v: number): void {
  setWindowOpacity(v).catch(() => {});
}

export interface TerminalConfig {
  /** "block" | "bar" | "underline" */
  cursorStyle: string;
  cursorBlink: boolean;
  /** Draw bold ANSI text using bright color variants */
  boldInBright: boolean;
  scrollbackLines: number;
  confirmCloseRunning: boolean;
}

/** Defaults mirror config.rs defaults. */
const DEFAULTS: TerminalConfig = {
  cursorStyle: "block",
  cursorBlink: true,
  boldInBright: true,
  scrollbackLines: 10000,
  confirmCloseRunning: true,
};

export const terminalConfig = writable<TerminalConfig>({ ...DEFAULTS });

/** Pull terminal settings from the persisted config and sync into the store. */
export async function loadTerminalConfig(): Promise<void> {
  try {
    const cfg = await configGet() as {
      appearance?: {
        cursor_style?: string;
        cursor_blink?: boolean;
        bold_in_bright?: boolean;
      };
      behavior?: {
        scrollback_lines?: number;
        confirm_close_running?: boolean;
      };
    };
    const a = cfg?.appearance;
    const b = cfg?.behavior;
    terminalConfig.set({
      cursorStyle: a?.cursor_style ?? DEFAULTS.cursorStyle,
      cursorBlink: a?.cursor_blink ?? DEFAULTS.cursorBlink,
      boldInBright: a?.bold_in_bright ?? DEFAULTS.boldInBright,
      scrollbackLines: b?.scrollback_lines ?? DEFAULTS.scrollbackLines,
      confirmCloseRunning: b?.confirm_close_running ?? DEFAULTS.confirmCloseRunning,
    });
  } catch {
    // Keep defaults if config isn't available yet
  }
}

/**
 * Update one or more terminal settings, apply immediately (live preview),
 * and persist to config.toml in the background.
 */
export async function updateTerminalConfig(patch: Partial<TerminalConfig>): Promise<void> {
  const updated = { ...get(terminalConfig), ...patch };
  terminalConfig.set(updated);

  await configUpdate({
    appearance: {
      cursor_style: updated.cursorStyle,
      cursor_blink: updated.cursorBlink,
      bold_in_bright: updated.boldInBright,
    },
    behavior: {
      scrollback_lines: updated.scrollbackLines,
      confirm_close_running: updated.confirmCloseRunning,
    },
  });
}
