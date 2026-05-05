import { describe, it, expect, beforeEach } from "vitest";
import { get } from "svelte/store";
import {
  tabs,
  toolCalls,
  addTab,
  removeTab,
  activateTab,
  setTabTitle,
  setTabCwd,
  setTabAgent,
  renameTab,
  addToolCall,
  updateToolCall,
  activeTab,
  activeCwd,
  activeAgentName,
} from "./app.js";

beforeEach(() => {
  tabs.set([]);
  toolCalls.set(new Map());
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

  it("setTabAgent marks agentActive and sets name", () => {
    addTab("s1", "Shell");
    setTabAgent("s1", "claude-code");
    const tab = get(tabs)[0];
    expect(tab.agentActive).toBe(true);
    expect(tab.agentName).toBe("claude-code");
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

  it("activeAgentName follows the active tab's agent", () => {
    addTab("s1", "Shell");
    setTabAgent("s1", "aider");
    expect(get(activeAgentName)).toBe("aider");
  });
});

// ── Tool call state transitions ───────────────────────────────────────────────

describe("tool call state transitions", () => {
  it("addToolCall inserts a call with running status", () => {
    addTab("s1", "Shell");
    addToolCall("s1", { id: "tc1", name: "read_file", summary: "Read foo.rs", status: "running" });
    const calls = get(toolCalls).get("s1");
    expect(calls).toHaveLength(1);
    expect(calls![0].id).toBe("tc1");
    expect(calls![0].status).toBe("running");
  });

  it("updateToolCall transitions running → ok with duration", () => {
    addToolCall("s1", { id: "tc1", name: "edit_file", summary: "Edit main.rs", status: "running" });
    updateToolCall("s1", "tc1", { status: "ok", durationMs: 1200, summary: "Edited 12 lines" });
    const call = get(toolCalls).get("s1")![0];
    expect(call.status).toBe("ok");
    expect(call.durationMs).toBe(1200);
    expect(call.summary).toBe("Edited 12 lines");
  });

  it("updateToolCall transitions running → error", () => {
    addToolCall("s1", { id: "tc1", name: "build", summary: "Run build", status: "running" });
    updateToolCall("s1", "tc1", { status: "error" });
    expect(get(toolCalls).get("s1")![0].status).toBe("error");
  });

  it("updateToolCall transitions running → cancelled", () => {
    addToolCall("s1", { id: "tc1", name: "test", summary: "Run tests", status: "running" });
    updateToolCall("s1", "tc1", { status: "cancelled" });
    expect(get(toolCalls).get("s1")![0].status).toBe("cancelled");
  });

  it("updateToolCall sets progress fields", () => {
    addToolCall("s1", { id: "tc1", name: "upload", summary: "Uploading…", status: "running" });
    updateToolCall("s1", "tc1", { progressPct: 60, progressMessage: "Applying patch…" });
    const call = get(toolCalls).get("s1")![0];
    expect(call.progressPct).toBe(60);
    expect(call.progressMessage).toBe("Applying patch…");
  });

  it("updateToolCall clears progress fields when undefined", () => {
    addToolCall("s1", { id: "tc1", name: "upload", summary: "Done", status: "running", progressPct: 80 });
    updateToolCall("s1", "tc1", { status: "ok", progressPct: undefined });
    const call = get(toolCalls).get("s1")![0];
    expect(call.progressPct).toBeUndefined();
  });

  it("multiple tool calls are tracked independently", () => {
    addToolCall("s1", { id: "tc1", name: "read_file", summary: "Reading", status: "running" });
    addToolCall("s1", { id: "tc2", name: "write_file", summary: "Writing", status: "running" });
    updateToolCall("s1", "tc1", { status: "ok" });
    const calls = get(toolCalls).get("s1")!;
    expect(calls.find((c) => c.id === "tc1")?.status).toBe("ok");
    expect(calls.find((c) => c.id === "tc2")?.status).toBe("running");
  });

  it("tool calls are scoped per session", () => {
    addToolCall("s1", { id: "tc1", name: "tool_a", summary: "Session 1", status: "running" });
    addToolCall("s2", { id: "tc1", name: "tool_b", summary: "Session 2", status: "running" });
    updateToolCall("s1", "tc1", { status: "ok" });
    expect(get(toolCalls).get("s1")![0].status).toBe("ok");
    expect(get(toolCalls).get("s2")![0].status).toBe("running");
  });
});
