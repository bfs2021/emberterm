import { describe, it, expect, beforeEach } from "vitest";
import { get } from "svelte/store";
import {
  tabs,
  addTab,
  removeTab,
  activateTab,
  setTabTitle,
  setTabCwd,
  renameTab,
  setTabSplit,
  clearTabSplit,
  reorderTab,
  activeTab,
  activeCwd,
} from "./app.js";

beforeEach(() => {
  tabs.set([]);
});

// ── Tab management ────────────────────────────────────────────────────────────

describe("tabs", () => {
  it("addTab creates an active tab", () => {
    const id = addTab("s1", "Shell");
    const ts = get(tabs);
    expect(ts).toHaveLength(1);
    expect(ts[0].id).toBe(id);
    expect(ts[0].isActive).toBe(true);
    expect(ts[0].title).toBe("Shell");
    expect(ts[0].sessionId).toBe("s1");
  });

  it("second addTab deactivates the first", () => {
    addTab("s1", "Tab 1");
    addTab("s2", "Tab 2");
    const ts = get(tabs);
    expect(ts[0].isActive).toBe(false);
    expect(ts[1].isActive).toBe(true);
  });

  it("activateTab switches active state", () => {
    const id1 = addTab("s1", "Tab 1");
    addTab("s2", "Tab 2");
    activateTab(id1);
    const ts = get(tabs);
    expect(ts.find((t) => t.id === id1)?.isActive).toBe(true);
    expect(ts.find((t) => t.sessionId === "s2")?.isActive).toBe(false);
  });

  it("removeTab removes and activates neighbour", () => {
    const id1 = addTab("s1", "Tab 1");
    addTab("s2", "Tab 2");
    removeTab(id1);
    const ts = get(tabs);
    expect(ts).toHaveLength(1);
    expect(ts[0].isActive).toBe(true);
  });

  it("setTabTitle updates by sessionId", () => {
    addTab("s1", "Shell");
    setTabTitle("s1", "My Process");
    expect(get(tabs)[0].title).toBe("My Process");
  });

  it("setTabCwd updates cwd", () => {
    addTab("s1", "Shell");
    setTabCwd("s1", "/home/user");
    expect(get(tabs)[0].cwd).toBe("/home/user");
  });

  it("renameTab updates title by tabId", () => {
    const id = addTab("s1", "Shell");
    renameTab(id, "My Tab");
    expect(get(tabs)[0].title).toBe("My Tab");
  });

  it("renameTab ignores blank names", () => {
    const id = addTab("s1", "Shell");
    renameTab(id, "   ");
    expect(get(tabs)[0].title).toBe("Shell");
  });
});

// ── Derived stores ────────────────────────────────────────────────────────────

describe("derived stores", () => {
  it("activeTab reflects the active tab", () => {
    const id1 = addTab("s1", "First");
    addTab("s2", "Second");
    activateTab(id1);
    expect(get(activeTab)?.id).toBe(id1);
  });

  it("activeCwd follows the active tab's cwd", () => {
    addTab("s1", "Shell");
    setTabCwd("s1", "/projects/foo");
    expect(get(activeCwd)).toBe("/projects/foo");
  });
});

// ── Split pane ────────────────────────────────────────────────────────────────

describe("split pane", () => {
  it("setTabSplit records split session and direction", () => {
    const tabId = addTab("s1", "Shell");
    setTabSplit(tabId, "s2", "horizontal");
    const t = get(tabs).find((t) => t.id === tabId)!;
    expect(t.splitSessionId).toBe("s2");
    expect(t.splitDirection).toBe("horizontal");
  });

  it("clearTabSplit removes split state", () => {
    const tabId = addTab("s1", "Shell");
    setTabSplit(tabId, "s2", "vertical");
    clearTabSplit(tabId);
    const t = get(tabs).find((t) => t.id === tabId)!;
    expect(t.splitSessionId).toBeUndefined();
    expect(t.splitDirection).toBeUndefined();
  });

  it("other tabs are unaffected by setTabSplit", () => {
    const t1 = addTab("s1", "Tab 1");
    const t2 = addTab("s2", "Tab 2");
    setTabSplit(t1, "s3", "horizontal");
    const tab2 = get(tabs).find((t) => t.id === t2)!;
    expect(tab2.splitSessionId).toBeUndefined();
  });
});

// ── Drag to reorder ───────────────────────────────────────────────────────────

describe("reorderTab", () => {
  it("moves tab forward in the list", () => {
    const t1 = addTab("s1", "A");
    const t2 = addTab("s2", "B");
    const t3 = addTab("s3", "C");
    reorderTab(t1, t3);
    const ids = get(tabs).map((t) => t.id);
    expect(ids).toEqual([t2, t1, t3]);
  });

  it("moves tab backward in the list", () => {
    const t1 = addTab("s1", "A");
    const t2 = addTab("s2", "B");
    const t3 = addTab("s3", "C");
    reorderTab(t3, t1);
    const ids = get(tabs).map((t) => t.id);
    expect(ids).toEqual([t3, t1, t2]);
  });

  it("noop when source equals target", () => {
    const t1 = addTab("s1", "A");
    const t2 = addTab("s2", "B");
    reorderTab(t1, t1);
    const ids = get(tabs).map((t) => t.id);
    expect(ids).toEqual([t1, t2]);
  });

  it("noop when tabId not found", () => {
    const t1 = addTab("s1", "A");
    reorderTab("nonexistent", t1);
    const ids = get(tabs).map((t) => t.id);
    expect(ids).toEqual([t1]);
  });
});
