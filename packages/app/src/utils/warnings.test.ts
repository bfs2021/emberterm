import { describe, it, expect } from "vitest";
import { formatWarning } from "./warnings.js";

describe("formatWarning", () => {
  it("mentions the backup when present", () => {
    expect(formatWarning({ file: "config.toml", message: "line 3: bad", backup: "C:/x/config.toml.broken-1" }))
      .toBe("config.toml: line 3: bad — saved a copy as C:/x/config.toml.broken-1");
  });
  it("omits the backup when absent", () => {
    expect(formatWarning({ file: "a.toml", message: "bad", backup: null })).toBe("a.toml: bad");
  });
});
