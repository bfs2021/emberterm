import { describe, it, expect } from "vitest";
import { isCopyChord } from "./copyKey.js";

const key = (over: Partial<Parameters<typeof isCopyChord>[0]>) => ({
  type: "keydown", code: "KeyC", ctrlKey: true, shiftKey: false, altKey: false, ...over,
});

describe("isCopyChord", () => {
  it("Ctrl+Shift+C is always a copy", () => {
    expect(isCopyChord(key({ shiftKey: true }), false)).toBe(true);
    expect(isCopyChord(key({ shiftKey: true }), true)).toBe(true);
  });

  it("Ctrl+C copies only when text is selected, otherwise it is the shell's interrupt", () => {
    expect(isCopyChord(key({}), true)).toBe(true);
    expect(isCopyChord(key({}), false)).toBe(false);
  });

  it("ignores other keys, key-up events and Alt chords", () => {
    expect(isCopyChord(key({ code: "KeyV", shiftKey: true }), true)).toBe(false);
    expect(isCopyChord(key({ type: "keyup", shiftKey: true }), true)).toBe(false);
    expect(isCopyChord(key({ altKey: true, shiftKey: true }), true)).toBe(false);
    expect(isCopyChord(key({ ctrlKey: false, shiftKey: true }), true)).toBe(false);
  });
});
