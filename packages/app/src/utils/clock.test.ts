import { describe, it, expect } from "vitest";
import { formatClock, msUntilNextMinute } from "./clock.js";

describe("formatClock", () => {
  it("pads hours and minutes", () => {
    expect(formatClock(new Date(2026, 9, 16, 15, 5))).toBe("15:05");
    expect(formatClock(new Date(2026, 9, 16, 0, 0))).toBe("00:00");
    expect(formatClock(new Date(2026, 9, 16, 9, 41))).toBe("09:41");
  });
});

describe("msUntilNextMinute", () => {
  it("counts to the next minute boundary", () => {
    expect(msUntilNextMinute(new Date(2026, 9, 16, 15, 24, 40, 250))).toBe(19_750);
  });
  it("waits a full minute when exactly on the boundary", () => {
    expect(msUntilNextMinute(new Date(2026, 9, 16, 15, 24, 0, 0))).toBe(60_000);
  });
  it("is never zero", () => {
    expect(msUntilNextMinute(new Date(2026, 9, 16, 15, 24, 59, 999))).toBe(1);
  });
});
