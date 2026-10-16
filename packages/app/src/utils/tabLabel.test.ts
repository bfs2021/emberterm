import { describe, it, expect } from "vitest";
import { tabLabels, type LabelTab } from "./tabLabel.js";

const tab = (over: Partial<LabelTab>): LabelTab => ({ id: "t", title: "bash", customTitle: false, ...over });
const one = (t: LabelTab) => tabLabels([t]).get(t.id);

describe("tabLabels", () => {
  it("uses the last folder for a Git Bash default title", () => {
    expect(one(tab({ title: "MINGW64:/d/code/projects/prep-maker", cwd: "/d/code/projects/prep-maker" }))).toBe("prep-maker");
  });

  it("uses the last folder for Windows, WSL and trailing-slash paths", () => {
    expect(one(tab({ title: "D:\\code\\x", cwd: "D:/code/emberterm/" }))).toBe("emberterm");
    expect(one(tab({ title: "arifin@host: ~/api", cwd: "/home/arifin/api" }))).toBe("api");
  });

  it("keeps a title that a program set", () => {
    expect(one(tab({ title: "vim notes.md", cwd: "/d/code/x" }))).toBe("vim notes.md");
    expect(one(tab({ title: "✳ Claude Code", cwd: "/d/code/x" }))).toBe("✳ Claude Code");
  });

  it("always shows a name the user chose", () => {
    expect(one(tab({ title: "api", customTitle: true, cwd: "/d/code/x" }))).toBe("api");
  });

  it("falls back to the title when the folder is unknown", () => {
    expect(one(tab({ title: "cmd" }))).toBe("cmd");
    expect(one(tab({ title: "MINGW64:/c/Users/arifin" }))).toBe("MINGW64:/c/Users/arifin");
  });

  it("shows roots sensibly", () => {
    expect(one(tab({ title: "MINGW64:/", cwd: "/" }))).toBe("/");
    expect(one(tab({ title: "x", cwd: "D:/", customTitle: false }))).toBe("x");
    expect(one(tab({ title: "D:\\", cwd: "D:/" }))).toBe("D:");
  });

  it("adds the parent folder when two tabs would read the same", () => {
    const a = tab({ id: "a", title: "MINGW64:/d/a/api", cwd: "/d/a/api" });
    const b = tab({ id: "b", title: "MINGW64:/d/b/api", cwd: "/d/b/api" });
    const c = tab({ id: "c", title: "MINGW64:/d/c/web", cwd: "/d/c/web" });
    const labels = tabLabels([a, b, c]);
    expect(labels.get("a")).toBe("a/api");
    expect(labels.get("b")).toBe("b/api");
    expect(labels.get("c")).toBe("web");
  });

  it("does not disambiguate against user-chosen or program titles", () => {
    const a = tab({ id: "a", title: "MINGW64:/d/a/api", cwd: "/d/a/api" });
    const b = tab({ id: "b", title: "api", customTitle: true });
    expect(tabLabels([a, b]).get("a")).toBe("api");
  });
});
