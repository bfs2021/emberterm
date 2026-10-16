import { describe, it, expect } from "vitest";
import { parseOsc7, parseOsc52 } from "./osc.js";

describe("parseOsc7", () => {
  it("reads a Git Bash MSYS path", () => {
    expect(parseOsc7("file://localhost/d/code/api")).toBe("/d/code/api");
  });
  it("reads a path with an empty host", () => {
    expect(parseOsc7("file:///home/arifin")).toBe("/home/arifin");
  });
  it("drops the slash before a Windows drive letter (pwsh)", () => {
    expect(parseOsc7("file://localhost/D:/code/api")).toBe("D:/code/api");
  });
  it("keeps spaces and # as-is", () => {
    expect(parseOsc7("file://localhost/D:/my #1 dir")).toBe("D:/my #1 dir");
    expect(parseOsc7("file://localhost/d/a b")).toBe("/d/a b");
  });
  it("percent-decodes encoded paths", () => {
    expect(parseOsc7("file://host/home/a%20b")).toBe("/home/a b");
  });
  it("keeps a literal % that is not valid encoding", () => {
    expect(parseOsc7("file://localhost/d/100%")).toBe("/d/100%");
  });
  it("rejects non-file payloads", () => {
    expect(parseOsc7("http://x/y")).toBeNull();
    expect(parseOsc7("file://hostonly")).toBeNull();
    expect(parseOsc7("")).toBeNull();
  });
});

describe("parseOsc52", () => {
  it("decodes a clipboard write", () => {
    expect(parseOsc52("c;" + btoa("hello"))).toBe("hello");
  });
  it("decodes UTF-8 text", () => {
    const b64 = btoa(String.fromCharCode(...new TextEncoder().encode("héllo ✓")));
    expect(parseOsc52("c;" + b64)).toBe("héllo ✓");
  });
  it("ignores clipboard read requests", () => {
    expect(parseOsc52("c;?")).toBeNull();
  });
  it("ignores malformed payloads", () => {
    expect(parseOsc52("c")).toBeNull();
    expect(parseOsc52("c;%%%")).toBeNull();
  });
});
