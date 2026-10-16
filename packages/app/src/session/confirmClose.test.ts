import { describe, it, expect, vi } from "vitest";
import { shouldConfirmClose } from "./confirmClose.js";

describe("shouldConfirmClose", () => {
  it("never asks, nor even checks, when the setting is off", async () => {
    const busy = vi.fn().mockResolvedValue(true);
    expect(await shouldConfirmClose(["a"], false, busy)).toBe(false);
    expect(busy).not.toHaveBeenCalled();
  });

  it("asks when any pane of the tab has a program running", async () => {
    const busy = vi.fn(async (id: string) => id === "split");
    expect(await shouldConfirmClose(["main", "split"], true, busy)).toBe(true);
  });

  it("does not ask for idle shells", async () => {
    expect(await shouldConfirmClose(["a", "b"], true, async () => false)).toBe(false);
  });

  it("does not nag when the check itself fails", async () => {
    expect(await shouldConfirmClose(["a"], true, async () => { throw new Error("gone"); })).toBe(false);
  });
});
