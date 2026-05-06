/**
 * stripAnsi — remove ANSI / terminal control sequences from a string.
 *
 * Agent-supplied OSC 7777 payloads (summary, progressMessage, name, etc.)
 * are rendered as DOM text.  If an agent embeds escape sequences — whether
 * accidentally or maliciously — they could confuse screen readers, trigger
 * parser edge-cases in downstream tooling, or (on browsers with legacy
 * rendering paths) cause unexpected output.  We strip them here before any
 * payload string reaches the DOM.
 *
 * Covers:
 *  - CSI sequences   ESC [ … final-byte
 *  - OSC sequences   ESC ] … ST  (ST = ESC \ or BEL)
 *  - DCS sequences   ESC P … ST
 *  - APC sequences   ESC _ … ST
 *  - PM  sequences   ESC ^ … ST
 *  - SOS sequences   ESC X … ST
 *  - Single ESC + single char (e.g. ESC M, ESC 7)
 *  - Raw C0 control codes (0x00–0x1F) except tab (0x09) and newline (0x0A)
 */

// Matches the broadest set of escape sequences + raw control characters.
// Written as a single alternation so one pass cleans everything.
const ANSI_RE = new RegExp(
  [
    // ESC-introduced multi-byte sequences (CSI, OSC, DCS, APC, PM, SOS)
    // CSI: ESC [ + optional param/intermediate bytes + final byte
    "\x1b\\[[\\x30-\\x3f]*[\\x20-\\x2f]*[\\x40-\\x7e]",
    // String-terminator sequences: ESC followed by ] P X ^ _ then chars until ST or BEL
    "\x1b[\\]PX\\^_][^\x1b\x07]*(?:\x07|\x1b\\\\)",
    // Lone ESC + any single char (Fe/Fp sequences like ESC M, ESC 7, ESC \\)
    "\x1b[^\\[\\]PX\\^_]?",
    // Raw C0 control codes (keep 0x09=tab, 0x0A=newline as benign)
    // Range \x0b-\x1f covers VT(0x0b), FF(0x0c), CR(0x0d), SO(0x0e)…US(0x1f).
    "[\\x00-\\x08\\x0b-\\x1f\\x7f]",
  ].join("|"),
  "g",
);

/**
 * Return `s` with all ANSI/control sequences removed.
 * The input is treated as a plain string; the result is safe for DOM text.
 */
export function stripAnsi(s: string): string {
  return s.replace(ANSI_RE, "");
}
