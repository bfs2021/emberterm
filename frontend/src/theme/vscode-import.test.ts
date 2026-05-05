import { describe, it, expect } from "vitest";
import { vscodeToTheme, parseVscodeTheme, type VsCodeTheme } from "./vscode-import.js";

const minimalDark: VsCodeTheme = {
  name: "Test Dark",
  type: "dark",
  colors: {
    "editor.background": "#1a1a2e",
    "terminal.foreground": "#e0e0e0",
    "sideBar.background": "#111122",
    "sideBar.border": "#333344",
    "focusBorder": "#ff4500",
    "errorForeground": "#ff3333",
    "terminal.ansiBlack": "#000000",
    "terminal.ansiRed": "#cc0000",
    "terminal.ansiGreen": "#00cc00",
    "terminal.ansiYellow": "#cccc00",
    "terminal.ansiBlue": "#0000cc",
    "terminal.ansiMagenta": "#cc00cc",
    "terminal.ansiCyan": "#00cccc",
    "terminal.ansiWhite": "#cccccc",
    "terminal.ansiBrightBlack": "#555555",
    "terminal.ansiBrightRed": "#ff5555",
    "terminal.ansiBrightGreen": "#55ff55",
    "terminal.ansiBrightYellow": "#ffff55",
    "terminal.ansiBrightBlue": "#5555ff",
    "terminal.ansiBrightMagenta": "#ff55ff",
    "terminal.ansiBrightCyan": "#55ffff",
    "terminal.ansiBrightWhite": "#ffffff",
  },
};

describe("vscodeToTheme", () => {
  it("preserves id and name", () => {
    const theme = vscodeToTheme(minimalDark, "test-dark");
    expect(theme.id).toBe("test-dark");
    expect(theme.name).toBe("Test Dark");
    expect(theme.kind).toBe("dark");
  });

  it("auto-generates id from name when not supplied", () => {
    const theme = vscodeToTheme(minimalDark);
    expect(theme.id).toBe("test-dark");
  });

  it("maps background and foreground CSS vars", () => {
    const theme = vscodeToTheme(minimalDark, "t");
    expect(theme.vars["--bg"]).toBe("#1a1a2e");
    expect(theme.vars["--bg-elevated"]).toBe("#111122");
    expect(theme.vars["--border"]).toBe("#333344");
    expect(theme.vars["--accent"]).toBe("#ff4500");
    expect(theme.vars["--danger"]).toBe("#ff3333");
  });

  it("maps xterm ANSI palette", () => {
    const theme = vscodeToTheme(minimalDark, "t");
    expect(theme.xterm.black).toBe("#000000");
    expect(theme.xterm.red).toBe("#cc0000");
    expect(theme.xterm.brightWhite).toBe("#ffffff");
  });

  it("xterm background matches terminal.background if present", () => {
    const vsc: VsCodeTheme = {
      ...minimalDark,
      colors: { ...minimalDark.colors, "terminal.background": "#0a0a0a" },
    };
    const theme = vscodeToTheme(vsc, "t");
    expect(theme.xterm.background).toBe("#0a0a0a");
  });

  it("falls back to editor.background when terminal.background missing", () => {
    const theme = vscodeToTheme(minimalDark, "t");
    expect(theme.xterm.background).toBe("#1a1a2e");
  });

  it("uses defaults for missing color keys", () => {
    const sparse: VsCodeTheme = { name: "Sparse", type: "dark", colors: {} };
    const theme = vscodeToTheme(sparse, "s");
    expect(theme.vars["--bg"]).toBe("#0d1117");
    expect(theme.xterm.black).toBe("#484f58");
  });
});

describe("parseVscodeTheme", () => {
  it("round-trips through JSON", () => {
    const json = JSON.stringify(minimalDark);
    const parsed = parseVscodeTheme(json);
    expect(parsed.name).toBe("Test Dark");
    expect(parsed.type).toBe("dark");
  });
});
