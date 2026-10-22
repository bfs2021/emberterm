import { describe, it, expect } from "vitest";
import { dropSide, dropIndex } from "./tabDrop.js";

describe("dropSide", () => {
  it("splits a tab at its midpoint", () => {
    const rect = { left: 100, width: 100 };
    expect(dropSide(120, rect)).toBe("before");
    expect(dropSide(180, rect)).toBe("after");
  });
});

describe("dropIndex", () => {
  it("drops after the last tab at the end of the strip", () => {
    expect(dropIndex(0, 2, "after")).toBe(2);
  });

  it("drops before the target when moving right", () => {
    expect(dropIndex(0, 2, "before")).toBe(1);
  });

  it("drops before the first tab at the start of the strip", () => {
    expect(dropIndex(2, 0, "before")).toBe(0);
  });

  it("drops after the target when moving left", () => {
    expect(dropIndex(2, 0, "after")).toBe(1);
  });

  it("leaves a tab in place when dropped beside itself", () => {
    expect(dropIndex(1, 1, "before")).toBe(1);
    expect(dropIndex(1, 1, "after")).toBe(1);
  });
});
