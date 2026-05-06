import { describe, test, expect } from "vitest";
import {
  encodeKittyKey,
  shouldInterceptKey,
  FLAG_DISAMBIGUATE,
  FLAG_REPORT_EVENTS,
  FLAG_REPORT_ALL,
} from "./kittyKeyboard.js";

// Helper: build a minimal KeyboardEvent-like object.
function mockKey(
  key: string,
  opts: Partial<Pick<KeyboardEvent, "ctrlKey" | "shiftKey" | "altKey" | "metaKey" | "repeat" | "type">> = {}
): KeyboardEvent {
  return {
    key,
    type: opts.type ?? "keydown",
    ctrlKey: opts.ctrlKey ?? false,
    shiftKey: opts.shiftKey ?? false,
    altKey: opts.altKey ?? false,
    metaKey: opts.metaKey ?? false,
    repeat: opts.repeat ?? false,
  } as KeyboardEvent;
}

function decode(bytes: Uint8Array): string {
  return new TextDecoder().decode(bytes);
}

// ── encodeKittyKey ────────────────────────────────────────────────────────────

describe("encodeKittyKey", () => {
  test("plain printable with no modifier returns null (let xterm handle)", () => {
    expect(encodeKittyKey(mockKey("a"), FLAG_DISAMBIGUATE)).toBeNull();
    expect(encodeKittyKey(mockKey("z"), FLAG_DISAMBIGUATE)).toBeNull();
    expect(encodeKittyKey(mockKey("1"), FLAG_DISAMBIGUATE)).toBeNull();
  });

  test("Enter (functional key) with no modifier → CSI 13 u", () => {
    const result = encodeKittyKey(mockKey("Enter"), FLAG_DISAMBIGUATE);
    expect(decode(result!)).toBe("\x1b[13u");
  });

  test("Tab with no modifier → CSI 9 u", () => {
    const result = encodeKittyKey(mockKey("Tab"), FLAG_DISAMBIGUATE);
    expect(decode(result!)).toBe("\x1b[9u");
  });

  test("Escape with no modifier → CSI 27 u", () => {
    const result = encodeKittyKey(mockKey("Escape"), FLAG_DISAMBIGUATE);
    expect(decode(result!)).toBe("\x1b[27u");
  });

  test("ArrowUp with no modifier → CSI 57352 u", () => {
    const result = encodeKittyKey(mockKey("ArrowUp"), FLAG_DISAMBIGUATE);
    expect(decode(result!)).toBe("\x1b[57352u");
  });

  test("Ctrl+I distinguishable from Tab: CSI 105 ; 5 u", () => {
    // 'i' has codepoint 105; Ctrl modifier = 4+1 = 5
    const result = encodeKittyKey(mockKey("i", { ctrlKey: true }), FLAG_DISAMBIGUATE);
    expect(decode(result!)).toBe("\x1b[105;5u");
  });

  test("Ctrl+M distinguishable from Enter: CSI 109 ; 5 u", () => {
    const result = encodeKittyKey(mockKey("m", { ctrlKey: true }), FLAG_DISAMBIGUATE);
    expect(decode(result!)).toBe("\x1b[109;5u");
  });

  test("Shift+Tab → CSI 9 ; 2 u", () => {
    // Shift modifier = 1+1 = 2
    const result = encodeKittyKey(mockKey("Tab", { shiftKey: true }), FLAG_DISAMBIGUATE);
    expect(decode(result!)).toBe("\x1b[9;2u");
  });

  test("F1 with no modifier → CSI 57364 u", () => {
    const result = encodeKittyKey(mockKey("F1"), FLAG_DISAMBIGUATE);
    expect(decode(result!)).toBe("\x1b[57364u");
  });

  test("F12 with Ctrl → CSI 57375 ; 5 u", () => {
    const result = encodeKittyKey(mockKey("F12", { ctrlKey: true }), FLAG_DISAMBIGUATE);
    expect(decode(result!)).toBe("\x1b[57375;5u");
  });

  test("bare modifier keys return null", () => {
    expect(encodeKittyKey(mockKey("Shift", { shiftKey: true }), FLAG_DISAMBIGUATE)).toBeNull();
    expect(encodeKittyKey(mockKey("Control", { ctrlKey: true }), FLAG_DISAMBIGUATE)).toBeNull();
    expect(encodeKittyKey(mockKey("Alt", { altKey: true }), FLAG_DISAMBIGUATE)).toBeNull();
  });

  test("with FLAG_REPORT_EVENTS: keyup uses event type 3", () => {
    const result = encodeKittyKey(
      mockKey("Enter", { type: "keyup" }),
      FLAG_REPORT_EVENTS,
      3
    );
    // mod=1 (no modifier), event_type=3 → CSI 13 ; 1 : 3 u
    expect(decode(result!)).toBe("\x1b[13;1:3u");
  });

  test("with FLAG_REPORT_ALL: plain printable char is encoded", () => {
    const result = encodeKittyKey(mockKey("a"), FLAG_REPORT_ALL);
    // 'a' = 97, no modifier (mod=1) → CSI 97 u
    expect(decode(result!)).toBe("\x1b[97u");
  });
});

// ── shouldInterceptKey ────────────────────────────────────────────────────────

describe("shouldInterceptKey", () => {
  test("returns false when flags=0 (Kitty mode off)", () => {
    expect(shouldInterceptKey(mockKey("ArrowUp"), 0)).toBe(false);
    expect(shouldInterceptKey(mockKey("a", { ctrlKey: true }), 0)).toBe(false);
  });

  test("functional keys are always intercepted in Kitty mode", () => {
    expect(shouldInterceptKey(mockKey("ArrowLeft"), FLAG_DISAMBIGUATE)).toBe(true);
    expect(shouldInterceptKey(mockKey("F5"), FLAG_DISAMBIGUATE)).toBe(true);
    expect(shouldInterceptKey(mockKey("Enter"), FLAG_DISAMBIGUATE)).toBe(true);
    expect(shouldInterceptKey(mockKey("Tab"), FLAG_DISAMBIGUATE)).toBe(true);
  });

  test("printable with modifier is intercepted", () => {
    expect(shouldInterceptKey(mockKey("i", { ctrlKey: true }), FLAG_DISAMBIGUATE)).toBe(true);
    expect(shouldInterceptKey(mockKey("a", { altKey: true }), FLAG_DISAMBIGUATE)).toBe(true);
  });

  test("plain printable without modifier is NOT intercepted (xterm handles it)", () => {
    expect(shouldInterceptKey(mockKey("a"), FLAG_DISAMBIGUATE)).toBe(false);
    expect(shouldInterceptKey(mockKey("z"), FLAG_DISAMBIGUATE)).toBe(false);
  });

  test("keyup is NOT intercepted unless FLAG_REPORT_EVENTS", () => {
    const up = mockKey("Enter", { type: "keyup" });
    expect(shouldInterceptKey(up, FLAG_DISAMBIGUATE)).toBe(false);
    expect(shouldInterceptKey(up, FLAG_DISAMBIGUATE | FLAG_REPORT_EVENTS)).toBe(true);
  });

  test("bare modifier keys are never intercepted", () => {
    expect(shouldInterceptKey(mockKey("Shift", { shiftKey: true }), FLAG_DISAMBIGUATE)).toBe(false);
    expect(shouldInterceptKey(mockKey("Control", { ctrlKey: true }), FLAG_DISAMBIGUATE | FLAG_REPORT_EVENTS)).toBe(false);
  });
});
