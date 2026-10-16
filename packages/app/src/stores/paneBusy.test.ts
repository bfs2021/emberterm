import { describe, it, expect, vi, beforeEach } from "vitest";
import { markPaneBusy, forgetPane, isPaneBusy } from "./paneBusy.js";

const fallback = vi.fn();
beforeEach(() => { fallback.mockReset(); forgetPane("s1"); });

describe("isPaneBusy", () => {
  it("trusts the shell's own markers over the process check", async () => {
    markPaneBusy("s1", true);
    expect(await isPaneBusy("s1", fallback)).toBe(true);
    markPaneBusy("s1", false);
    expect(await isPaneBusy("s1", fallback)).toBe(false);
    expect(fallback).not.toHaveBeenCalled();
  });

  it("asks the process check for shells that never sent markers", async () => {
    fallback.mockResolvedValue(true);
    expect(await isPaneBusy("s1", fallback)).toBe(true);
    expect(fallback).toHaveBeenCalledWith("s1");
  });

  it("forgets a closed pane", async () => {
    markPaneBusy("s1", true);
    forgetPane("s1");
    fallback.mockResolvedValue(false);
    expect(await isPaneBusy("s1", fallback)).toBe(false);
    expect(fallback).toHaveBeenCalled();
  });
});
