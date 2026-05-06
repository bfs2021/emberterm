import { describe, it, expect } from "vitest";
import { stripAnsi } from "./stripAnsi.js";

describe("stripAnsi", () => {
  // ── No-op on clean strings ─────────────────────────────────────────────────

  it("returns plain text unchanged", () => {
    expect(stripAnsi("hello world")).toBe("hello world");
  });

  it("preserves tab and newline", () => {
    expect(stripAnsi("a\tb\nc")).toBe("a\tb\nc");
  });

  it("handles empty string", () => {
    expect(stripAnsi("")).toBe("");
  });

  // ── CSI sequences ──────────────────────────────────────────────────────────

  it("strips SGR bold", () => {
    expect(stripAnsi("\x1b[1mhello\x1b[0m")).toBe("hello");
  });

  it("strips 256-colour foreground", () => {
    expect(stripAnsi("\x1b[38;5;214mtext\x1b[0m")).toBe("text");
  });

  it("strips cursor movement (A/B/C/D)", () => {
    expect(stripAnsi("\x1b[2Aup")).toBe("up");
    expect(stripAnsi("\x1b[10Bright")).toBe("right");
  });

  it("strips erase in display", () => {
    expect(stripAnsi("\x1b[2J")).toBe("");
  });

  // ── OSC sequences ──────────────────────────────────────────────────────────

  it("strips OSC 0 title (BEL-terminated)", () => {
    expect(stripAnsi("\x1b]0;My Title\x07rest")).toBe("rest");
  });

  it("strips OSC 7 cwd (ST-terminated)", () => {
    expect(stripAnsi("\x1b]7;file:///home/user\x1b\\rest")).toBe("rest");
  });

  it("strips embedded OSC 7777 sequence", () => {
    const evil = "\x1b]7777;tool.start;{}\x07summary";
    expect(stripAnsi(evil)).toBe("summary");
  });

  // ── Single-char ESC sequences ──────────────────────────────────────────────

  it("strips ESC M (reverse index)", () => {
    expect(stripAnsi("\x1bMtext")).toBe("text");
  });

  it("strips ESC at end of string", () => {
    // Lone ESC with no following char is consumed entirely.
    expect(stripAnsi("text\x1b")).toBe("text");
  });

  it("strips ESC Fe sequences — ESC + letter is a valid Fe escape", () => {
    // ESC + 't' is a valid Fe sequence; both chars are stripped.
    expect(stripAnsi("\x1bt" + "more")).toBe("more");
  });

  // ── Raw C0 control codes ───────────────────────────────────────────────────

  it("strips NUL", () => {
    expect(stripAnsi("a\x00b")).toBe("ab");
  });

  it("strips BEL", () => {
    // BEL (0x07) used as OSC terminator — when standalone in a payload it's noise
    expect(stripAnsi("a\x07b")).toBe("ab");
  });

  it("strips backspace", () => {
    expect(stripAnsi("a\x08b")).toBe("ab");
  });

  it("strips carriage return (\\x0d)", () => {
    expect(stripAnsi("a\rb")).toBe("ab");
  });

  // ── Mixed real-world examples ──────────────────────────────────────────────

  it("strips colours from a tool summary line", () => {
    const coloured = "\x1b[32mEdited\x1b[0m src/main.rs — \x1b[33m+12/-3\x1b[0m lines";
    expect(stripAnsi(coloured)).toBe("Edited src/main.rs — +12/-3 lines");
  });

  it("strips a malicious embedded OSC from a summary field", () => {
    // An attacker-supplied summary that tries to inject an OSC 52 clipboard write
    const malicious = "Safe text\x1b]52;c;aGVsbG8=\x07 done";
    expect(stripAnsi(malicious)).toBe("Safe text done");
  });

  it("handles multiple sequences in one string", () => {
    const s = "\x1b[1m\x1b[31mERROR\x1b[0m: something went wrong";
    expect(stripAnsi(s)).toBe("ERROR: something went wrong");
  });
});
