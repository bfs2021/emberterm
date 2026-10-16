import { describe, it, expect } from "vitest";
import { primaryFamily, stackFor, pickableFamilies, type InstalledFont } from "./fontFamily.js";

const f = (name: string, monospaced: boolean): InstalledFont => ({ name, monospaced });
const all = [f("Arial", false), f("Cascadia Code", true), f("JetBrains Mono", true), f("Segoe UI", false)];

describe("primaryFamily", () => {
  it("reads the first family from a CSS stack", () => {
    expect(primaryFamily('"JetBrains Mono", "Cascadia Code", monospace')).toBe("JetBrains Mono");
    expect(primaryFamily("JetBrains Mono, Cascadia Code, monospace")).toBe("JetBrains Mono");
    expect(primaryFamily("'Fira Code', monospace")).toBe("Fira Code");
  });
  it("handles a single or empty value", () => {
    expect(primaryFamily("monospace")).toBe("monospace");
    expect(primaryFamily("")).toBe("");
  });
});

describe("stackFor", () => {
  it("quotes the name and keeps a generic fallback", () => {
    expect(stackFor("JetBrains Mono")).toBe('"JetBrains Mono", monospace');
  });
  it("drops quote characters from the name", () => {
    expect(stackFor('Bad"Name')).toBe('"BadName", monospace');
  });
});

describe("pickableFamilies", () => {
  it("lists monospaced fonts only by default", () => {
    expect(pickableFamilies(all, false, "").map((x) => x.name)).toEqual(["Cascadia Code", "JetBrains Mono"]);
  });
  it("lists everything when asked", () => {
    expect(pickableFamilies(all, true, "").map((x) => x.name)).toHaveLength(4);
  });
  it("keeps the current font visible even if it is not monospaced", () => {
    expect(pickableFamilies(all, false, "Segoe UI").map((x) => x.name)).toEqual(["Cascadia Code", "JetBrains Mono", "Segoe UI"]);
  });
});
