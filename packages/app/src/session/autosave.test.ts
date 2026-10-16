import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { createAutosave } from "./autosave.js";
import type { SavedSession } from "../stores/sessionSnapshot.js";

const snap = (n: number): SavedSession => ({
  version: 1, active_tab: 0, tabs: Array.from({ length: n }, () => ({ custom_title: false, pane: {} })),
});

describe("createAutosave", () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  it("debounces to the latest snapshot", async () => {
    const save = vi.fn().mockResolvedValue(undefined);
    const a = createAutosave({ delayMs: 500, save, onError: vi.fn() });
    a.schedule(snap(1));
    a.schedule(snap(2));
    await vi.advanceTimersByTimeAsync(499);
    expect(save).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(1);
    expect(save).toHaveBeenCalledTimes(1);
    expect(save).toHaveBeenCalledWith(snap(2));
  });

  it("flush saves the pending snapshot immediately", async () => {
    const save = vi.fn().mockResolvedValue(undefined);
    const a = createAutosave({ delayMs: 500, save, onError: vi.fn() });
    a.schedule(snap(3));
    await a.flush();
    expect(save).toHaveBeenCalledWith(snap(3));
    await vi.advanceTimersByTimeAsync(1000);
    expect(save).toHaveBeenCalledTimes(1);
  });

  it("suspend_drops_pending_and_future_saves", async () => {
    const save = vi.fn().mockResolvedValue(undefined);
    const a = createAutosave({ delayMs: 500, save, onError: vi.fn() });
    a.schedule(snap(1));
    a.suspend();
    a.schedule(snap(0));
    await vi.advanceTimersByTimeAsync(1000);
    expect(save).not.toHaveBeenCalled();
  });

  it("reports save errors", async () => {
    const onError = vi.fn();
    const a = createAutosave({ delayMs: 10, save: vi.fn().mockRejectedValue(new Error("disk")), onError });
    a.schedule(snap(1));
    await vi.advanceTimersByTimeAsync(10);
    expect(onError).toHaveBeenCalledTimes(1);
  });
});
