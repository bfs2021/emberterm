import { describe, it, expect } from "vitest";
import { formatAction, btnClass, resolveKey } from "./approval-utils.js";

// ── formatAction ──────────────────────────────────────────────────────────────

describe("formatAction", () => {
  it("converts snake_case to Title Case", () => {
    expect(formatAction("allow_once")).toBe("Allow Once");
    expect(formatAction("allow_session")).toBe("Allow Session");
    expect(formatAction("deny")).toBe("Deny");
  });

  it("handles single-word actions", () => {
    expect(formatAction("accept")).toBe("Accept");
    expect(formatAction("reject")).toBe("Reject");
    expect(formatAction("edit")).toBe("Edit");
  });

  it("handles multi-underscore keys", () => {
    expect(formatAction("allow_always_for_session")).toBe(
      "Allow Always For Session"
    );
  });

  it("returns empty string for empty input", () => {
    expect(formatAction("")).toBe("");
  });
});

// ── btnClass ──────────────────────────────────────────────────────────────────

describe("btnClass", () => {
  it("deny always gets btn-danger", () => {
    expect(btnClass("deny", 0)).toBe("btn btn-danger");
    expect(btnClass("deny", 2)).toBe("btn btn-danger");
  });

  it("first action (i=0) gets btn-primary", () => {
    expect(btnClass("allow_once", 0)).toBe("btn btn-primary");
  });

  it("allow_session gets btn-primary regardless of index", () => {
    expect(btnClass("allow_session", 1)).toBe("btn btn-primary");
    expect(btnClass("allow_session", 2)).toBe("btn btn-primary");
  });

  it("allow_always gets btn-primary regardless of index", () => {
    expect(btnClass("allow_always", 2)).toBe("btn btn-primary");
  });

  it("other allow* at non-zero index gets btn-ghost", () => {
    expect(btnClass("allow_once", 1)).toBe("btn btn-ghost");
  });

  it("unknown action at non-zero index gets plain btn", () => {
    expect(btnClass("cancel", 1)).toBe("btn");
    expect(btnClass("edit", 2)).toBe("btn");
  });
});

// ── resolveKey ────────────────────────────────────────────────────────────────

const STANDARD_ACTIONS = ["allow_once", "allow_session", "deny"];

describe("resolveKey", () => {
  it("y/Y → first allow* action", () => {
    expect(resolveKey("y", STANDARD_ACTIONS)).toBe("allow_once");
    expect(resolveKey("Y", STANDARD_ACTIONS)).toBe("allow_once");
  });

  it("a/A → allow_session if present", () => {
    expect(resolveKey("a", STANDARD_ACTIONS)).toBe("allow_session");
    expect(resolveKey("A", STANDARD_ACTIONS)).toBe("allow_session");
  });

  it("a/A → allow_always if present", () => {
    const actions = ["allow_once", "allow_always", "deny"];
    expect(resolveKey("a", actions)).toBe("allow_always");
  });

  it("n/N → deny", () => {
    expect(resolveKey("n", STANDARD_ACTIONS)).toBe("deny");
    expect(resolveKey("N", STANDARD_ACTIONS)).toBe("deny");
  });

  it("Escape → deny when present", () => {
    expect(resolveKey("Escape", STANDARD_ACTIONS)).toBe("deny");
  });

  it("Escape → last action when no deny", () => {
    const actions = ["allow_once", "allow_session"];
    expect(resolveKey("Escape", actions)).toBe("allow_session");
  });

  it("y with no allow actions → null", () => {
    expect(resolveKey("y", ["deny"])).toBeNull();
  });

  it("n with no deny action → null", () => {
    expect(resolveKey("n", ["allow_once"])).toBeNull();
  });

  it("a with no allow_session/allow_always → null", () => {
    expect(resolveKey("a", ["allow_once", "deny"])).toBeNull();
  });

  it("unrecognised key → null", () => {
    expect(resolveKey("Enter", STANDARD_ACTIONS)).toBeNull();
    expect(resolveKey("x", STANDARD_ACTIONS)).toBeNull();
    expect(resolveKey("ArrowLeft", STANDARD_ACTIONS)).toBeNull();
  });

  it("empty actions list → null for all keys", () => {
    expect(resolveKey("y", [])).toBeNull();
    expect(resolveKey("Escape", [])).toBeNull();
  });
});
