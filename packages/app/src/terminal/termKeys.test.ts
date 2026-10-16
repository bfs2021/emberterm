import { describe, it, expect } from "vitest";
import { isCopyChord, isPasteChord, isFindChord } from "./termKeys.js";

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

describe("isPasteChord", () => {
  it("Ctrl+V and Ctrl+Shift+V paste", () => {
    expect(isPasteChord(key({ code: "KeyV" }))).toBe(true);
    expect(isPasteChord(key({ code: "KeyV", shiftKey: true }))).toBe(true);
  });
  it("ignores other keys, key-up and Alt chords", () => {
    expect(isPasteChord(key({ code: "KeyC" }))).toBe(false);
    expect(isPasteChord(key({ code: "KeyV", type: "keyup" }))).toBe(false);
    expect(isPasteChord(key({ code: "KeyV", altKey: true }))).toBe(false);
    expect(isPasteChord(key({ code: "KeyV", ctrlKey: false }))).toBe(false);
  });
});

describe("isFindChord", () => {
  it("only Ctrl+Shift+F opens find; plain Ctrl+F stays the shell's", () => {
    expect(isFindChord(key({ code: "KeyF", shiftKey: true }))).toBe(true);
    expect(isFindChord(key({ code: "KeyF" }))).toBe(false);
    expect(isFindChord(key({ code: "KeyF", shiftKey: true, type: "keyup" }))).toBe(false);
    expect(isFindChord(key({ code: "KeyF", shiftKey: true, altKey: true }))).toBe(false);
  });
});
