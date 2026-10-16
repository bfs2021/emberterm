import { describe, it, expect } from "vitest";
import { clampPadding, DEFAULT_PADDING } from "./paddingStore.js";

describe("clampPadding", () => {
  it("keeps whole numbers in range", () => {
    expect(clampPadding(0)).toBe(0);
    expect(clampPadding(12)).toBe(12);
    expect(clampPadding(64)).toBe(64);
  });
  it("clamps out-of-range values", () => {
    expect(clampPadding(-5)).toBe(0);
    expect(clampPadding(200)).toBe(64);
  });
  it("rounds fractions", () => {
    expect(clampPadding(7.6)).toBe(8);
  });
  it("falls back for non-numbers", () => {
    expect(clampPadding(NaN)).toBe(DEFAULT_PADDING);
    expect(clampPadding(NaN, 20)).toBe(20);
  });
});
