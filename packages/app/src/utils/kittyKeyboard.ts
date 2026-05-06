/**
 * Kitty keyboard protocol encoder.
 * Spec: https://sw.kovidgoyal.net/kitty/keyboard-protocol/
 *
 * When an application running in the PTY pushes Kitty keyboard mode via
 *   ESC [ > <flags> u
 * the terminal must encode key events in the enhanced Kitty format instead of
 * the traditional VT/xterm encoding. This disambiguates combinations like
 * Ctrl+I vs Tab, Ctrl+M vs Enter, Ctrl+[ vs Escape, etc.
 */

// ── Kitty flag constants ────────────────────────────────────────────────────

/** Disambiguate Escape (Esc vs modifier prefix), Ctrl+letter from control chars. */
export const FLAG_DISAMBIGUATE = 0x1;
/** Report key repeat and release events in addition to press. */
export const FLAG_REPORT_EVENTS = 0x2;
/** Report alternate key symbols (shifted chars, etc.) */
export const FLAG_REPORT_ALTERNATE = 0x4;
/** Report all keys as escape codes, never as text. */
export const FLAG_REPORT_ALL = 0x8;
/** Report associated Unicode code point for key events. */
export const FLAG_REPORT_ASSOCIATED = 0x10;

// ── Special key codepoints ─────────────────────────────────────────────────

// Keys that have no Unicode codepoint but exist in the Kitty protocol.
// Values sourced from the Kitty keyboard protocol spec (functional keys).
const FUNCTIONAL_KEYS: Record<string, number> = {
  Escape: 27,
  Enter: 13,
  Tab: 9,
  Backspace: 127,
  Insert: 57348,
  Delete: 57349,
  ArrowLeft: 57350,
  ArrowRight: 57351,
  ArrowUp: 57352,
  ArrowDown: 57353,
  PageUp: 57354,
  PageDown: 57355,
  Home: 57356,
  End: 57357,
  CapsLock: 57358,
  ScrollLock: 57359,
  NumLock: 57360,
  PrintScreen: 57361,
  Pause: 57362,
  Menu: 57363,
  F1: 57364,
  F2: 57365,
  F3: 57366,
  F4: 57367,
  F5: 57368,
  F6: 57369,
  F7: 57370,
  F8: 57371,
  F9: 57372,
  F10: 57373,
  F11: 57374,
  F12: 57375,
  F13: 57376,
  F14: 57377,
  F15: 57378,
  F16: 57379,
  F17: 57380,
  F18: 57381,
  F19: 57382,
  F20: 57383,
};

// ── Modifier encoding ───────────────────────────────────────────────────────

/**
 * Encode keyboard modifiers in Kitty format.
 * Kitty modifier = (shift | alt<<1 | ctrl<<2 | super<<3 | ...) + 1
 * The +1 is so that "no modifier" = 1, not 0.
 */
function kittyMod(e: KeyboardEvent): number {
  let mod = 0;
  if (e.shiftKey) mod |= 1;
  if (e.altKey) mod |= 2;
  if (e.ctrlKey) mod |= 4;
  if (e.metaKey) mod |= 8;
  return mod + 1;
}

function hasModifier(e: KeyboardEvent): boolean {
  return e.shiftKey || e.altKey || e.ctrlKey || e.metaKey;
}

// ── Public API ──────────────────────────────────────────────────────────────

/**
 * Encode a keyboard event as a Kitty protocol escape sequence.
 *
 * Returns null when:
 * - This is a plain printable character with no modifier (let xterm.js handle it)
 * - This is a modifier-only key press (Shift, Ctrl, Alt, Meta alone)
 *
 * Otherwise returns the Kitty-encoded bytes to write to the PTY.
 *
 * @param e   The browser KeyboardEvent (keydown or keyup)
 * @param flags  Current Kitty keyboard flags bitmask
 * @param eventType  1 = press, 2 = repeat, 3 = release
 */
export function encodeKittyKey(
  e: KeyboardEvent,
  flags: number,
  eventType: 1 | 2 | 3 = 1
): Uint8Array | null {
  const { key } = e;

  // Ignore bare modifier keys — they don't emit key events.
  if (["Shift", "Control", "Alt", "Meta", "Hyper", "Super", "CapsLock", "NumLock", "ScrollLock"].includes(key)) {
    return null;
  }

  const mod = kittyMod(e);
  const noMod = mod === 1; // No modifier set (1 = Kitty base value)

  // ── Plain printable with no modifier ──────────────────────────────────────
  // With FLAG_DISAMBIGUATE only, printable characters without any modifier
  // are still sent as plain UTF-8 bytes. Only ambiguous combos use Kitty format.
  // Exception: if FLAG_REPORT_ALL is active, everything uses Kitty format.
  if (noMod && key.length === 1 && !(flags & FLAG_REPORT_ALL)) {
    return null; // xterm.js handles printable chars
  }

  // ── Functional / special keys ─────────────────────────────────────────────
  const funcCode = FUNCTIONAL_KEYS[key];
  if (funcCode !== undefined) {
    return buildKittySequence(funcCode, mod, flags, eventType);
  }

  // ── Single printable character with modifier ──────────────────────────────
  if (key.length === 1) {
    const codepoint = key.codePointAt(0)!;
    return buildKittySequence(codepoint, mod, flags, eventType);
  }

  // ── Multi-character keys (not encodable in Kitty v1) ─────────────────────
  return null;
}

/** Build the CSI u Kitty sequence for a given codepoint + modifier. */
function buildKittySequence(
  codepoint: number,
  mod: number,
  flags: number,
  eventType: 1 | 2 | 3
): Uint8Array {
  let seq: string;

  const reportEventTypes = !!(flags & FLAG_REPORT_EVENTS);

  if (mod === 1 && (!reportEventTypes || eventType === 1)) {
    // No modifier, no event type suffix needed
    seq = `\x1b[${codepoint}u`;
  } else if (reportEventTypes) {
    seq = `\x1b[${codepoint};${mod}:${eventType}u`;
  } else {
    seq = `\x1b[${codepoint};${mod}u`;
  }

  return new TextEncoder().encode(seq);
}

/**
 * Returns true when the given key event should be intercepted from xterm.js
 * and encoded in Kitty format instead.
 *
 * Called from the `attachCustomKeyEventHandler` callback. When true,
 * the caller must call `encodeKittyKey` and write the result to the PTY,
 * then return `false` to suppress xterm.js default handling.
 */
export function shouldInterceptKey(e: KeyboardEvent, flags: number): boolean {
  if (!flags) return false;
  if (e.type !== "keydown" && e.type !== "keyup") return false;

  const { key } = e;

  // Never intercept bare modifier keys.
  if (["Shift", "Control", "Alt", "Meta", "Hyper", "Super", "CapsLock", "NumLock", "ScrollLock"].includes(key)) {
    return false;
  }

  // Keyup events: only intercept if FLAG_REPORT_EVENTS is active.
  if (e.type === "keyup") {
    return !!(flags & FLAG_REPORT_EVENTS);
  }

  // Always intercept functional keys (arrows, F-keys, etc.).
  if (FUNCTIONAL_KEYS[key] !== undefined) return true;

  // Intercept printable keys with any modifier (disambiguate mode).
  if (key.length === 1 && hasModifier(e)) return true;

  // FLAG_REPORT_ALL: intercept everything.
  if (flags & FLAG_REPORT_ALL) return true;

  return false;
}
