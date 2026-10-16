import { describe, it, expect } from "vitest";
import { toSnapshot, clampActiveTab, type SavedSession } from "./sessionSnapshot.js";
import type { Tab } from "./app.js";

const tab = (over: Partial<Tab>): Tab => ({
  id: "t", sessionId: "s", title: "bash", customTitle: false,
  isActive: false, hasUnread: false, isRunning: true, ...over,
});

describe("toSnapshot", () => {
  it("captures panes, splits, custom titles and the active index", () => {
    const snap = toSnapshot([
      tab({ id: "a", profileId: "git-bash", cwd: "/d/api", title: "api", customTitle: true }),
      tab({ id: "b", isActive: true, profileId: "pwsh", cwd: "D:/x",
            splitSessionId: "s2", splitDirection: "vertical", splitProfileId: "wsl", splitCwd: "/home/me" }),
    ]);
    expect(snap).toEqual({
      version: 1,
      active_tab: 1,
      tabs: [
        { title: "api", custom_title: true, pane: { profile_id: "git-bash", cwd: "/d/api" } },
        { custom_title: false, pane: { profile_id: "pwsh", cwd: "D:/x" },
          split: { direction: "vertical", pane: { profile_id: "wsl", cwd: "/home/me" } } },
      ],
    });
  });

  it("omits unknown fields instead of writing undefined", () => {
    const snap = toSnapshot([tab({ isActive: true })]);
    expect(snap.tabs[0]).toEqual({ custom_title: false, pane: {} });
    expect(JSON.stringify(snap)).not.toContain("undefined");
  });

  it("uses 0 when no tab is active and handles empty lists", () => {
    expect(toSnapshot([tab({})]).active_tab).toBe(0);
    expect(toSnapshot([])).toEqual({ version: 1, active_tab: 0, tabs: [] });
  });
});

describe("clampActiveTab", () => {
  const s = (active: number, n: number): SavedSession =>
    ({ version: 1, active_tab: active, tabs: Array.from({ length: n }, () => ({ custom_title: false, pane: {} })) });
  it("clamps out-of-range indices", () => {
    expect(clampActiveTab(s(5, 2))).toBe(1);
    expect(clampActiveTab(s(-1, 2))).toBe(0);
    expect(clampActiveTab(s(1, 2))).toBe(1);
  });
});
