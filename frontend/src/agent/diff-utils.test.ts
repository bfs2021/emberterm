import { describe, it, expect } from "vitest";
import { parseDiff, diffStats, toSideBySide } from "./diff-utils.js";

const SIMPLE_PATCH = `--- a/src/main.rs
+++ b/src/main.rs
@@ -1,4 +1,5 @@
 fn main() {
-    println!("hello");
+    println!("hello, world");
+    // added comment
 }
`;

describe("parseDiff", () => {
  it("returns empty array for empty patch", () => {
    expect(parseDiff("")).toEqual([]);
  });

  it("classifies meta lines (--- / +++ / diff / index)", () => {
    const lines = parseDiff("--- a/foo.rs\n+++ b/foo.rs\n");
    expect(lines).toHaveLength(2);
    expect(lines[0].type).toBe("meta");
    expect(lines[1].type).toBe("meta");
  });

  it("classifies hunk header", () => {
    const lines = parseDiff("@@ -1,3 +1,3 @@\n");
    expect(lines).toHaveLength(1);
    expect(lines[0].type).toBe("hunk");
  });

  it("classifies add, del, and ctx lines", () => {
    const lines = parseDiff(SIMPLE_PATCH);
    const types = lines.map((l) => l.type);
    expect(types).toContain("meta");
    expect(types).toContain("hunk");
    expect(types).toContain("del");
    expect(types).toContain("add");
    expect(types).toContain("ctx");
  });

  it("strips the leading +/- sigil from text", () => {
    const lines = parseDiff("@@ -1 +1 @@\n-old line\n+new line\n");
    const del = lines.find((l) => l.type === "del")!;
    const add = lines.find((l) => l.type === "add")!;
    expect(del.text).toBe("old line");
    expect(add.text).toBe("new line");
  });

  it("assigns line numbers to add/del/ctx lines", () => {
    const lines = parseDiff("@@ -5,2 +5,2 @@\n old\n-removed\n+added\n");
    const ctx = lines.find((l) => l.type === "ctx")!;
    const del = lines.find((l) => l.type === "del")!;
    const add = lines.find((l) => l.type === "add")!;
    expect(ctx.oldNum).toBe(5);
    expect(ctx.newNum).toBe(5);
    expect(del.oldNum).toBe(6);
    expect(add.newNum).toBe(6);
  });

  it("does not treat +++ / --- as add/del lines", () => {
    const lines = parseDiff("--- a/x\n+++ b/x\n");
    expect(lines.every((l) => l.type === "meta")).toBe(true);
  });
});

describe("toSideBySide", () => {
  it("returns empty array for empty input", () => {
    expect(toSideBySide([])).toEqual([]);
  });

  it("pairs del and add lines from same hunk", () => {
    const lines = parseDiff("@@ -1 +1 @@\n-old\n+new\n");
    const sbs = toSideBySide(lines);
    const content = sbs.filter((r) => r.type === "content");
    expect(content).toHaveLength(1);
    expect(content[0].oldType).toBe("del");
    expect(content[0].newType).toBe("add");
    expect(content[0].oldText).toBe("old");
    expect(content[0].newText).toBe("new");
  });

  it("pads unmatched del with empty right side", () => {
    const lines = parseDiff("@@ -1,2 +1 @@\n-line1\n-line2\n+only\n");
    const sbs = toSideBySide(lines);
    const content = sbs.filter((r) => r.type === "content");
    expect(content).toHaveLength(2);
    expect(content[0].oldType).toBe("del");
    expect(content[0].newType).toBe("add");
    expect(content[1].oldType).toBe("del");
    expect(content[1].newType).toBe("empty");
  });

  it("ctx lines appear on both sides", () => {
    const lines = parseDiff("@@ -1 +1 @@\n same\n");
    const sbs = toSideBySide(lines);
    const content = sbs.filter((r) => r.type === "content");
    expect(content).toHaveLength(1);
    expect(content[0].oldType).toBe("ctx");
    expect(content[0].newType).toBe("ctx");
    expect(content[0].oldText).toBe(content[0].newText);
  });

  it("hunk header emits a hunk-type row", () => {
    const lines = parseDiff("@@ -1 +1 @@\n");
    const sbs = toSideBySide(lines);
    expect(sbs[0].type).toBe("hunk");
  });
});

describe("diffStats", () => {
  it("counts add and del lines", () => {
    const lines = parseDiff(SIMPLE_PATCH);
    const s = diffStats(lines);
    expect(s.add).toBe(2);
    expect(s.del).toBe(1);
  });

  it("returns zero for empty patch", () => {
    const s = diffStats([]);
    expect(s.add).toBe(0);
    expect(s.del).toBe(0);
  });

  it("ignores hunk and meta lines", () => {
    const lines = parseDiff("@@ -1 +1 @@\n unchanged\n");
    const s = diffStats(lines);
    expect(s.add).toBe(0);
    expect(s.del).toBe(0);
  });
});
