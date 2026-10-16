import { describe, it, expect } from "vitest";
import { parseThemeConfig, resolveThemeId } from "./themeConfig.js";

describe("parseThemeConfig", () => {
  it("reads the saved theme id and imported themes", () => {
    expect(parseThemeConfig({ appearance: { theme: "dracula", custom_themes: ["{}", "{\"a\":1}"] } })).toEqual({
      themeId: "dracula",
      customJson: ["{}", "{\"a\":1}"],
    });
  });

  it("tolerates a missing or malformed config", () => {
    expect(parseThemeConfig(undefined)).toEqual({ themeId: null, customJson: [] });
    expect(parseThemeConfig({})).toEqual({ themeId: null, customJson: [] });
    expect(parseThemeConfig({ appearance: { theme: 5, custom_themes: "nope" } })).toEqual({ themeId: null, customJson: [] });
  });

  it("drops non-string entries from the imported list", () => {
    expect(parseThemeConfig({ appearance: { custom_themes: ["{}", 3, null] } }).customJson).toEqual(["{}"]);
  });
});

describe("resolveThemeId", () => {
  const ids = ["ember-dark", "dracula", "my-theme"];
  it("keeps a saved id that exists", () => {
    expect(resolveThemeId("dracula", ids, "ember-dark")).toBe("dracula");
  });
  it("falls back when the saved theme is gone or unset", () => {
    expect(resolveThemeId("deleted", ids, "ember-dark")).toBe("ember-dark");
    expect(resolveThemeId(null, ids, "ember-dark")).toBe("ember-dark");
  });
});
